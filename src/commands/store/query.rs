use anyhow::Context;
use clap::Subcommand;

use crate::{
    commands::store::lib::handle_jq_query,
    config::{EtnaConfig, ExperimentConfig},
    store::Store,
};

#[derive(Debug, Subcommand)]
pub enum QueryOption {
    #[clap(name = "--jq", about = "JQ Query")]
    Jq {
        /// Name of the experiment
        /// [default: current directory]
        #[clap(short, long, default_value = None)]
        experiment: Option<String>,
        /// Query string
        query_string: String,
    },
    #[clap(name = "--experiment-by-id", about = "Get an experiment by id")]
    ExperimentById {
        /// Experiment ID
        experiment_id: String,
    },
    #[clap(name = "--experiment-by-name", about = "Get an experiment by name")]
    ExperimentByName {
        /// Experiment Name
        experiment_name: String,
    },
    #[clap(
        name = "--all-experiments-by-name",
        about = "Get all experiment for a given name"
    )]
    AllExperimentsByName {
        /// Experiment Name
        experiment_name: String,
    },
    #[clap(
        name = "--metrics-by-experiment-id",
        about = "Get all metrics for a given experiment id"
    )]
    MetricsByExperimentId {
        /// Experiment ID
        experiment_id: String,
    },
    #[clap(
        name = "--metrics-by-fields",
        about = "Get all metrics that match the given fields"
    )]
    MetricsByFields {
        /// Fields to match
        fields_json_string: String,
    },
    #[clap(
        name = "--snapshots-by-fields",
        about = "Get all snapshots that match the given fields"
    )]
    SnapshotsByFields {
        /// Fields to match
        fields_json_string: String,
    },
    #[clap(
        name = "--snapshots-by-name",
        about = "Get all snapshots for a given name"
    )]
    SnapshotsByName {
        /// Snapshot Name
        snapshot_name: String,
    },
    #[clap(
        name = "--snapshot-by-hash",
        about = "Get the snapshot for a given hash"
    )]
    SnapshotByHash {
        /// Snapshot Hash
        snapshot_hash: String,
    },
}

pub fn invoke(experiment: Option<String>, filter: String) -> anyhow::Result<()> {
    let etna_config = EtnaConfig::get_etna_config()?;

    let experiment_config = match experiment {
        Some(name) => ExperimentConfig::from_etna_config(&name, &etna_config).context(format!(
            "Failed to get experiment config for '{}'",
            name
        )),
        None => ExperimentConfig::from_current_dir().context("No experiment name is provided, and the current directory is not an experiment directory"),
    }?;
    // Load the Store
    let store = Store::load(&experiment_config.store)?;

    handle_jq_query(store, filter).context("Failed to handle jq query")
}
