use nako_core::{MediaRepository, MediaSource, NakoError, PageRequest, Result};
use nako_vfs::{StorageBackend, StorageUri};

use super::{NfoCodec, NfoService, NfoSidecar};

impl<B, R, C> NfoService<B, R, C> {
    pub fn new(backend: B, repository: R, codec: C) -> Self {
        Self {
            backend,
            repository,
            codec,
        }
    }
}

impl<B, R, C> NfoService<B, R, C>
where
    B: StorageBackend,
    R: MediaRepository,
    C: NfoCodec,
{
    pub async fn discover_sidecars(
        &self,
        library_id: nako_core::LibraryId,
    ) -> Result<Vec<NfoSidecar>> {
        let sources = self.list_all_sources(library_id).await?;
        let mut sidecars = Vec::new();

        for source in sources {
            let nfo_uri = nfo_uri_for_source(&source)?;
            match self.backend.stat(&nfo_uri).await {
                Ok(_) => {
                    sidecars.push(NfoSidecar {
                        source_id: source.id,
                        item_id: source.item_id,
                        source_locator: source.locator,
                        nfo_uri,
                    });
                }
                Err(NakoError::NotFound { .. }) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(sidecars)
    }

    pub(crate) async fn list_all_sources(
        &self,
        library_id: nako_core::LibraryId,
    ) -> Result<Vec<MediaSource>> {
        let mut offset = 0;
        let mut sources = Vec::new();

        loop {
            let page = self
                .repository
                .list_media_sources(
                    library_id,
                    PageRequest {
                        limit: PageRequest::MAX_LIMIT,
                        offset,
                    },
                )
                .await?;
            let returned = page.len();
            sources.extend(page);

            if returned < PageRequest::MAX_LIMIT as usize {
                break;
            }

            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(sources)
    }
}

pub(crate) fn nfo_uri_for_source(source: &MediaSource) -> Result<StorageUri> {
    let uri = StorageUri::parse(&source.locator)?;
    let path = uri.path_part();
    let Some((stem, _extension)) = path.rsplit_once('.') else {
        return Err(NakoError::InvalidInput {
            message: format!(
                "media source has no extension for NFO sidecar: {}",
                source.locator
            ),
        });
    };

    StorageUri::parse(format!("{}://{stem}.nfo", uri.scheme()))
}
