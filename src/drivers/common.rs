use std::{
    collections::HashMap,
    path::{Path, PathBuf}, process::Stdio,
};

use anyhow::Context as _;
use chrono::Duration;
use process_control::{ChildExt as _, Control as _};
use tabled::grid::config;

use crate::{commands::store, workload::{Language, Step, Workload}};

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

pub(crate) struct DriverConfig {}

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
    let language: Language =
        serde_json::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();

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
    ) -> anyhow::Result<()> {
        for i in 0..run_config.trials {
            log::debug!("running trial {}", i);

            // Run the Racket command
            let step = run_step.realize(params)?;

            log::debug!(
                "running {}",
                step.command.clone() + " " + &step.args.join(" ")
            );

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

            log::debug!("result: {}", result);
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
    ) -> anyhow::Result<()> {
        change_dir(build_dir, &|| {
            println!("running check commands...");
            for step in check_steps.iter() {
                println!("running check command: {}", step.command);
                println!("with args: {:?}", step.args);
                println!("with params: {:?}", step.params);
                let step = step.realize(params)?;
                log::debug!(
                    "running check step: {}",
                    step.command.clone() + " " + &step.args.join(" ")
                );
                // Run the check command
                let mut cmd = std::process::Command::new(&step.command);

                let cmd = step.args.iter().fold(&mut cmd, |cmd, arg| cmd.arg(arg));

                let output = cmd.output().context("Failed to execute check command")?;

                if !output.status.success() {
                    println!("[✗] '{}' failed", step.command.clone() + " " + &step.args.join(" "));
                    anyhow::bail!("check command failed with status: {}", output.status);
                } else {
                    println!("[✓] '{}' passed", step.command.clone() + " " + &step.args.join(" "));
                }
            }
            println!("check commands are successfull.");
            println!("running build commands...");
            for step in build_steps.iter() {
                let step = step.realize(params)?;
                log::debug!(
                    "running build step: {}",
                    step.command.clone() + " " + &step.args.join(" ")
                );
                // Run the build command
                let mut cmd = std::process::Command::new(&step.command);
                let cmd = step.args.iter().fold(&mut cmd, |cmd, arg| cmd.arg(arg));
                let output = cmd.output().context("Failed to execute build command")?;

                if !output.status.success() {
                    println!("[✗] '{}' failed", step.command);
                    anyhow::bail!("build command failed with status: {}", output.status);
                } else {
                    println!("[✓] '{}' passed", step.command);
                }
            }
            println!("build commands are successfull.");

            Ok(())
        })
    }

    fn config(&self) -> DriverConfig;
}
