use std::{
    collections::HashMap,
    io::Write as _,
    path::{Path, PathBuf},
    process::Stdio,
    time::SystemTime,
};

use serde_json::{Map, Value};
use std::time::Duration;

use crate::{
    commands::store,
    config::ExperimentConfig,
    store::{Metric, Store},
    workload::{Command, Language, Step, Workload},
};

use anyhow::Context;

use process_control::{ChildExt, Control};

use crate::experiment::{ExperimentSnapshot, Test};

pub(crate) struct RunConfig {
    pub(crate) language: String,
    pub(crate) workload_dir: PathBuf,
    pub(crate) experiment_id: String,
    pub(crate) trials: usize,
    pub(crate) workload: String,
    pub(crate) strategy: String,
    pub(crate) mutations: Vec<String>,
    pub(crate) property: String,
    pub(crate) timeout: f64,
    pub(crate) short_circuit: bool,
    pub(crate) cross: bool,
    pub(crate) seeds: Option<Vec<u64>>,
}

fn load_language(experiment_path: &Path, language: &str) -> anyhow::Result<Language> {
    let language_path = experiment_path.join("workloads").join(language);
    let config_path = language_path.join("config").with_extension("json");

    let mut config: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&config_path)
            .with_context(|| format!("could not read config at '{}'", config_path.display()))?,
    )
    .with_context(|| {
        format!(
            "config file at '{}' is not a valid Language",
            config_path.display()
        )
    })?;

    config
        .as_object_mut()
        .context("config file is not a valid JSON object")?
        .insert(
            "name".to_string(),
            serde_json::Value::String(language.to_string()),
        );

    let language: Language =
        serde_json::from_value(config).context("config file is not a valid Language")?;

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

    log::debug!("Step '{}' not found, using default step", step_index);
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

    anyhow::ensure!(
        workload_path.exists(),
        "Workload directory not found at '{}'",
        workload_path.display()
    );

    let config_path = workload_path.join("config").with_extension("json");

    anyhow::ensure!(
        config_path.exists(),
        "Config file not found at '{}'",
        config_path.display()
    );

    let workload_config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();

    let language = load_language(experiment_path, language)?;

    Ok(Workload {
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
    })
}

fn metric_matches<'a>(
    m: &'a Metric,
    language: Option<&str>,
    workload: Option<&str>,
    mutations: Option<&Vec<String>>,
    strategy: Option<&str>,
    property: Option<&str>,
    timeout: Option<f64>,
    trial: Option<usize>,
    cross: Option<bool>,
) -> Option<&'a Metric> {
    let language_match = language.map_or(true, |l| {
        m.data
            .get("language")
            .map_or(false, |v| v.as_str() == Some(l))
    });
    let workload_match = workload.map_or(true, |w| {
        m.data
            .get("workload")
            .map_or(false, |v| v.as_str() == Some(w))
    });
    let mutations_match = mutations.map_or(true, |muts| {
        m.data.get("mutations").map_or(false, |v| {
            v.as_array().map_or(false, |arr| {
                arr.iter()
                    .all(|mv| muts.contains(&mv.as_str().unwrap().to_string()))
            })
        })
    });
    let strategy_match = strategy.map_or(true, |s| {
        m.data
            .get("strategy")
            .map_or(false, |v| v.as_str() == Some(s))
    });
    let property_match = property.map_or(true, |p| {
        m.data
            .get("property")
            .map_or(false, |v| v.as_str() == Some(p))
    });
    let timeout_match = timeout.map_or(true, |t| {
        m.data
            .get("timeout")
            .and_then(|v| v.as_f64())
            .map_or(false, |v| v >= t)
    });
    let trial_match = trial.map_or(true, |t| {
        m.data
            .get("trial")
            .and_then(|v| v.as_u64())
            .map_or(false, |v| v as usize == t)
    });
    let cross_match = cross.map_or(true, |c| {
        m.data
            .get("cross")
            .and_then(|v| v.as_bool())
            .map_or(false, |v| v == c)
    });

    if language_match
        && workload_match
        && mutations_match
        && strategy_match
        && property_match
        && timeout_match
        && trial_match
        && cross_match
    {
        Some(m)
    } else {
        None
    }
}

