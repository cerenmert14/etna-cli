use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

use crate::service::mutations::list_mutations;

pub fn invoke(path: PathBuf) -> Result<()> {
    let path = if &path == "." {
        std::env::current_dir()?
    } else {
        path.canonicalize().unwrap_or(path)
    };

    let files_with_mutations = list_mutations(&path)?;

    if files_with_mutations.is_empty() {
        info!("No mutations found in {:?}", path);
        return Ok(());
    }

    for file_info in files_with_mutations {
        let relative_path = file_info
            .file
            .strip_prefix(&path)
            .unwrap_or(&file_info.file);
        info!("File: {}", relative_path.display());

        for mutation in &file_info.mutations {
            let status = if mutation.active {
                "active"
            } else {
                "inactive"
            };
            info!(
                "  {} ({}) - lines {}-{}",
                mutation.name, status, mutation.line, mutation.end_line
            );
        }
    }

    Ok(())
}
