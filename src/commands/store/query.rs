use clap::Subcommand;

use crate::{service::store::query_metrics, store::Store};

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

/// Query metrics from the store using the service layer.
///
/// The CLI handles argument parsing and delegates to the service layer
/// for the actual query logic.
pub fn invoke(store: Store, filter: String) -> anyhow::Result<()> {
    // Call service layer
    let result = query_metrics(&store, &filter)?;

    // Format and display output
    println!("{}", serde_json::to_string_pretty(&result.metrics)?);

    Ok(())
}
