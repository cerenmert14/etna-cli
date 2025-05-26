use anyhow::Context;
use clap::Subcommand;

use lib::{handle_jq_query, handle_specialized_query};

use crate::{config::EtnaConfig, store::Store};

mod lib;

#[derive(Debug, Subcommand)]
pub enum QueryOption {
    #[clap(name = "--jq", about = "JQ Query")]
    Jq {
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

pub fn invoke(query_option: QueryOption) -> anyhow::Result<()> {
    let etna_config = EtnaConfig::get_etna_config()?;
    let store = Store::load(&etna_config.store_path())?;

    let use_jq = std::env::var("ETNA_USE_JQ")
        .unwrap_or("false".to_string())
        .parse::<bool>()?;

    match query_option {
        QueryOption::Jq { .. }
        | QueryOption::MetricsByFields { .. }
        | QueryOption::SnapshotsByFields { .. } => {
            handle_jq_query(store, query_option).context("Failed to handle jq query")
        }
        _ => {
            if use_jq {
                handle_jq_query(store, query_option).context("Failed to handle jq query")
            } else {
                handle_specialized_query(store, query_option)
                    .context("Failed to handle special query")
            }
        }
    }
}
