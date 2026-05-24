mod artwork;
mod candidate;
mod catalog;
mod item;
mod library;
mod merge;
mod metadata;
mod probe;
mod profile;
mod provider;
mod scan;
mod source;

pub use artwork::*;
pub use candidate::*;
pub use catalog::*;
pub use item::*;
pub use library::*;
pub use merge::*;
pub use metadata::*;
pub use probe::*;
pub use profile::*;
pub use provider::*;
pub use scan::*;
pub use source::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anime_preset_is_editable_configuration_not_media_kind() {
        let mut options = LibraryOptions::from_preset(LibraryPreset::Anime);

        assert_eq!(options.domain, MediaDomain::Video);
        assert_eq!(options.preset, LibraryPreset::Anime);
        assert_eq!(options.naming_strategy, NamingStrategy::Anime);
        assert!(
            options
                .metadata_profile
                .item_kinds
                .contains(&MediaKind::Movie)
        );
        assert!(
            options
                .metadata_profile
                .item_kinds
                .contains(&MediaKind::Episode)
        );
        assert_eq!(
            options.metadata_profile.metadata_providers,
            vec![
                ExternalProvider::Bangumi,
                ExternalProvider::Tmdb,
                ExternalProvider::Douban
            ]
        );

        options.metadata_profile.metadata_providers = vec![ExternalProvider::Tmdb];

        assert_eq!(
            options.metadata_profile.metadata_providers,
            vec![ExternalProvider::Tmdb]
        );
        assert!(!matches!(MediaKind::Movie, MediaKind::Unknown));
    }

    #[test]
    fn metadata_profile_builds_scan_acquisition_plan_from_local_readers_and_policy() {
        let mut profile = MetadataProfile::from_preset(LibraryPreset::Movies);

        let plan = profile.scan_acquisition_plan();

        assert!(plan.local_nfo_import);
        assert!(!plan.provider_refresh);
        assert!(!plan.addon_scrape);
        assert!(!plan.embedded_read);
        assert!(!plan.sidecar_read);
        assert!(!plan.image_discovery);

        profile.local_metadata_policy = LocalMetadataPolicy::Disabled;
        assert!(!profile.scan_acquisition_plan().local_nfo_import);

        profile.local_metadata_policy = LocalMetadataPolicy::LocalFirst;
        profile.local_readers.clear();
        assert!(!profile.scan_acquisition_plan().local_nfo_import);

        profile.local_readers = vec![LocalMetadataReader::Nfo];
        profile.scan = MetadataScanPolicy::disabled();
        assert!(!profile.scan_acquisition_plan().local_nfo_import);
    }
}