fn task_completed(
    language: &str,
    workload: &str,
    mutations: &Vec<String>,
    strategy: &str,
    property: &str,
    timeout: f64,
    trials: usize,
    short_circuit: bool,
    cross: bool,
    metrics: &Vec<Metric>,
) -> bool {
    log::trace!(
        "Checking if task is completed for language '{}', workload '{}', strategy '{}', property '{}', mutations '{:?}'",
        language, workload, strategy, property, mutations
    );
    let filtered_metrics = metrics
        .iter()
        .filter(|m| {
            metric_matches(
                m,
                Some(language),
                Some(workload),
                Some(mutations),
                Some(strategy),
                Some(property),
                None,
                None,
                Some(cross),
            )
            .is_some()
        })
        .collect::<Vec<_>>();

    let mut timed_out = false;
    (0..trials as u64).into_iter().all(|i| {
        filtered_metrics
            .iter()
            .find(|m| m.data.get("trial").and_then(|v| v.as_u64()).map(|u| u == i) == Some(true))
            .and_then(|m| {
                log::trace!("Checking metric: {:?} for trial {}", m.data, i);
                if short_circuit {
                    if timed_out {
                        return Some(true);
                    }
                    if m.data
                        .get("result")
                        .map_or(false, |v| v.as_str() == Some("timed_out"))
                    {
                        timed_out = true;
                    }
                }
                m.data
                    .get("timeout")
                    .and_then(|v| v.as_f64())
                    .map(|t| t >= timeout)
            })
            == Some(true)
            || timed_out
    })
}

pub(crate) fn run(
    experiment_config: &ExperimentConfig,
    run_config: &RunConfig,
    run_step: &Step,
    params: &HashMap<String, String>,
    tags: &HashMap<String, Vec<String>>,
    metrics: &Vec<Metric>,
) -> anyhow::Result<()> {
    let run_steps = run_step.realize(params, tags)?;
    anyhow::ensure!(
        run_steps.len() == 1,
        "Expected exactly one run step, got {}",
        run_steps.len()
    );
    // unwrap here is fine because we just checked the length
    let step = run_steps.first().unwrap();

    log::info!("running the trials via '{}'", step);

    for i in 0..run_config.trials {
        log::trace!("running trial {}", i);

        let previous_metric = metrics.iter().find(|m| {
            metric_matches(
                m,
                Some(&run_config.language),
                Some(&run_config.workload),
                Some(&run_config.mutations),
                Some(&run_config.strategy),
                Some(&run_config.property),
                Some(run_config.timeout),
                Some(i),
                Some(run_config.cross),
            )
            .is_some()
        });

        if let Some(metric) = previous_metric {
            // If the metric is a timeout and short-circuit is enabled, break the loop
            if run_config.short_circuit
                && metric
                    .data
                    .get("result")
                    .map_or(false, |v| v.as_str() == Some("timed_out"))
            {
                log::info!("Short-circuiting the experiment due to previous timeout");
                break;
            }

            // Skip the trial because it has already been run
            log::info!(
                    "Skipping trial {} for language '{}', workload '{}', strategy '{}', property '{}' because it has already been run",
                    i,
                    run_config.language,
                    run_config.workload,
                    run_config.strategy,
                    run_config.property
                );
            continue;
        }

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
        params.insert("cross".to_string(), run_config.cross.to_string());
        let old_step = step;
        let step = old_step.decide(&params, tags);
        log::trace!("step '{old_step}' is evaluated to '{step}' with params: {params:?}");

        let cdir = PathBuf::from(step.run_at.unwrap_or(".".to_string()));

        let mut cmd = std::process::Command::new(&step.command);
        cmd.args(&step.args);

        let result = serde_json::json!({
            "language": run_config.language,
            "workload": run_config.workload,
            "experiment": run_config.experiment_id,
            "strategy": run_config.strategy,
            "property": run_config.property,
            "mutations": run_config.mutations,
            "trial": i,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "cross": run_config.cross,
            "timeout": run_config.timeout,
        });

        let result = if run_config.cross {
            run_cross(cdir, result, cmd, &step.command, run_config)?
        } else {
            run_default(cdir, result, cmd, &step.command, run_config)?
        };

        store::write::invoke(
            Some(experiment_config),
            run_config.experiment_id.clone(),
            result.to_string(),
        )?;

        if result.get("result").is_some_and(|v| v == "timed_out") {
            if run_config.short_circuit {
                log::info!("Short-circuiting the experiment due to timeout");
                return Ok(());
            } else {
                log::info!("Process timed out, but short-circuit is not enabled, so continuing with the next trial");
            }
        }
    }

    Ok(())
}

