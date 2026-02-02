use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

use crate::service::mutations::reset_mutations;

pub fn invoke(path: PathBuf) -> Result<()> {
    let path = if path == PathBuf::from(".") {
        std::env::current_dir()?
    } else {
        path.canonicalize().unwrap_or(path)
    };

    reset_mutations(&path)?;

    info!("Reset all mutations in {:?}", path);

    Ok(())
}
