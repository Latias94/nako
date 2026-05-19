use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use taru_core::{
    ManagedArtworkArtifactId, ManagedArtworkArtifactRecord, Result, SelectedArtworkId,
    StorageErrorKind, TaruError,
};
use tokio::{fs, io::AsyncWriteExt};

#[derive(Clone, Debug)]
pub(super) struct LocalManagedArtworkArtifactStore {
    root: PathBuf,
}

#[derive(Clone, Debug)]
pub(super) struct StoredManagedArtworkArtifact {
    pub(super) storage_uri: String,
    path: PathBuf,
}

impl LocalManagedArtworkArtifactStore {
    pub(super) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub(super) async fn write(
        &self,
        artifact_id: ManagedArtworkArtifactId,
        extension: &str,
        bytes: &[u8],
    ) -> std::result::Result<StoredManagedArtworkArtifact, ArtifactStoreWriteError> {
        let artifact_id_text = artifact_id.to_string();
        let shard = artifact_id_text.get(0..2).ok_or(ArtifactStoreWriteError)?;
        let directory = self.root.join(shard);
        let final_path = directory.join(format!("{artifact_id_text}.{extension}"));
        let temp_path = directory.join(format!("{artifact_id_text}.tmp"));

        let result = async {
            fs::create_dir_all(&directory).await?;
            let mut file = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temp_path)
                .await?;
            file.write_all(bytes).await?;
            file.sync_all().await?;
            drop(file);
            fs::rename(&temp_path, &final_path).await
        }
        .await;

        if result.is_err() {
            let _ = fs::remove_file(&temp_path).await;
            return Err(ArtifactStoreWriteError);
        }