/// Runs the canonical serialized runner (Rust for now) for the given workload, mutation, property, and tests.
/// Report the index of the failing test if any.
fn run_canonical_serialized(
    workload: &str,
    mutations: &[String],
    property: &str,
    tests: &str,
) -> anyhow::Result<Option<usize>> {
    // Change the current working directory to the workload directory
    let workload_dir = PathBuf::from(
        std::env::var("ETNA_DIR").context("ETNA_DIR environment variable is not set")?,
    )
    .join("workloads")
    .join("Rust")
    .join(workload);

    // Run marauders to mutate the canonical serializer
    log::trace!(
        "Running marauders to mutate the canonical serializer for workload '{}', mutations '{:?}'",
        workload,
        mutations
    );
    marauders::run_reset_command(&workload_dir)?;

    let glob = format!("*.{}", marauders::Language::Rust.file_extension());

    for variant in mutations.iter() {
        log::trace!(
            "Running marauders to mutate the canonical serializer for workload '{}', variant '{}'",
            workload,
            variant
        );
        marauders::run_set_command(&workload_dir, variant, Some(glob.as_str()))?;
    }

    // Run the build command for the canonical serializer
    let mut cmd = std::process::Command::new("cargo");
    cmd.current_dir(&workload_dir);
    cmd.args(&["build", "--release"]);
    log::debug!(
        "running 'cargo build --release' in '{}'",
        workload_dir.display()
    );

    let output = cmd.output();
    match output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to build canonical serializer: '{}'", stderr);
            }
        }
        Err(e) => {
            anyhow::bail!("Failed to run 'cargo build --release': '{}'", e);
        }
    }

    // Run the canonical serializer
    log::debug!(
        "Running canonical serializer for workload '{}', mutations '{:?}', property '{}'",
        workload,
        mutations,
        property
    );
    let mut cmd = std::process::Command::new(
        PathBuf::from(&workload_dir)
            .join("target")
            .join("release")
            .join(format!("{}-serialized", workload.to_lowercase())),
    );
    cmd.args(&[tests, property]);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    log::trace!("Running canonical serializer command: {:?}", cmd);
    let output = cmd.output();
    match output {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Parse the stderr to find the index of the failing test
            // The JSON output starts at [| and ends at |]
            log::trace!("Canonical serializer output: {}", stderr);

            let json_value: serde_json::Value =
                serde_json::from_str(&stderr).context("Failed to parse JSON output")?;

            let index = json_value.get("test").and_then(|v| v.as_i64());

            if let Some(index) = index {
                log::info!(
                    "Canonical serializer found a failing test at index {}",
                    index
                );
                Ok(Some(index as usize))
            } else {
                log::info!("Canonical serializer failed while running, no failing test found");
                log::debug!("Canonical serializer output: {}", stderr);
                Ok(None)
            }
        }
        Err(e) => {
            log::error!("Failed to run canonical serializer: {}", e);
            anyhow::bail!("Failed to run canonical serializer: '{}'", e);
        }
    }
}

