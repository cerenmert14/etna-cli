use anyhow::Context;
use log::{info, warn};

use crate::{
    config::{EtnaConfig, ExperimentConfig},
    driver::run_experiment,
    experiment::Test,
    git_driver,
    store::Store,
};

fn get_tests(
    test: Option<String>,
    tests: Vec<String>,
    experiment_config: &ExperimentConfig,
) -> anyhow::Result<Vec<Test>> {
    println!("test: {:?}, tests: {:?}", test, tests);

    if test.is_some() && !tests.is_empty() {
        anyhow::bail!("You can either specify a single test with --test or multiple tests with --tests, but not both at the same time.");
    }

    if tests.is_empty() && test.is_none() {
        anyhow::bail!("No tests provided. Please specify at least one test to run.");
    }

    if let Some(single_test) = test {
        let test = serde_json::from_str::<Test>(&single_test).with_context(|| {
            format!(
                "Failed to parse test from the provided string '{}'",
                single_test
            )
        })?;
        Ok(vec![test])
    } else {
        let mut all_tests = Vec::new();
        for test in tests {
            let test_path = experiment_config
                .path
                .join("tests")
                .join(test)
                .with_extension("json");
            let test: Vec<Test> = serde_json::from_str(&std::fs::read_to_string(&test_path)?)
                .context(format!(
                    "Failed to read test from '{}'",
                    test_path.display()
                ))?;
            all_tests.extend(test);
        }
        Ok(all_tests)
    }
}

pub fn invoke(
    experiment_name: Option<String>,
    test: Option<String>,
    tests: Vec<String>,
    short_circuit: bool,
    cross: bool,
) -> anyhow::Result<()> {
    log::trace!("running experiment with name '{:?}'", experiment_name);
    let etna_config = EtnaConfig::get_etna_config()?;

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

    let tests = get_tests(test, tests, &experiment_config)
        .context("Failed to get tests for the experiment")?;

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
    for test in &tests {
        info!("Running test: {}", test);
        run_experiment(test, &experiment_config, snapshot.clone(), short_circuit, cross)?;
    }

    Ok(())
}
