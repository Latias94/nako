use std::collections::{HashMap, HashSet};

use taru_core::{Library, LibraryId, LibraryRepository, PageRequest, Result, TaruError};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ConfiguredLibraryReconciliationReport {
    pub configured_libraries: usize,
    pub added_libraries: usize,
    pub updated_libraries: usize,
    pub unchanged_libraries: usize,
    pub retained_unconfigured_libraries: usize,
}

#[derive(Debug)]
pub(crate) struct ConfiguredLibraryReconciliationService<'a, R>
where
    R: LibraryRepository + ?Sized,
{
    repository: &'a R,
}

impl<'a, R> ConfiguredLibraryReconciliationService<'a, R>
where
    R: LibraryRepository + ?Sized,
{
    pub(crate) const fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub(crate) async fn reconcile(
        &self,
        desired_libraries: Vec<Library>,
    ) -> Result<ConfiguredLibraryReconciliationReport> {
        validate_desired_libraries(&desired_libraries)?;

        let persisted_libraries = self.list_all_persisted_libraries().await?;
        let persisted_by_id = persisted_libraries
            .iter()
            .map(|library| (library.id, library))
            .collect::<HashMap<_, _>>();
        let desired_ids = desired_libraries
            .iter()
            .map(|library| library.id)
            .collect::<HashSet<_>>();
        let mut report = ConfiguredLibraryReconciliationReport {
            configured_libraries: desired_libraries.len(),
            retained_unconfigured_libraries: persisted_libraries
                .iter()
                .filter(|library| !desired_ids.contains(&library.id))
                .count(),
            ..ConfiguredLibraryReconciliationReport::default()
        };

        for library in desired_libraries {
            match persisted_by_id.get(&library.id) {
                None => report.added_libraries += 1,
                Some(persisted) if *persisted == &library => report.unchanged_libraries += 1,
                Some(_) => report.updated_libraries += 1,
            }

            self.repository.upsert_library(&library).await?;
        }

        Ok(report)
    }

    async fn list_all_persisted_libraries(&self) -> Result<Vec<Library>> {
        let mut libraries = Vec::new();
        let mut offset = 0;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let mut batch = self.repository.list_libraries(page).await?;
            let returned = batch.len();
            libraries.append(&mut batch);

            if returned < PageRequest::MAX_LIMIT as usize {
                return Ok(libraries);
            }

            offset =
                offset
                    .checked_add(returned as u64)
                    .ok_or_else(|| TaruError::InvalidInput {
                        message: "library reconciliation pagination offset overflowed".to_owned(),
                    })?;
        }
    }
}

fn validate_desired_libraries(libraries: &[Library]) -> Result<()> {
    if libraries.is_empty() {
        return Err(TaruError::InvalidInput {
            message: "server config must include at least one library".to_owned(),
        });
    }

    let mut seen = HashSet::<LibraryId>::new();
    for library in libraries {
        if !seen.insert(library.id) {
            return Err(TaruError::InvalidInput {
                message: format!("duplicate configured library id: {}", library.id),
            });
        }
    }

    Ok(())
}