/// Cross-language runner for comparing across different languages.
/// Logically runs the following;
/// 1. Runs `workloads/language/workload-sampler` for the given strategy and property.
/// 2. Parses the output of the sampler to get the samples along with their durations.
/// 3. Writes the samples to a temporary file.
/// 4. Mutates and builds `workloads/Rust/workload-serialized` for running the serializer.
/// 5. Runs `workloads/Rust/workload-serialized` with the temporary file and the property as arguments.
/// 6. Reads the output of the serializer from stderrr and puts it in the result store.
///
/// The function is designed to run in a loop until the timeout is reached, with batches of 1000 samples
/// collected in each iteration.
fn run_cross(
    cdir: PathBuf,
    mut result: serde_json::Value,
    mut cmd: std::process::Command,
    step: &str,
    run_config: &RunConfig,
) -> anyhow::Result<serde_json::Value> {
    let t = SystemTime::now();
    let timeout = Duration::from_secs_f64(run_config.timeout);

    let mut total_time = Duration::default();
    let mut total_samples = 0;

    while t.elapsed().unwrap() < timeout {
        // sample the command
        log::debug!("sampling command: {}", step);
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&cdir)
            .spawn()
            .with_context(|| format!("Failed to spawn '{}'", step))?
            .wait_with_output()
            .with_context(|| format!("Failed to run command '{}'", step));

        match child {
            Ok(output) => {
                let status = output.status;
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::debug!("stdout: {}", &stdout[..100.min(stdout.len())]);
                log::debug!("stderr: {}", &stderr[..100.min(stderr.len())]);
                if !status.success() {
                    log::error!("Command '{}' failed with status: {}", step, status);
                    panic!("Command should not have a non-zero exit status when running in cross-language mode");
                }

                let samples: Vec<serde_json::Value> = serde_json::from_str(&stdout)
                    .context(format!("Failed to parse output of command '{}'", step))?;

                let (durations, samples) = samples
                    .into_iter()
                    .map(|s| {
                        let duration = s.get("time").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let sample = s.get("value").and_then(|v| v.as_str()).unwrap_or("unknown");
                        (duration.to_string(), sample.to_string())
                    })
                    .unzip::<String, String, Vec<_>, Vec<_>>();

                log::debug!("{} samples collected", samples.len());

                // write samples to a temporary file
                let mut temp_file =
                    tempfile::NamedTempFile::new().context("Failed to create temporary file")?;
                temp_file
                    .write_all(format!("({})", samples.join(" ")).as_bytes())
                    .context("Failed to write samples to temporary file")?;

                // Call the Rust serializer for the specific workload
                log::debug!(
                    "Running canonical serializer for workload '{}', mutations '{:?}', property '{}'",
                    run_config.workload,
                    run_config.mutations,
                    run_config.property
                );
                log::debug!("Using temporary file: {}", temp_file.path().display());
                let results = run_canonical_serialized(
                    &run_config.workload,
                    &run_config.mutations,
                    &run_config.property,
                    temp_file.path().to_str().unwrap(),
                )
                .context("Failed to run canonical serialized runner")?;

                let time_cutoff = results.unwrap_or(durations.len());

                for d in durations.iter().take(time_cutoff) {
                    let d = parse_duration::parse(d)
                        .with_context(|| format!("Failed to parse duration: {}", d))?;
                    total_time += d;
                }
                total_samples += time_cutoff;
                
                log::debug!(
                    "Total time for this batch: {:?}, total samples: {}",
                    total_time,
                    time_cutoff
                );
                if results.is_some() {
                    let index = results.unwrap();
                    log::info!(
                        "Canonical serializer found a failing test at index {}",
                        index
                    );
                    result.as_object_mut().unwrap().insert(
                        "result".to_owned(),
                        serde_json::Value::String("foundbug".to_owned()),
                    );
                    result
                        .as_object_mut()
                        .unwrap()
                        .insert("test".to_owned(), serde_json::Value::Number(total_samples.into()));
                    result.as_object_mut().unwrap().insert(
                        "generation_time".to_owned(),
                        serde_json::Value::String(format!("{:?}", total_time)),
                    );
                    result.as_object_mut().unwrap().insert(
                        "counterexample".to_owned(),
                        serde_json::Value::String(
                            samples
                                .get(index)
                                .cloned()
                                .unwrap_or_else(|| "No counterexample found".to_string()),
                        ),
                    );
                    return Ok(result);
                } else {
                    log::info!("Canonical serializer did not find any failing test");
                }
            }
            Err(err) => {
                log::error!("Failed to spawn child process: {}", err);
                result.as_object_mut().unwrap().insert(
                    "result".to_owned(),
                    serde_json::Value::String("aborted".to_owned()),
                );
                return Ok(result);
            }
        }
    }

    log::info!(
        "Cross-language run completed in {:?} with {} samples",
        total_time,
        total_samples
    );
    result.as_object_mut().unwrap().insert(
        "result".to_owned(),
        serde_json::Value::String("timed out".to_owned()),
    );
    result.as_object_mut().unwrap().insert(
        "generation_time".to_owned(),
        serde_json::Value::String(format!("{:?}", total_time)),
    );
    result.as_object_mut().unwrap().insert(
        "samples".to_owned(),
        serde_json::Value::Number(total_samples.into()),
    );
    result.as_object_mut().unwrap().insert(
        "counterexample".to_owned(),
        serde_json::Value::Null,
    );

    Ok(result)
}

