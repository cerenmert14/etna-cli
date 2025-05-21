mod cli;
mod commands;
mod config;
mod drivers;
mod experiment;
mod git_driver;
mod property;
mod python_driver;
mod snapshot;
mod store;
mod strategy;
mod workload;

fn main() -> anyhow::Result<()> {
    // Initialize the logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Invoke the CLI
    cli::run()
}
