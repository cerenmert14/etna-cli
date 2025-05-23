use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::Context as _;
use chrono::Duration;
use process_control::{ChildExt as _, Control as _};
use serde_json::Map;

use crate::{
    commands::store,
    workload::{Language, Step, Workload},
};

use anyhow::Context;

use process_control::{ChildExt, Control};

use crate::experiment::{ExperimentSnapshot, Test};

pub(crate) struct RunConfig {
    pub(crate) workload_dir: PathBuf,
    pub(crate) experiment_id: String,
    pub(crate) trials: usize,
    pub(crate) workload: String,
    pub(crate) strategy: String,
    pub(crate) mutations: Vec<String>,
    pub(crate) property: String,
    pub(crate) timeout: usize,
    pub(crate) short_ciruit: bool,
    pub(crate) seeds: Option<Vec<u64>>,
}

pub(crate) fn change_dir(path: &Path, cmd: &dyn Fn() -> anyhow::Result<()>) -> anyhow::Result<()> {
    // Change the current working directory to the specified path
    std::env::set_current_dir(path)
        .context(format!("Failed to change directory to {}", path.display()))?;

    // Execute the command
    let result = cmd().context(format!("Failed to execute command in {}", path.display()));

    if let Err(e) = result {
        log::error!(
            "Command failed with '{}', changing back to parent directory",
            e
        );
    }

    std::env::set_current_dir("..").context("Failed to change directory back to parent")
}

fn load_language(experiment_path: &Path, language: &str) -> anyhow::Result<Language> {
    let language_path = experiment_path.join("workloads").join(language);
    let config_path = language_path.join("config").with_extension("json");

    let language: Language = serde_json::from_str(
        &std::fs::read_to_string(&config_path)
            .with_context(|| format!("could not read config at '{}'", config_path.display()))?,
    )
    .with_context(|| {
        format!(
            "config file at '{}' is not a valid Language",
            config_path.display()
        )
    })?;

    Ok(language)
}

pub(crate) fn get_step(
    config: &serde_json::Value,
    step_index: &str,
    language_steps: &dyn Fn() -> Vec<Step>,
) -> Vec<Step> {
    let step = config.get(step_index);

    if let Some(step) = step {
        let step = serde_json::from_value::<Vec<Step>>(step.clone());
        if let Ok(step) = step {
            return step;
        } else {
            log::error!("Failed to parse step: '{}'", step_index);
        }
    }

    log::warn!("Step '{}' not found, using default step", step_index);
    language_steps()
}

pub(crate) fn load_workload(
    experiment_path: &Path,
    language: &str,
    workload: &str,
) -> anyhow::Result<Workload> {
    let workload_path = experiment_path
        .join("workloads")
        .join(language)
        .join(workload);

    let config_path = workload_path.join("config").with_extension("json");

    let workload_config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();

    let language = load_language(experiment_path, language)?;

    Ok((Workload {
        name: workload.to_string(),
        language: language.name,
        dir: workload_path,
        properties: vec![],
        variations: vec![],
        strategies: vec![],
        build_steps: get_step(&workload_config, "build_steps", &|| {
            language.build_steps.clone()
        }),
        check_steps: get_step(&workload_config, "check_steps", &|| {
            language.check_steps.clone()
        }),
        run_step: get_step(&workload_config, "run_steps", &|| {
            vec![language.run_step.clone()]
        })[0]
            .clone(),
    }))
}

pub(crate) trait Driver {
    fn init(&self);

