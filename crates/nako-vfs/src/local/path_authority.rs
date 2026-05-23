use std::{
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use nako_core::{NakoError, Result, StorageErrorKind};

use crate::StorageUri;

pub(super) fn canonicalize_root(root: PathBuf) -> Result<PathBuf> {
    let root = root.canonicalize().map_err(|err| {
        NakoError::storage_io(
            root.display().to_string(),
            format!("failed to canonicalize local root: {err}"),
        )
    })?;

    if !root.is_dir() {
        return Err(NakoError::InvalidInput {
            message: format!("local root must be a directory: {}", root.display()),
        });
    }

    Ok(root)
}

pub(super) fn existing_path_for(root: &Path, uri: &StorageUri, scheme: &str) -> Result<PathBuf> {
    ensure_local_scheme(uri, scheme)?;

    let relative = relative_path(uri)?;
    let candidate = root.join(relative);
    let canonical = candidate.canonicalize().map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            NakoError::NotFound {
                entity: "storage_object",
                id: uri.to_string(),
            }
        } else {
            NakoError::storage_io(
                uri.to_string(),
                format!("failed to resolve local path: {err}"),
            )
        }
    })?;

    if !canonical.starts_with(root) {
        return Err(NakoError::storage(
            uri.to_string(),
            StorageErrorKind::SecurityViolation,
            "resolved local path escaped backend root",
        ));
    }

    Ok(candidate)
}

pub(super) fn writable_path_for(root: &Path, uri: &StorageUri, scheme: &str) -> Result<PathBuf> {
    ensure_local_scheme(uri, scheme)?;

    let relative = relative_path(uri)?;
    let candidate = root.join(relative);
    let parent = candidate.parent().ok_or_else(|| {
        NakoError::storage(
            uri.to_string(),
            StorageErrorKind::SecurityViolation,
            "local write target has no parent directory",
        )
    })?;
    let canonical_parent = parent.canonicalize().map_err(|err| {
        NakoError::storage_io(
            uri.to_string(),
            format!("failed to resolve local write parent: {err}"),
        )
    })?;

    if !canonical_parent.starts_with(root) {
        return Err(NakoError::storage(
            uri.to_string(),
            StorageErrorKind::SecurityViolation,
            "resolved local write path escaped backend root",
        ));
    }

    Ok(candidate)
}

pub(super) fn cleanup_path_for(root: &Path, uri: &StorageUri, scheme: &str) -> Result<PathBuf> {
    ensure_local_scheme(uri, scheme)?;

    let relative = relative_path(uri)?;
    let candidate = root.join(relative);
    let parent = candidate.parent().ok_or_else(|| {
        NakoError::storage(
            uri.to_string(),
            StorageErrorKind::SecurityViolation,
            "local cleanup target has no parent directory",
        )
    })?;
    let canonical_parent = parent.canonicalize().map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            NakoError::NotFound {
                entity: "storage_object",
                id: uri.to_string(),
            }
        } else {
            NakoError::storage_io(
                uri.to_string(),
                format!("failed to resolve local cleanup parent: {err}"),
            )
        }
    })?;

    if !canonical_parent.starts_with(root) {
        return Err(NakoError::storage(
            uri.to_string(),
            StorageErrorKind::SecurityViolation,
            "resolved local cleanup path escaped backend root",
        ));
    }

    Ok(candidate)
}

pub(super) fn uri_for_path(root: &Path, path: &Path) -> Result<StorageUri> {
    let relative = path.strip_prefix(root).map_err(|err| {
        NakoError::storage(
            path.display().to_string(),
            StorageErrorKind::SecurityViolation,
            format!("failed to build local uri: {err}"),
        )
    })?;

    let relative = relative.to_string_lossy().replace('\\', "/");
    StorageUri::from_parts("local", &relative)
}

pub(super) fn backup_uri_for_path(root: &Path, path: &Path) -> Result<StorageUri> {
    let relative = path.strip_prefix(root).map_err(|err| {
        NakoError::storage_security_violation(
            path.display().to_string(),
            format!("local backup path escaped backend root: {err}"),
        )
    })?;
    let relative = relative.to_string_lossy().replace('\\', "/");
    StorageUri::from_parts("local", &relative)
}

pub(super) fn ensure_local_scheme(uri: &StorageUri, scheme: &str) -> Result<()> {
    if uri.scheme() != scheme {
        return Err(NakoError::InvalidInput {
            message: format!(
                "local backend only accepts '{}' uris, got '{}'",
                scheme,
                uri.scheme()
            ),
        });
    }

    Ok(())
}

pub(super) fn relative_path(uri: &StorageUri) -> Result<PathBuf> {
    let raw = uri.path_part().trim_start_matches(['/', '\\']);
    let normalized = raw.replace('\\', "/");
    let mut relative = PathBuf::new();

    if normalized.is_empty() {
        return Ok(relative);
    }

    for component in Path::new(&normalized).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(NakoError::InvalidInput {
                    message: format!("local uri path is not allowed to escape root: {uri}"),
                });
            }
        }
    }

    Ok(relative)
}

pub(super) fn is_security_violation(err: &NakoError) -> bool {
    match err {
        NakoError::InvalidInput { message } => message.contains("escape root"),
        NakoError::Storage {
            kind: StorageErrorKind::SecurityViolation,
            ..
        } => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_rejects_parent_directory_traversal() {
        let uri = StorageUri::parse("local:///Movies/../Secrets/demo.mkv").unwrap();

        let err = relative_path(&uri).unwrap_err();

        assert!(is_security_violation(&err));
    }

    #[test]
    fn uri_for_path_normalizes_local_uri_separators() {
        let temp = tempfile::tempdir().unwrap();
        let movies = temp.path().join("Movies");
        std::fs::create_dir(&movies).unwrap();
        let source = movies.join("Demo.mkv");
        std::fs::write(&source, b"demo").unwrap();
        let root = canonicalize_root(temp.path().to_path_buf()).unwrap();

        let uri = uri_for_path(&root, &source.canonicalize().unwrap()).unwrap();

        assert_eq!(uri.as_str(), "local:///Movies/Demo.mkv");
    }
}
