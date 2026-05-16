mod artwork;
mod catalog;
mod item;
mod library;
mod metadata;
mod probe;
mod profile;
mod provider;
mod scan;
mod source;

pub use artwork::*;
pub use catalog::*;
pub use item::*;
pub use library::*;
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
}
