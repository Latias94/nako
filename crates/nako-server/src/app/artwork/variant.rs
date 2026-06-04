use std::{fmt::Write as _, io::Cursor};

use image::{GenericImageView, imageops::FilterType};
use nako_core::{
    ManagedArtworkArtifactRecord, NakoError, Result, SelectedArtworkId, StorageErrorKind,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ImageVariantRequest {
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
}

impl ImageVariantRequest {
    #[must_use]
    pub(crate) const fn original() -> Self {
        Self {
            width: None,
            height: None,
        }
    }

    pub(crate) fn bounded(width: Option<u32>, height: Option<u32>) -> Result<Self> {
        validate_variant_edge("width", width)?;
        validate_variant_edge("height", height)?;
        Ok(Self { width, height })
    }

    #[must_use]
    const fn is_original(self) -> bool {
        self.width.is_none() && self.height.is_none()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImageVariantKey {
    Original,
    Bounded { width: u32, height: u32 },
}

fn validate_variant_edge(name: &str, value: Option<u32>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value == 0 {
        return Err(NakoError::InvalidInput {
            message: format!("{name} must be greater than zero"),
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ImageVariantPolicy {
    max_width: u32,
    max_height: u32,
}

impl ImageVariantPolicy {
    pub(super) const fn new(max_width: u32, max_height: u32) -> Self {
        Self {
            max_width,
            max_height,
        }
    }

    pub(super) fn validate(self, variant: ImageVariantRequest) -> Result<ValidatedImageVariant> {
        validate_variant_edge_against_limit("width", variant.width, self.max_width)?;
        validate_variant_edge_against_limit("height", variant.height, self.max_height)?;
        Ok(ValidatedImageVariant { variant })
    }
}

fn validate_variant_edge_against_limit(name: &str, value: Option<u32>, limit: u32) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value > limit {
        return Err(NakoError::InvalidInput {
            message: format!("{name} must be less than or equal to {limit}"),
        });
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ValidatedImageVariant {
    variant: ImageVariantRequest,
}

impl ValidatedImageVariant {
    pub(super) fn for_artifact(
        self,
        artifact: &ManagedArtworkArtifactRecord,
    ) -> Result<SelectedImageVariantPlan> {
        let original_media_type = artifact
            .media_type
            .clone()
            .ok_or_else(|| managed_artwork_variant_storage_error("media type is missing"))?;
        Ok(SelectedImageVariantPlan {
            variant: self.variant,
            original_media_type,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SelectedImageVariantPlan {
    variant: ImageVariantRequest,
    original_media_type: String,
}

impl SelectedImageVariantPlan {
    pub(super) fn preflight_etag(
        &self,
        selected_id: SelectedArtworkId,
        artifact: &ManagedArtworkArtifactRecord,
    ) -> Result<Option<ManagedArtworkImagePreflight>> {
        if super::image_format_for_media_type(&self.original_media_type).is_none() {
            return Err(managed_artwork_variant_storage_error(
                "media type is unsupported",
            ));
        }

        let variant = if self.variant.is_original() {
            ImageVariantKey::Original
        } else {
            let (Some(original_width), Some(original_height)) = (artifact.width, artifact.height)
            else {
                return Ok(None);
            };
            if original_width == 0 || original_height == 0 {
                return Ok(None);
            }

            let (width, height) =
                variant_dimensions(original_width, original_height, self.variant)?;
            if width == original_width && height == original_height {
                ImageVariantKey::Original
            } else {
                ImageVariantKey::Bounded { width, height }
            }
        };

        Ok(Some(ManagedArtworkImagePreflight {
            etag: public_selected_image_etag(selected_id, artifact, variant),
        }))
    }

    pub(super) fn derive(
        self,
        selected_id: SelectedArtworkId,
        artifact: &ManagedArtworkArtifactRecord,
        bytes: Vec<u8>,
    ) -> Result<ManagedArtworkImageBytes> {
        if self.variant.is_original() {
            return ManagedArtworkImageBytes {
                bytes,
                media_type: self.original_media_type,
                content_length: 0,
                etag: Some(public_selected_image_etag(
                    selected_id,
                    artifact,
                    ImageVariantKey::Original,
                )),
            }
            .with_content_length();
        }

        derive_selected_image_variant(
            selected_id,
            artifact,
            &self.original_media_type,
            bytes,
            self.variant,
        )
    }
}

fn derive_selected_image_variant(
    selected_id: SelectedArtworkId,
    artifact: &ManagedArtworkArtifactRecord,
    original_media_type: &str,
    bytes: Vec<u8>,
    variant: ImageVariantRequest,
) -> Result<ManagedArtworkImageBytes> {
    let (format, _extension) = super::image_format_for_media_type(original_media_type)
        .ok_or_else(|| managed_artwork_variant_storage_error("media type is unsupported"))?;
    let image =
        image::load_from_memory_with_format(&bytes, format).map_err(|_err| NakoError::Storage {
            uri: "managed-artwork://artifact".to_owned(),
            kind: StorageErrorKind::Unknown,
            message: "managed artwork artifact image is invalid".to_owned(),
        })?;
    let (original_width, original_height) = image.dimensions();
    let (width, height) = variant_dimensions(original_width, original_height, variant)?;
    if width == original_width && height == original_height {
        return ManagedArtworkImageBytes {
            bytes,
            media_type: original_media_type.to_owned(),
            content_length: 0,
            etag: Some(public_selected_image_etag(
                selected_id,
                artifact,
                ImageVariantKey::Original,
            )),
        }
        .with_content_length();
    }

    let resized = image.resize(width, height, FilterType::Lanczos3);
    let mut output = Cursor::new(Vec::new());
    resized
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|_err| NakoError::Storage {
            uri: "managed-artwork://artifact".to_owned(),
            kind: StorageErrorKind::Unknown,
            message: "failed to encode managed artwork image variant".to_owned(),
        })?;

    ManagedArtworkImageBytes {
        bytes: output.into_inner(),
        media_type: "image/png".to_owned(),
        content_length: 0,
        etag: Some(public_selected_image_etag(
            selected_id,
            artifact,
            ImageVariantKey::Bounded { width, height },
        )),
    }
    .with_content_length()
}

fn variant_dimensions(
    original_width: u32,
    original_height: u32,
    variant: ImageVariantRequest,
) -> Result<(u32, u32)> {
    if original_width == 0 || original_height == 0 {
        return Err(managed_artwork_variant_storage_error(
            "image dimensions are invalid",
        ));
    }

    let target_width = variant.width.unwrap_or(original_width);
    let target_height = variant.height.unwrap_or(original_height);
    let width_ratio = target_width as f64 / original_width as f64;
    let height_ratio = target_height as f64 / original_height as f64;
    let scale = width_ratio.min(height_ratio).min(1.0);
    let width = ((original_width as f64) * scale).round().max(1.0) as u32;
    let height = ((original_height as f64) * scale).round().max(1.0) as u32;

    Ok((width, height))
}

fn public_selected_image_etag(
    selected_id: SelectedArtworkId,
    artifact: &ManagedArtworkArtifactRecord,
    variant: ImageVariantKey,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"nako-public-image-etag-v1");
    hasher.update(selected_id.to_string().as_bytes());
    hasher.update(artifact.id.to_string().as_bytes());
    hasher.update(artifact.updated_at.as_bytes());
    match variant {
        ImageVariantKey::Original => hasher.update(b"original"),
        ImageVariantKey::Bounded { width, height } => {
            hasher.update(b"bounded");
            hasher.update(width.to_be_bytes());
            hasher.update(height.to_be_bytes());
        }
    }
    let digest = hasher.finalize();
    let mut output = String::from("nako-img-v1-");
    for byte in digest.iter().take(16) {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn managed_artwork_variant_storage_error(message: &str) -> NakoError {
    NakoError::Storage {
        uri: "managed-artwork://artifact".to_owned(),
        kind: StorageErrorKind::Unknown,
        message: format!("managed artwork artifact {message}"),
    }
}

impl ManagedArtworkImageBytes {
    fn with_content_length(mut self) -> Result<Self> {
        self.content_length =
            u64::try_from(self.bytes.len()).map_err(|err| NakoError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::Unknown,
                message: format!("managed artwork image length is too large: {err}"),
            })?;

        Ok(self)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ManagedArtworkImageBytes {
    pub(crate) bytes: Vec<u8>,
    pub(crate) media_type: String,
    pub(crate) content_length: u64,
    pub(crate) etag: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ManagedArtworkImagePreflight {
    pub(crate) etag: String,
}
