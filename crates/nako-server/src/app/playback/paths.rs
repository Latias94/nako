use std::path::Path;

use nako_core::{NakoError, Result};

pub(super) fn path_exists(path: &Path) -> Result<bool> {
    path.try_exists().map_err(|err| {
        NakoError::storage_io(
            path.display().to_string(),
            format!("failed to check path: {err}"),
        )
    })
}

pub(super) async fn ensure_remux_output_parent(output_path: &Path) -> Result<()> {
    let Some(parent) = output_path.parent() else {
        return Err(NakoError::storage_security_violation(
            output_path.display().to_string(),
            "remux output path does not have a parent directory",
        ));
    };

    tokio::fs::create_dir_all(parent).await.map_err(|err| {
        NakoError::storage_io(
            parent.display().to_string(),
            format!("failed to create remux output directory: {err}"),
        )
    })
}
