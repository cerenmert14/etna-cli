use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

use crate::service::mutations::set_mutation;

pub fn invoke(path: PathBuf, variant: String, glob: Option<String>) -> Result<()> {
    let path = if path == PathBuf::from(".") {
        std::env::current_dir()?
    } else {
        path.canonicalize().unwrap_or(path)
    };

    set_mutation(&path, &variant, glob.as_deref())?;

    match &glob {
        Some(pattern) => info!(
            "Set mutation '{}' as active in {:?} (glob: {})",
            variant, path, pattern
        ),
        None => info!("Set mutation '{}' as active in {:?}", variant, path),
    }

    Ok(())
}