    fn run(
        &self,
        run_config: &RunConfig,
        run_step: &Step,
        params: &HashMap<String, String>,
        tags: &HashMap<String, Vec<String>>,
    ) -> anyhow::Result<()> {
        let run_steps = run_step.realize(params, tags)?;
        anyhow::ensure!(
            run_steps.len() == 1,
            "Expected exactly one run step, got {}",
            run_steps.len()
        );
        // unwrap here is fine because we just checked the length
        let step = run_steps.get(0).unwrap();

        log::info!("running '{}'", step);

        for i in 0..run_config.trials {
            log::trace!("running trial {}", i);

            let mut params = HashMap::new();
            let step_params = step.params();

            if step_params.contains(&"strategy") {
                params.insert("strategy".to_string(), run_config.strategy.clone());
            }
            if step_params.contains(&"property") {
                params.insert("property".to_string(), run_config.property.clone());
            }
            if step_params.contains(&"workload_path") {
                params.insert(
                    "workload_path".to_string(),
                    run_config.workload_dir.display().to_string(),
                );
            }

            let step = step.decide(&params, tags);

            let mut cmd = std::process::Command::new(&step.command);
            let cmd = step.args.iter().fold(&mut cmd, |cmd, arg| cmd.arg(arg));

            let output = cmd
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .with_context(|| format!("Failed to spawn '{}'", step.command))?
                .controlled_with_output()
                .time_limit(
                    Duration::seconds(run_config.timeout as i64)
                        .to_std()
                        .context("Failed to convert duration")?,
                )
                .terminate_for_timeout()
                .wait()?
                .context("process timed out")?;

            if !output.status.success() {
                anyhow::bail!("Run command failed with status: {}", output.status);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            log::debug!("stdout: {}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::debug!("stderr: {}", stderr);
            // look for the result between [| and |]
            let result = match (stdout.find("[|"), stdout.find("|]")) {
                (Some(start), Some(end)) => &stdout[start + 2..end],
                _ => {
                    log::warn!("No result found in stdout");
                    match (stderr.find("[|"), stderr.find("|]")) {
                        (Some(start), Some(end)) => &stderr[start + 2..end],
                        _ => {
                            log::warn!("No result found in stderr");
                            anyhow::bail!("No result found in stdout or stderr");
                        }
                    }
                }
            };

            log::info!("writing result '{}' to store", result);
            store::write::invoke(run_config.experiment_id.clone(), result.to_string())?;
        }

        Ok(())
    }

    fn build(
        &self,
        build_dir: &Path,
        check_steps: &Vec<Step>,
        build_steps: &Vec<Step>,
        params: &HashMap<String, String>,
        tags: &HashMap<String, Vec<String>>,
    ) -> anyhow::Result<()> {
        change_dir(build_dir, &|| {
            log::info!("running check commands...");
            let check_steps = check_steps
                .iter()
                .map(|step| step.realize(params, tags))
                .collect::<Vec<_>>();

            anyhow::ensure!(check_steps.iter().all(anyhow::Result::is_ok));

            let check_steps = check_steps
                .into_iter()
                .map(anyhow::Result::unwrap)
                .flatten()
                .collect::<Vec<_>>();

            for step in check_steps.iter() {
                log::debug!("running check step: {}", step);
                // Run the check command
                let step = step.decide(params, tags);
                let mut cmd = std::process::Command::new(&step.command);

                let cmd = step.args.iter().fold(&mut cmd, |cmd, arg| cmd.arg(arg));

                let output = cmd.output().context("Failed to execute check command")?;

                if !output.status.success() {
                    log::info!(
                        "[✗] '{}' failed",
                        step.command.clone() + " " + &step.args.join(" ")
                    );
                    anyhow::bail!("check command failed with status: {}", output.status);
                } else {
                    log::info!(
                        "[✓] '{}' passed",
                        step.command.clone() + " " + &step.args.join(" ")
                    );
                }
            }
            log::info!("check commands are successfull.");
            log::info!("running build commands...");

            let build_steps = build_steps
                .iter()
                .map(|step| step.realize(params, tags))
                .collect::<Vec<_>>();

            anyhow::ensure!(build_steps.iter().all(anyhow::Result::is_ok));

            let build_steps = build_steps
                .into_iter()
                .map(anyhow::Result::unwrap)
                .flatten()
                .collect::<Vec<_>>();

            for step in build_steps.iter() {
                // Run the build command
                log::debug!("running build step: {}", step);
                let step = step.decide(params, tags);
                log::debug!("step is evaluated to '{step}'");
                let mut cmd = std::process::Command::new(&step.command);
                let cmd = step.args.iter().fold(&mut cmd, |cmd, arg| cmd.arg(arg));
                let output = cmd.output().context("Failed to execute build command")?;

                if !output.status.success() {
                    log::info!("[✗] '{}' failed", step.command);
                    anyhow::bail!("build command failed with status: {}", output.status);
                } else {
                    log::info!("[✓] '{}' passed", step.command);
                }
            }
            log::info!("build commands are successfull.");

            Ok(())
        })
    }

    fn run_experiment(
        &self,
        test: &Test,
        experiment_config: &crate::config::ExperimentConfig,
        snapshot: ExperimentSnapshot,
    ) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        pub(crate) fn aux(
            driver: &dyn Driver,
            test: &Test,
            experiment_config: &crate::config::ExperimentConfig,
            snapshot: ExperimentSnapshot,
        ) -> anyhow::Result<()> {
            let lang = marauders::Language::name_to_language(&test.language, &vec![])
                .with_context(|| {
                    format!("language '{}' is not known or supported", test.language)
                })?;
            let glob = format!("*.{}", lang.file_extension());

            for variant in test.mutations.iter() {
                marauders::run_set_command(&experiment_config.path, variant, Some(glob.as_str()))?;
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

            // todo: there's a bug when two params share a prefix, fix it.
            let params = [(
                "workload_path".to_string(),
                workload_dir.display().to_string(),
            )];

            let config_path = workload_dir.join("config").with_extension("json");

            let cfg: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();

            let tags: HashMap<String, Vec<String>> = serde_json::from_value(
                cfg.get("tags")
                    .unwrap_or(&serde_json::Value::Object(Map::new()))
                    .clone(),
            )
            .unwrap();

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
                    trials: test.trials,
                    workload: test.workload.clone(),
                    strategy: strategy.to_string(),
                    mutations: test.mutations.clone(),
                    property: property.to_string(),
                    timeout: test.timeout,
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

        let result = aux(self, test, experiment_config, snapshot);
        if let Err(e) = &result {
            log::error!("Experiment failed with error: {}", e);
        } else {
            log::info!("Experiment completed successfully");
        }

        marauders::run_reset_command(&experiment_config.path)?;
        result
    }
}

pub struct DefaultDriver;
impl Driver for DefaultDriver {
    fn init(&self) {
        log::info!("Initialized default ETNA driver.");
    }
}
