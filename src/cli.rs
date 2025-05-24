use std::path::PathBuf;

use clap::{Parser, Subcommand};

use etna::commands::{self, store::query::QueryOption};

fn main() -> anyhow::Result<()> {
    // Initialize the logger
    env_logger::init();

    // Invoke the CLI
    run()
}

pub(crate) fn run() -> anyhow::Result<()> {
    let cli = Args::parse();

    match cli.command {
        Command::Experiment(exp) => match exp {
            ExperimentCommand::New {
                name,
                path,
                overwrite,
                description,
            } => commands::experiment::new_experiment::invoke(name, path, overwrite, description),
            ExperimentCommand::Run { name, tests } => {
                commands::experiment::run_experiment::invoke(name, tests)
            }
            ExperimentCommand::Show {
                hash,
                name,
                show_all,
            } => commands::experiment::show_experiment::invoke(hash, name, show_all),
        },
        Command::Workload(wl) => match wl {
            WorkloadCommand::AddWorkload {
                experiment,
                language,
                workload,
            } => commands::workload::add_workload::invoke(experiment, language, workload),
            WorkloadCommand::RemoveWorkload {
                experiment,
                language,
                workload,
            } => commands::workload::remove_workload::invoke(experiment, language, workload),
            WorkloadCommand::ListWorkloads {
                experiment,
                language,
                kind,
            } => commands::workload::list_workloads::invoke(experiment, language, kind),
        },
        Command::Config(cl) => match cl {
            ConfigCommand::ChangeBranch { branch } => {
                commands::config::change_branch::invoke(branch)
            }
            ConfigCommand::Show => commands::config::show::invoke(),
        },
        Command::Setup {
            overwrite,
            branch,
            repo_path,
        } => commands::config::setup::invoke(overwrite, branch, repo_path),
        Command::Store(store_command) => match store_command {
            StoreCommand::Write {
                experiment_id,
                metric,
            } => commands::store::write::invoke(experiment_id, metric),
            StoreCommand::Query(query_option) => commands::store::query::invoke(query_option),
        },
        Command::Analyze(_analyze_command) => todo!(),
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum ExperimentCommand {
    #[clap(name = "new", about = "Create a new experiment")]
    New {
        /// Name of the new experiment
        name: String,
        path: Option<PathBuf>,
        /// Overwrite the existing experiment
        #[clap(short = 'o', long)]
        overwrite: bool,
        /// Description of the experiment
        /// [default: A description of the experiment]
        #[clap(short = 'd', long)]
        description: Option<String>,
    },
    #[clap(name = "run", about = "Run an experiment")]
    Run {
        /// Name of the experiment to run
        /// [default: current directory]
        #[clap(short, long)]
        name: Option<String>,
        /// Tests to run
        #[clap(short, long)]
        tests: Vec<String>,
    },
    #[clap(name = "show", about = "Show the details of an experiment")]
    Show {
        /// Name
        #[clap(long)]
        name: Option<String>,
        /// Hash
        #[clap(long)]
        hash: Option<String>,
        /// Show all the experiments
        #[clap(short = 'a', long, default_value = "false")]
        show_all: bool,
    },
}
#[derive(Debug, Subcommand)]
enum WorkloadCommand {
    #[clap(name = "add", about = "Add a workload to the experiment")]
    AddWorkload {
        /// Name of the experiment
        /// [default: current directory]
        #[clap(short, long, default_value = None)]
        experiment: Option<String>,
        /// Language of the workload
        /// [default: coq]
        /// [possible_values(coq, haskell, racket)]
        language: String,
        /// Workload to be added
        /// [default: bst]
        /// [possible_values(bst, rbt, stlc, ifc)]
        workload: String,
    },
    #[clap(name = "remove", about = "Remove a workload from the experiment")]
    RemoveWorkload {
        /// Name of the experiment
        /// [default: current directory]
        #[clap(short, long, default_value = None)]
        experiment: Option<String>,
        /// Language of the workload
        /// [possible_values(coq, haskell, racket)]
        language: String,
        /// Workload to be removed
        /// [possible_values(bst, rbt, stlc, ifc)]
        workload: String,
    },
    #[clap(name = "list", about = "List all workloads")]
    ListWorkloads {
        /// Name of the experiment
        /// [default: current directory]
        #[clap(short, long, default_value = None)]
        experiment: Option<String>,
        /// Language of the workload
        /// [possible_values(coq, haskell, racket)]
        /// [default: all]
        #[clap(short, long, default_value = "all")]
        language: String,
        /// Available or experiment workloads
        /// [possible_values(available, experiment)]
        /// [default: experiment]
        #[clap(short, long, default_value = "experiment")]
        kind: String,
    },
}

#[derive(Debug, Subcommand)]
enum StoreCommand {
    #[clap(name = "write", about = "Write a metric to the store")]
    Write {
        /// Experiment ID
        experiment_id: String,
        /// Metric as a json string
        metric: String,
    },
    #[command(subcommand, name = "query", about = "Query the store")]
    Query(QueryOption),
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    #[command(
        name = "change-branch",
        about = "Change the branch of the etna repository"
    )]
    ChangeBranch {
        /// Branch to clone the etna repository
        #[clap(short, long)]
        branch: String,
    },
    #[command(name = "show", about = "Show the current configuration")]
    Show,
}

#[derive(Debug, Subcommand)]
enum AnalyzeCommand {
    #[clap(name = "bucket", about = "Create bucket charts for the experiment")]
    BucketGen {
        /// Name of the experiment to run
        /// [default: current directory]
        #[clap(short, long)]
        name: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(subcommand, name = "experiment", about = "Manage experiments")]
    Experiment(ExperimentCommand),
    #[command(subcommand, name = "workload", about = "Manage workloads")]
    Workload(WorkloadCommand),
    #[command(subcommand, name = "store", about = "Manage the etna store")]
    Store(StoreCommand),
    #[command(subcommand, name = "config", about = "Manage etna-cli configuration")]
    Config(ConfigCommand),
    #[command(name = "setup", about = "Setup etna-cli")]
    Setup {
        /// Overwrite the existing configuration
        #[clap(short, long, default_value = "false")]
        overwrite: bool,
        /// Branch to clone the etna repository
        #[clap(short, long, default_value = "main")]
        branch: String,
        /// Repository path, if already cloned
        #[clap(long, default_value = None)]
        repo_path: Option<String>,
    },
    #[command(
        subcommand,
        name = "analyze",
        about = "Run analysis on results of the experiments"
    )]
    Analyze(AnalyzeCommand),
}
