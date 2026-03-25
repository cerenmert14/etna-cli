use std::path::{PathBuf, Path};

use anyhow::Result;
use tracing::info;

use crate::service::mutations::reset_mutations;

pub fn invoke(path: PathBuf) -> Result<()> {
     let path = if path == Path::new(".") {
        std::env::current_dir()?
    } else {
        path.canonicalize().unwrap_or(path)
    };

    reset_mutations(&path)?;

    info!("Reset all mutations in {:?}", path);

    Ok(())
}
