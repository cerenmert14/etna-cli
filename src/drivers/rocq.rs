use std::{collections::HashMap, path::Path, process::Stdio};

use anyhow::Context;
use chrono::Duration;

use process_control::{ChildExt, Control};

use crate::{
    commands::{store, workload},
    drivers::common::{load_workload, Driver, DriverConfig, RunConfig},
    experiment::{ExperimentSnapshot, Test},
    workload::{Step, Workload},
};

struct RocqDriver {}

impl Driver for RocqDriver {
    fn init(&self) {
        // Initialize the Rocq driver
        log::info!("initializing Rocq driver...");
    }

    fn config(&self) -> DriverConfig {
        // Return the configuration for the Rocq driver
        DriverConfig {}
    }
}

pub(crate) fn run_experiment(
    test: &Test,
    experiment_config: &crate::config::ExperimentConfig,
    snapshot: ExperimentSnapshot,
) -> anyhow::Result<()> {
    pub(crate) fn aux(
        test: &Test,
        experiment_config: &crate::config::ExperimentConfig,
        snapshot: ExperimentSnapshot,
    ) -> anyhow::Result<()> {
        // Create a new Rocq driver
        let driver = RocqDriver {};

        // Initialize the driver
        driver.init();

        for variant in test.mutations.iter() {
            marauders::run_set_command(&experiment_config.path, variant)?;
        }

        let workload_dir = experiment_config
            .path
            .join("workloads")
            .join(test.language.as_str())
            .join(test.workload.as_str());

        let workload: Workload = load_workload(
            &experiment_config.path,
            test.language.as_str(),
            test.workload.as_str(),
        )?;

        let params = [
            (
                "workload_path".to_string(),
                workload_dir.display().to_string(),
            ),
            ("workload".to_string(), test.workload.clone()),
            ("language".to_string(), test.language.clone()),
        ];

        let config_path = workload_dir.join("config").with_extension("json");

        let cfg: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap())
                .unwrap();

        let tags: HashMap<String, Vec<String>> =
            serde_json::from_value(cfg.get("tags").unwrap().clone()).unwrap();

        driver.build(
            &workload_dir,
            &workload.check_steps,
            &workload.build_steps,
            &HashMap::from(params),
            &tags,
        )?;

        for (strategy, property) in test.tasks.iter() {
            let params = [
                (
                    "workload_path".to_string(),
                    workload_dir.display().to_string(),
                ),
                ("property".to_string(), property.clone()),
                ("strategy".to_string(), strategy.clone()),
            ];

            // Run the experiment
            let run_config = RunConfig {
                workload_dir: workload_dir.clone(),
                experiment_id: snapshot.experiment.clone(),
                trials: 10,
                workload: test.workload.clone(),
                strategy: strategy.to_string(),
                mutations: test.mutations.clone(),
                property: property.to_string(),
                timeout: 5,
                short_ciruit: false,
                seeds: None,
            };
            driver.run(
                &run_config,
                &workload.run_step,
                &HashMap::from(params),
                &tags,
            )?;
        }

        Ok(())
    }

    let result = aux(test, experiment_config, snapshot);
    if let Err(e) = &result {
        log::error!("Experiment failed with error: {}", e);
    } else {
        log::info!("Experiment completed successfully");
    }

    marauders::run_reset_command(&experiment_config.path)?;
    result
}
