use std::sync::{Arc, Mutex};

use anyhow::Context;
use tracing::info;

use crate::{
    driver::run_experiment,
    experiment::{ExperimentMetadata, Test},
    git_driver,
    manager::Manager,
};

fn get_tests(tests: Vec<String>, experiment: &ExperimentMetadata) -> anyhow::Result<Vec<Test>> {
    println!("tests: {:?}", tests);

    if tests.is_empty() {
        anyhow::bail!("No tests provided. Please specify at least one test to run. Try running `etna experiment list-tests` to see available tests.");
    }

    let mut all_tests = Vec::new();
    for test in tests {
        let test_path = experiment
            .path
            .join("tests")
            .join(test)
            .with_extension("json");
        let test: Vec<Test> = serde_json::from_str(&std::fs::read_to_string(&test_path)?).context(
            format!("Failed to read test from '{}'", test_path.display()),
        )?;
        all_tests.extend(test);
    }
    Ok(all_tests)
}

pub fn invoke(
    mut mgr: Manager,
    experiment: ExperimentMetadata,
    tests: Vec<String>,
    short_circuit: bool,
    parallel: bool,
) -> anyhow::Result<()> {
    tracing::trace!("running experiment with name '{:?}'", experiment.name);
    let tests = get_tests(tests, &experiment).context("Failed to get tests for the experiment")?;

    // Load metrics from the store
    mgr.store.load_metrics()?;

    git_driver::commit(&experiment.path, "Running experiment")?;

    let mgr = Arc::new(Mutex::new(mgr));

    for test in &tests {
        info!("Running test: {}", test);
        run_experiment(mgr.clone(), test, &experiment, short_circuit, parallel)?;
    }

    Ok(())
}
