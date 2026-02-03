use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use serde_json::Value;
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
    cli_params: Vec<(String, String)>,
) -> anyhow::Result<()> {
    tracing::trace!("running experiment with name '{:?}'", experiment.name);
    let mut tests =
        get_tests(tests, &experiment).context("Failed to get tests for the experiment")?;

    // Convert CLI params to HashMap
    let cli_params: HashMap<String, String> = cli_params.into_iter().collect();

    // Load metrics from the store
    mgr.store.load_metrics()?;

    git_driver::commit(&experiment.path, "Running experiment")?;

    let mgr = Arc::new(Mutex::new(mgr));

    for test in &mut tests {
        info!("Running test: {}", test);
        for p in cli_params.iter() {
            test.params
                .as_mut()
                .and_then(|params| params.insert(p.0.clone(), Value::String(p.1.clone())));
        }
        run_experiment(
            mgr.clone(),
            test,
            &experiment,
            short_circuit,
            parallel,
            &cli_params,
            None, // No cancel flag for CLI
        )?;
    }

    Ok(())
}
