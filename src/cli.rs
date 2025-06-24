use std::{env, path::PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};

use etna::commands::{self, store::query::QueryOption};
use fern::colors::ColoredLevelConfig;

fn main() -> anyhow::Result<()> {
    // Initialize the logger
    let color_config = ColoredLevelConfig::new()
        .info(fern::colors::Color::Green)
        .debug(fern::colors::Color::Blue)
        .trace(fern::colors::Color::Cyan)
        .error(fern::colors::Color::Red);

    let log_level = env::var("RUST_LOG")
        .ok()
        .and_then(|lvl| lvl.parse::<log::LevelFilter>().ok());

    fern::Dispatch::new()
        .level(log_level.unwrap_or(log::LevelFilter::Info))
        .level_for("marauders", log::LevelFilter::Error)
        .level_for("ignore", log::LevelFilter::Error)
        .format(move |out, message, record| match record.level() {
            log::Level::Info if log_level.is_none() => out.finish(format_args!("{}", message,)),
            log::Level::Info
            | log::Level::Error
            | log::Level::Warn
            | log::Level::Debug
            | log::Level::Trace => out.finish(format_args!(
                "[{}][{}] {}",
                color_config.color(record.level()),
                record.target(),
                message
            )),
        })
        .chain(std::io::stdout())
        .apply()
        .context("Failed to initialize the logger")?;

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
                register,
                description,
                local_store,
            } => commands::experiment::new::invoke(name, path, overwrite, register, description, local_store),
            ExperimentCommand::Run { name, tests } => {
                commands::experiment::run::invoke(name, tests)
            }
            ExperimentCommand::Show {
                hash,
                name,
                show_all,
            } => commands::experiment::show::invoke(hash, name, show_all),
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
            } => commands::workload::remove_workload::invoke(experiment, language, workload)
                .context("Try running `etna workload remove` in an experiment directory, or explicitly specify the experiment name with `etna workload remove --experiment <NAME>`"),
            WorkloadCommand::ListWorkloads {
                experiment,
                language,
                kind,
            } => commands::workload::list_workloads::invoke(experiment, language, kind),
        },
        Command::Config(cl) => match cl {
            ConfigCommand::Show => commands::config::show::invoke(),
        },
        Command::Setup { overwrite } => commands::config::setup::invoke(overwrite),
        Command::Store(store_command) => match store_command {
            StoreCommand::Write {
                experiment_id,
                metric,
            } => commands::store::write::invoke(None, experiment_id, metric),
            StoreCommand::Query(query_option) => commands::store::query::invoke(query_option),
        },
        Command::Analyze(_analyze_command) => todo!(),
        Command::Check { restore, remove } => commands::check::integrity::invoke(restore, remove),
    }
    .context("Aborting run due to an error")
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
        /// An optional root path, if not provided, the current directory is used
        path: Option<PathBuf>,
        /// Overwrite the existing experiment
        #[clap(short = 'o', long)]
        overwrite: bool,
        /// Register the experiment in the store
        /// [default: false]
        #[clap(short = 'r', long)]
        register: bool,
        /// Description of the experiment
        /// [default: A description of the experiment]
        #[clap(short = 'd', long)]
        description: Option<String>,
        /// Does the experiment use a local store instead of the global store
        /// [default: false]
        #[clap(short = 's', long, default_value = "false")]
        local_store: bool,
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
        /// [possible_values(coq, haskell, racket, ocaml)]
        language: String,
        /// Workload to be added
        /// [default: bst]
        /// [possible_values(bst, rbt, stlc, systemf, ifc)]
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
        #[clap(short, long, default_value = "all")]
        language: String,
        /// Available or experiment workloads
        /// [possible_values(available, experiment)]
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
    },
    #[command(name = "check", about = "Run checks on etna")]
    Check {
        /// Restore the store from the backup
        #[clap(long, default_value = "false")]
        restore: bool,
        /// Remove the store
        #[clap(long, default_value = "false")]
        remove: bool,
    },
    #[command(
        subcommand,
        name = "analyze",
        about = "Run analysis on results of the experiments"
    )]
    Analyze(AnalyzeCommand),
}
