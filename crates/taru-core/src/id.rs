use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
        )]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            #[must_use]
            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl From<$name> for Uuid {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
                Uuid::parse_str(value).map(Self)
            }
        }
    };
}

define_id!(LibraryId);
define_id!(MediaItemId);
define_id!(MediaSourceId);
define_id!(JobId);
define_id!(JobWorkerId);
define_id!(JobRunToken);
define_id!(EventId);
define_id!(AutomationProviderId);
define_id!(AutomationArtifactId);
define_id!(WebhookEndpointId);
define_id!(WebhookDeliveryAttemptId);
define_id!(AddonId);
define_id!(AddonTokenId);
define_id!(AddonGrantId);
define_id!(AddonSideEffectId);
define_id!(PersonId);
define_id!(GenreId);
define_id!(TagId);
define_id!(CollectionId);
define_id!(StudioId);
define_id!(ImageAssetId);
define_id!(ArtworkCandidateId);
define_id!(ManagedArtworkIngestId);
define_id!(ManagedArtworkArtifactId);
define_id!(ManagedImportArtifactId);
define_id!(ManagedImportPromotionApplyId);
define_id!(NfoSidecarApplyId);
define_id!(SelectedArtworkId);
define_id!(ScanSnapshotId);
define_id!(ArtworkTaskId);
define_id!(TranscodeSessionId);
define_id!(StagingManifestId);
define_id!(MetadataProviderAttemptId);
define_id!(ProviderSubjectId);
define_id!(ProviderMappingId);
define_id!(SourceDuplicateRelationshipId);
define_id!(LocalInferenceEvidenceId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_round_trips_through_string() {
        let id = MediaItemId::new();
        let parsed = id.to_string().parse::<MediaItemId>().unwrap();

        assert_eq!(id, parsed);
    }
}
