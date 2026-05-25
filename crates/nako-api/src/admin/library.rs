use nako_core::{LibraryId, MetadataProfile, MetadataScanAcquisitionPlan};
use serde::{Deserialize, Serialize};

use super::ADMIN_API_VERSION;
use crate::public_client::API_VERSION;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminUpdateLibraryMetadataProfileRequest {
    pub profile: MetadataProfile,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLibraryMetadataProfileResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub library_id: LibraryId,
    pub profile: MetadataProfile,
    pub scan_acquisition_plan: AdminMetadataScanAcquisitionPlan,
}

impl AdminLibraryMetadataProfileResponse {
    #[must_use]
    pub fn from_profile(library_id: LibraryId, profile: MetadataProfile) -> Self {
        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            library_id,
            scan_acquisition_plan: AdminMetadataScanAcquisitionPlan::from_plan(
                profile.scan_acquisition_plan(),
            ),
            profile,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataScanAcquisitionPlan {
    pub local_nfo_import: bool,
    pub provider_refresh: bool,
    pub addon_scrape: bool,
    pub addon_writeback: bool,
    pub embedded_read: bool,
    pub sidecar_read: bool,
    pub image_discovery: bool,
}

impl AdminMetadataScanAcquisitionPlan {
    #[must_use]
    pub const fn from_plan(plan: MetadataScanAcquisitionPlan) -> Self {
        Self {
            local_nfo_import: plan.local_nfo_import,
            provider_refresh: plan.provider_refresh,
            addon_scrape: plan.addon_scrape,
            addon_writeback: plan.addon_writeback,
            embedded_read: plan.embedded_read,
            sidecar_read: plan.sidecar_read,
            image_discovery: plan.image_discovery,
        }
    }
}