fn run_default(
    cdir: PathBuf,
    mut result: serde_json::Value,
    mut cmd: std::process::Command,
    command: &str,
    run_config: &RunConfig,
) -> anyhow::Result<serde_json::Value> {
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&cdir)
        .spawn()
        .with_context(|| format!("Failed to spawn '{}'", command))?
        .controlled_with_output()
        .time_limit(Duration::from_secs_f64(run_config.timeout))
        .terminate_for_timeout()
        .wait()
        .context(format!("Failed to run command '{}'", command));

    log::trace!("metadata: {}", result);

    match output {
        Ok(None) => {
            log::warn!("Process timed out after {} seconds", run_config.timeout);

            result
                .as_object_mut()
                .unwrap()
                .insert("result".to_owned(), Value::String("timed_out".to_owned()));
        }
        Ok(Some(output)) => {
            if !output.status.success() {
                anyhow::bail!("Run command failed with status: {}", output.status);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            log::debug!("stdout: {}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::debug!("stderr: {}", stderr);
            // look for the result between [| and |]
            let metric = match (stdout.find("[|"), stdout.find("|]")) {
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
            let mut metric = serde_json::from_str::<serde_json::Value>(metric)
                .with_context(|| format!("Failed to parse result: '{}'", metric))?;

            log::trace!("result: {}", metric);

            result
                .as_object_mut()
                .context("the printed metric is not a valid json object")?
                .extend(metric.as_object().unwrap().clone());
        }
        Err(err) => {
            log::error!("Aborting! Failed to run command '{}': {}", command, err);

            result
                .as_object_mut()
                .unwrap()
                .insert("result".to_owned(), Value::String("aborted".to_owned()));
        }
    }

    Ok(result)
}

pub(crate) fn build(
    build_dir: &Path,
    check_steps: &[Step],
    build_steps: &[Step],
    params: &HashMap<String, String>,
    tags: &HashMap<String, Vec<String>>,
) -> anyhow::Result<()> {
    log::info!("running check commands...");
    let check_steps = check_steps
        .iter()
        .map(|step| step.realize(params, tags))
        .collect::<Vec<_>>();

    anyhow::ensure!(check_steps.iter().all(anyhow::Result::is_ok));

    let check_steps = check_steps
        .into_iter()
        .flat_map(anyhow::Result::unwrap)
        .collect::<Vec<_>>();

    for step in check_steps.iter() {
        log::debug!("running check step: {}", step);
        // Run the check command
        let step = step.decide(params, tags);
        log::debug!("step is evaluated to '{step}'");

        let mut cmd = std::process::Command::new(&step.command);
        cmd.current_dir(build_dir).args(&step.args);

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
        .flat_map(anyhow::Result::unwrap)
        .collect::<Vec<_>>();

    for step in build_steps.iter() {
        // Run the build command
        log::debug!("running build step: {}", step);
        let step = step.decide(params, tags);
        log::debug!("step is evaluated to '{step}'");

        let mut cmd = std::process::Command::new(&step.command);
        cmd.current_dir(build_dir).args(&step.args);

        let output = cmd.output().context("Failed to execute build command")?;

        if !output.status.success() {
            log::info!("[✗] '{}' failed", step);
            anyhow::bail!("build command failed with status: {}", output.status);
        } else {
            log::info!("[✓] '{}' passed", step);
        }
    }
    log::info!("build commands are successfull.");

    Ok(())
}

pub(crate) fn run_experiment(
    test: &Test,
    experiment_config: &crate::config::ExperimentConfig,
    snapshot: ExperimentSnapshot,
    short_circuit: bool,
    cross: bool,
) -> anyhow::Result<()> {
    pub(crate) fn aux(
        test: &Test,
        experiment_config: &crate::config::ExperimentConfig,
        snapshot: ExperimentSnapshot,
        short_circuit: bool,
        cross: bool,
    ) -> anyhow::Result<()> {
        let lang = marauders::Language::name_to_language(&test.language, &vec![])
            .with_context(|| format!("language '{}' is not known or supported", test.language))?;
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

        let store = Store::load(&experiment_config.store)?;

        log::trace!(
            "Checking if all tasks for language '{}' and workload '{}' are already completed",
            test.language,
            test.workload
        );
        let all_tasks_completed = test.tasks.iter().all(|(strategy, property)| {
            task_completed(
                &test.language,
                &test.workload,
                &test.mutations,
                strategy,
                property,
                test.timeout,
                test.trials,
                short_circuit,
                cross,
                &store.metrics,
            )
        });

        if all_tasks_completed {
            log::info!(
                    "All tasks for the current test with language '{}', workload '{}' and mutations '{:?}' are already completed, skipping the build and run steps.",
                    test.language,
                    test.workload,
                    test.mutations
                );
            return Ok(());
        }

        build(
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
                language: test.language.clone(),
                workload_dir: workload_dir.clone(),
                experiment_id: snapshot.experiment.clone(),
                trials: test.trials,
                workload: test.workload.clone(),
                strategy: strategy.to_string(),
                mutations: test.mutations.clone(),
                property: property.to_string(),
                timeout: test.timeout,
                short_circuit,
                cross,
                seeds: None,
            };
            let result = run(
                experiment_config,
                &run_config,
                &workload.run_step,
                &HashMap::from(params),
                &tags,
                // todo: we already filter the metrics in the task_completed function, so we should not pass the whole metrics here but the filtered ones.
                &store.metrics,
            );

            if let Err(e) = &result {
                log::error!("Failed to run experiment: {}", e);
            }
        }

        Ok(())
    }

    let result = aux(test, experiment_config, snapshot, short_circuit, cross);
    if let Err(e) = &result {
        log::error!("Experiment failed with error: {}", e);
    } else {
        log::info!("Experiment completed successfully");
    }

    marauders::run_reset_command(&experiment_config.path)?;
    result
}
