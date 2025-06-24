use anyhow::Context;

use crate::{
    config::{EtnaConfig, ExperimentConfig},
    store::{Metric, Store},
};

pub fn invoke(
    experiment_config: Option<&ExperimentConfig>,
    experiment_id: String,
    metric: String,
) -> anyhow::Result<()> {
    let store_path = if let Some(cfg) = experiment_config {
        &cfg.store
    } else {
        // Get Etna configuration
        let etna_config = EtnaConfig::get_etna_config().context("Failed to get etna config")?;
        &etna_config.store_path()
    };
    // Load the Store
    let mut store = Store::load(&store_path).context("Failed to load the store")?;

    // Deserialize the metric
    let data: serde_json::Value = serde_json::from_str(&metric).context(format!(
        "Failed to deserialize the metric as a json string '{}'",
        metric
    ))?;

    // Add the metric to the store
    store.metrics.push(Metric {
        experiment_id,
        data,
    });

    store.save().context("Failed to save the store")?;

    Ok(())
}
