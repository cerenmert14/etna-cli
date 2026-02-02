use crate::service::config::setup;

/// Run the etna setup process using the service layer.
///
/// The CLI handles argument parsing and delegates to the service layer
/// for the actual setup logic.
pub fn invoke(overwrite: bool) -> anyhow::Result<()> {
    // Call service layer
    let result = setup(overwrite)?;

    tracing::info!(
        "etna configured at '{}' (version {})",
        result.etna_dir.display(),
        result.version
    );

    Ok(())
}
