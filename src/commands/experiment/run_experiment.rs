use anyhow::Context;
use log::{info, warn};

use crate::{
    config::{EtnaConfig, ExperimentConfig},
    drivers,
    experiment::Test,
    git_driver, python_driver,
    store::Store,
};

pub(crate) fn invoke(experiment_name: Option<String>, tests: Vec<String>) -> anyhow::Result<()> {
    let etna_config = EtnaConfig::get_etna_config()?;
    log::debug!("experiment_name: {:?}", experiment_name);
    let experiment_config = match experiment_name {
        Some(name) => ExperimentConfig::from_etna_config(&name, &etna_config).context(format!(
            "Failed to get experiment config for '{}'",
            name
        )),
        None => ExperimentConfig::from_current_dir().context("No experiment name is provided, and the current directory is not an experiment directory"),
    }?;

    let mut store = Store::load(&etna_config.store_path())?;

    let snapshot = Store::take_snapshot(&mut store, &etna_config, &experiment_config)?;

    let experiment = store.get_experiment_by_name(&experiment_config.name)?;

    let tests = tests
        .iter()
        .map(|test| {
            let test_path = experiment
                .path
                .join("tests")
                .join(test)
                .with_extension("json");
            let test: Vec<Test> =
                serde_json::from_str(&std::fs::read_to_string(test_path).unwrap()).unwrap();
            test
        })
        .flatten()
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
        store.save(&etna_config.store_path())?;
    }

    // python_driver::run_experiment(&etna_config, &experiment_config, snapshot)?;
    for test in &tests {
        match test.language.as_str() {
            "Racket" => {
                log::debug!("running Racket experiment");
                drivers::racket::run_experiment(test, &experiment_config, snapshot.clone())?
            }
            "Rocq" => {
                log::debug!("running Rocq experiment");
                drivers::rocq::run_experiment(test, &experiment_config, snapshot.clone())?
            }
            _ => {
                log::warn!("Unsupported language: {}", test.language);
                return Err(anyhow::anyhow!("Unsupported language: {}", test.language));
            }
        }
    }

    Ok(())
}
