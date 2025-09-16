use anyhow::Context;
use clap::Subcommand;

use crate::{commands::store::lib::handle_jq_query, store::Store};

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
}

pub fn invoke(store: Store, filter: String) -> anyhow::Result<()> {
    handle_jq_query(store, filter).context("Failed to handle jq query")
}