        Ok(StoredManagedArtworkArtifact {
            storage_uri: format!("managed-artwork://artifact/{artifact_id_text}"),
            path: final_path,
        })
    }

    pub(super) async fn delete_best_effort(&self, stored: &StoredManagedArtworkArtifact) {
        if path_has_prefix(&stored.path, &self.root) {
            let _ = fs::remove_file(&stored.path).await;
        }
    }

    pub(super) async fn delete_artifact_best_effort(
        &self,
        artifact: &ManagedArtworkArtifactRecord,
    ) -> ArtifactFileDeleteOutcome {
        match self.path_for_artifact(artifact) {
            Ok(path) if path_has_prefix(&path, &self.root) => match fs::remove_file(&path).await {
                Ok(()) => ArtifactFileDeleteOutcome::Deleted,
                Err(err) if err.kind() == ErrorKind::NotFound => ArtifactFileDeleteOutcome::Missing,
                Err(_) => ArtifactFileDeleteOutcome::Failed,
            },
            _ => ArtifactFileDeleteOutcome::Failed,
        }
    }

    pub(super) async fn delete_discovered_file_best_effort(
        &self,
        path: &Path,
    ) -> ArtifactFileDeleteOutcome {
        if !path_has_prefix(path, &self.root) {
            return ArtifactFileDeleteOutcome::Failed;
        }
        match fs::remove_file(path).await {
            Ok(()) => ArtifactFileDeleteOutcome::Deleted,
            Err(err) if err.kind() == ErrorKind::NotFound => ArtifactFileDeleteOutcome::Missing,
            Err(_) => ArtifactFileDeleteOutcome::Failed,
        }
    }

    pub(super) async fn file_status(
        &self,
        artifact: &ManagedArtworkArtifactRecord,
    ) -> ArtifactFileStatus {
        let Ok(path) = self.path_for_artifact(artifact) else {
            return ArtifactFileStatus::UnresolvableExpectedPath;
        };

        match fs::metadata(&path).await {
            Ok(metadata) if metadata.is_file() => ArtifactFileStatus::Present,
            Ok(_) => ArtifactFileStatus::Missing,
            Err(err) if err.kind() == ErrorKind::NotFound => ArtifactFileStatus::Missing,
            Err(_) => ArtifactFileStatus::MetadataReadFailed,
        }
    }

    pub(super) async fn discover_files(
        &self,
        max_files: u32,
    ) -> Result<ArtifactStoreFileInventory> {
        let mut inventory = ArtifactStoreFileInventory::default();
        let mut directories = vec![self.root.clone()];

        while let Some(directory) = directories.pop() {
            let mut entries = match fs::read_dir(&directory).await {
                Ok(entries) => entries,
                Err(err) if err.kind() == ErrorKind::NotFound => continue,
                Err(_) => return Err(managed_artwork_artifact_store_inventory_error()),
            };

            loop {
                let entry = entries
                    .next_entry()
                    .await
                    .map_err(|_| managed_artwork_artifact_store_inventory_error())?;
                let Some(entry) = entry else {
                    break;
                };
                let path = entry.path();
                if !path_has_prefix(&path, &self.root) {
                    continue;
                }
                let file_type = entry
                    .file_type()
                    .await
                    .map_err(|_| managed_artwork_artifact_store_inventory_error())?;
                if file_type.is_dir() {
                    directories.push(path);
                    continue;
                }

                if inventory.scanned_files >= max_files {
                    inventory.truncated = true;
                    return Ok(inventory);
                }

                let byte_len = if file_type.is_file() {
                    entry.metadata().await.ok().map(|metadata| metadata.len())
                } else {
                    None
                };
                inventory.scanned_files = inventory.scanned_files.saturating_add(1);
                inventory
                    .files
                    .push(self.describe_discovered_file(path, byte_len));
            }
        }

        Ok(inventory)
    }

    fn describe_discovered_file(
        &self,
        path: PathBuf,
        byte_len: Option<u64>,
    ) -> DiscoveredArtifactFile {
        let layout = parse_discovered_artifact_file(&self.root, &path)
            .unwrap_or(DiscoveredArtifactFileLayout::Unrecognized);
        DiscoveredArtifactFile {
            path,
            layout,
            byte_len,
        }
    }

    pub(super) async fn read(
        &self,
        selected_id: SelectedArtworkId,
        artifact: &ManagedArtworkArtifactRecord,
    ) -> Result<Vec<u8>> {
        let path = self.path_for_artifact(artifact)?;

        fs::read(&path).await.map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                TaruError::NotFound {
                    entity: "selected_artwork_image",
                    id: selected_id.to_string(),
                }
            } else {
                TaruError::Storage {
                    uri: "managed-artwork://artifact".to_owned(),
                    kind: StorageErrorKind::Io,
                    message: "failed to read managed artwork artifact".to_owned(),
                }
            }
        })
    }

    pub(super) fn path_for_artifact(
        &self,
        artifact: &ManagedArtworkArtifactRecord,
    ) -> Result<PathBuf> {
        let expected_storage_uri = format!("managed-artwork://artifact/{}", artifact.id);
        if artifact.storage_uri != expected_storage_uri {
            return Err(TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::SecurityViolation,
                message: "managed artwork artifact storage reference is invalid".to_owned(),
            });
        }

        let Some(media_type) = artifact.media_type.as_deref() else {
            return Err(TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::Unknown,
                message: "managed artwork artifact media type is missing".to_owned(),
            });
        };
        let extension =
            image_extension_for_media_type(media_type).ok_or_else(|| TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::Unknown,
                message: "managed artwork artifact media type is unsupported".to_owned(),
            })?;
        let artifact_id_text = artifact.id.to_string();
        let shard = artifact_id_text
            .get(0..2)
            .ok_or_else(|| TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::Unknown,
                message: "managed artwork artifact id is invalid".to_owned(),
            })?;
        let path = self
            .root
            .join(shard)
            .join(format!("{artifact_id_text}.{extension}"));
        if !path_has_prefix(&path, &self.root) {
            return Err(TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::SecurityViolation,
                message: "managed artwork artifact path escaped artifact root".to_owned(),
            });
        }

        Ok(path)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ArtifactStoreWriteError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ArtifactFileDeleteOutcome {
    Deleted,
    Missing,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ArtifactFileStatus {
    Present,
    Missing,
    UnresolvableExpectedPath,
    MetadataReadFailed,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ArtifactStoreFileInventory {
    pub(super) scanned_files: u32,
    pub(super) files: Vec<DiscoveredArtifactFile>,
    pub(super) truncated: bool,
}

#[derive(Clone, Debug)]
pub(super) struct DiscoveredArtifactFile {
    pub(super) path: PathBuf,
    pub(super) layout: DiscoveredArtifactFileLayout,
    byte_len: Option<u64>,
}

impl DiscoveredArtifactFile {
    pub(super) fn into_classified(
        self,
        issue: ArtifactStoreFileIssue,
    ) -> ClassifiedArtifactStoreFile {
        let (recognized_artifact_id, extension) = match self.layout {
            DiscoveredArtifactFileLayout::Recognized {
                artifact_id,
                extension,
                ..
            } => (Some(artifact_id), Some(extension)),
            DiscoveredArtifactFileLayout::Unrecognized => (None, None),
        };
        ClassifiedArtifactStoreFile {
            path: self.path,
            issue,
            recognized_artifact_id,
            extension,
            byte_len: self.byte_len,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum DiscoveredArtifactFileLayout {
    Recognized {
        artifact_id: ManagedArtworkArtifactId,
        extension: String,
        supported_extension: bool,
        shard_matches: bool,
    },
    Unrecognized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ArtifactStoreFileIssue {
    UntrackedArtifactFile,
    UnexpectedActiveArtifactPath,
    UnsupportedExtension,
    UnrecognizedLayout,
}

#[derive(Clone, Debug)]
pub(super) struct ClassifiedArtifactStoreFile {
    pub(super) path: PathBuf,
    pub(super) issue: ArtifactStoreFileIssue,
    pub(super) recognized_artifact_id: Option<ManagedArtworkArtifactId>,
    pub(super) extension: Option<String>,
    pub(super) byte_len: Option<u64>,
}

fn parse_discovered_artifact_file(
    root: &Path,
    path: &Path,
) -> Option<DiscoveredArtifactFileLayout> {
    let relative = path.strip_prefix(root).ok()?;
    let mut components = relative.components();
    let shard = components.next()?.as_os_str().to_str()?;
    let file_name = components.next()?.as_os_str().to_str()?;
    if components.next().is_some() {
        return Some(DiscoveredArtifactFileLayout::Unrecognized);
    }

    let (stem, extension) = file_name.rsplit_once('.')?;
    let artifact_id = stem.parse::<ManagedArtworkArtifactId>().ok()?;
    let expected_shard = stem.get(0..2)?;
    let normalized_extension = extension.to_ascii_lowercase();

    Some(DiscoveredArtifactFileLayout::Recognized {
        artifact_id,
        extension: normalized_extension.clone(),
        supported_extension: supported_artifact_file_extension(&normalized_extension),
        shard_matches: shard == expected_shard,
    })
}

fn supported_artifact_file_extension(extension: &str) -> bool {
    matches!(extension, "jpg" | "png" | "webp")
}

fn image_extension_for_media_type(media_type: &str) -> Option<&'static str> {
    match media_type {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}

fn path_has_prefix(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
}

fn managed_artwork_artifact_store_inventory_error() -> TaruError {
    TaruError::Storage {
        uri: "managed-artwork://artifact".to_owned(),
        kind: StorageErrorKind::Io,
        message: "failed to inventory managed artwork artifact store".to_owned(),
    }
}
