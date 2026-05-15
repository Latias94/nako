use std::path::PathBuf;

use crate::config::{
    LocalLibraryConfig, TaruServerConfig, WebDavLibraryConfig, configured_library_config_for,
};
use taru_core::{Library, MediaSource, Result, TaruError};
use taru_db::SqliteStore;
use taru_vfs::{LocalFsBackend, StorageBackend, StorageUri};

pub(super) struct StorageBackendFactory<'a> {
    config: &'a TaruServerConfig,
    store: SqliteStore,
}

impl<'a> StorageBackendFactory<'a> {
    pub(super) fn new(config: &'a TaruServerConfig, store: SqliteStore) -> Self {
        Self { config, store }
    }

    pub(super) fn backend_for_library_root(
        &self,
        library: &Library,
    ) -> Result<Box<dyn StorageBackend>> {
        let config = configured_library_config_for(self.config, library.id)?;
        self.backend_for_library_config(&config)
    }

    pub(super) async fn backend_for_media_source(
        &self,
        source: &MediaSource,
    ) -> Result<(StorageUri, Box<dyn StorageBackend>)> {
        let uri = StorageUri::parse(&source.locator)?;
        let library_config = self
            .configured_library_config_for_source(source, &uri)
            .await?;
        let backend = self.backend_for_library_config(&library_config)?;

        Ok((uri, backend))
    }

    fn backend_for_library_config(
        &self,
        config: &LocalLibraryConfig,
    ) -> Result<Box<dyn StorageBackend>> {
        match config.webdav.as_ref() {
            Some(webdav) => self.webdav_storage_backend(webdav),
            None => Ok(Box::new(LocalFsBackend::new(&config.root)?)),
        }
    }

    async fn configured_library_config_for_source(
        &self,
        source: &MediaSource,
        uri: &StorageUri,
    ) -> Result<LocalLibraryConfig> {
        match configured_library_config_for(self.config, source.library_id) {
            Ok(config) => Ok(config),
            Err(TaruError::NotFound { .. }) => {
                let matches = self
                    .config
                    .libraries
                    .clone()
                    .into_iter()
                    .filter(|config| library_config_matches_uri(config, uri))
                    .collect::<Vec<_>>();

                match matches.as_slice() {
                    [config] => Ok(config.clone()),
                    [] => Err(TaruError::Unsupported(
                        "source library is not configured and source URI does not match any configured library backend",
                    )),
                    _ => Err(TaruError::Unsupported(
                        "source library is not configured and source URI matches multiple configured library backends",
                    )),
                }
            }
            Err(err) => Err(err),
        }
    }

    fn webdav_storage_backend(
        &self,
        config: &WebDavLibraryConfig,
    ) -> Result<Box<dyn StorageBackend>> {
        let backend = taru_vfs::WebDavBackend::new(webdav_backend_config(config))?;
        Ok(Box::new(taru_vfs::CachedStorageBackend::new(
            backend,
            self.store.clone(),
        )))
    }
}

pub(super) fn webdav_backend_config(config: &WebDavLibraryConfig) -> taru_vfs::WebDavBackendConfig {
    taru_vfs::WebDavBackendConfig {
        base_url: config.base_url.clone(),
        username: config.username.clone(),
        password_env: config.password_env.clone(),
        timeout_ms: config.timeout_ms,
        max_attempts: config.max_attempts,
    }
}

pub(super) fn library_config_matches_uri(config: &LocalLibraryConfig, uri: &StorageUri) -> bool {
    match (uri.scheme(), config.webdav.as_ref()) {
        ("local", None) => true,
        ("webdav", Some(webdav)) => storage_uri_is_within_root(uri.as_str(), &webdav.root),
        _ => false,
    }
}

pub(super) fn storage_uri_is_within_root(uri: &str, root: &str) -> bool {
    if uri == root {
        return true;
    }

    let root = root.trim_end_matches('/');
    uri.strip_prefix(root)
        .is_some_and(|rest| rest.starts_with('/'))
}

pub(super) fn remote_probe_staging_root(
    library: &Library,
    config: &TaruServerConfig,
) -> Option<PathBuf> {
    library
        .roots
        .iter()
        .any(|root| root.starts_with("webdav://"))
        .then(|| config.remux_staging_root.join("probe-inputs"))
}
