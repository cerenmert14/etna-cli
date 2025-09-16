use anyhow::Context;

use crate::store::{Metric, Store};

pub fn invoke(mut store: Store, experiment_id: String, metric: String) -> anyhow::Result<()> {
    // Deserialize the metric
    let data: serde_json::Value = serde_json::from_str(&metric).context(format!(
        "Failed to deserialize the metric as a json string '{}'",
        metric
    ))?;

    tracing::debug!(
        "Adding metric for experiment {}: {}",
        experiment_id,
        serde_json::to_string(&data).unwrap_or_else(|_| "Invalid JSON".to_string())
    );

    // Add the metric to the store
    store.push(Metric {
        experiment_id,
        data,
    })?;

    Ok(())
}
