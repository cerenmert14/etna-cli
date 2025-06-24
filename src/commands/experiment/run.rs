use anyhow::Context;
use log::{info, warn};

use crate::{
    config::{EtnaConfig, ExperimentConfig},
    driver::{DefaultDriver, Driver},
    experiment::Test,
    git_driver,
    store::Store,
};

pub fn invoke(experiment_name: Option<String>, tests: Vec<String>) -> anyhow::Result<()> {
    log::trace!("running experiment with name '{:?}'", experiment_name);
    let etna_config = EtnaConfig::get_etna_config()?;

    if tests.is_empty() {
        anyhow::bail!("No tests provided. Please specify at least one test to run.");
    }

    let experiment_config = match experiment_name {
        Some(name) => ExperimentConfig::from_etna_config(&name, &etna_config).context(format!(
            "Failed to get experiment config for '{}'",
            name
        )),
        None => ExperimentConfig::from_current_dir().context("No experiment name is provided, and the current directory is not an experiment directory"),
    }?;

    let mut store = Store::load(&experiment_config.store)?;

    let snapshot = Store::take_snapshot(&mut store, &experiment_config)?;

    let experiment = store.get_experiment_by_name(&experiment_config.name)?;

    let tests = tests
        .iter()
        .flat_map(|test| {
            let test_path = experiment
                .path
                .join("tests")
                .join(test)
                .with_extension("json");
            let test: Vec<Test> =
                serde_json::from_str(&std::fs::read_to_string(test_path).unwrap()).unwrap();
            test
        })
        .collect::<Vec<Test>>();
    info!(
        "Taking snapshot for the experiment {}",
        experiment_config.name
    );

    if snapshot != experiment.snapshot {
        warn!(
            "Updating snapshot for the experiment {}",
            experiment_config.name
        );

        git_driver::print_diff(&snapshot, &experiment.snapshot)?;

        let experiment = experiment.with_snapshot(snapshot.clone());
        store.experiments.insert(experiment);
        store.save()?;
    }

    // python_driver::run_experiment(&etna_config, &experiment_config, snapshot)?;
    let driver = DefaultDriver {};
    driver.init();
    for test in &tests {
        info!("Running test: {}", test);
        driver.run_experiment(test, &experiment_config, snapshot.clone())?;
    }

    Ok(())
}
