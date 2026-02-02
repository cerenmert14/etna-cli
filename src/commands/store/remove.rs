use crate::{
    service::{store::remove_metrics, types::RemoveMetricsOptions},
    store::Store,
};

/// Remove metrics from the store using the service layer.
///
/// The CLI handles argument parsing and delegates to the service layer
/// for the actual removal logic.
pub fn invoke(mut store: Store, filter: String) -> anyhow::Result<()> {
    // Convert CLI args to service options
    let options = RemoveMetricsOptions { filter };

    // Call service layer
    let removed_count = remove_metrics(&mut store, options)?;

    tracing::info!("Removed {} metrics from the store", removed_count);

    Ok(())
}
