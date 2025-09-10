use std::{
    collections::HashMap,
    io::Write as _,
    path::{Path, PathBuf},
    process::Stdio,
};

use serde_json::{Map, Value};
use std::time::Duration;

use crate::{
    commands::store,
    config::ExperimentConfig,
    store::{Metric, Store},
    workload::{Command, Language, Step, Steps, Workload},
};

use anyhow::Context;

use process_control::{ChildExt, Control};

use crate::experiment::{ExperimentSnapshot, Test};

#[derive(Debug)]
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
    #[allow(dead_code)]
    pub(crate) seeds: Option<Vec<u64>>,
}

fn load_language(experiment_path: &Path, language: &str) -> anyhow::Result<Language> {
    let language_path = experiment_path.join("workloads").join(language);
    let config_path = language_path.join("config").with_extension("json");
    log::debug!("Loading language config from '{}'", config_path.display());
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

    log::debug!("Loaded language config: {:?}", language);
    log::debug!("config: {}", config);
    let language: Language =
        serde_json::from_value(config).context("config file is not a valid Language")?;

    Ok(language)
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

    let workload_config: Option<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(config_path).unwrap()).ok();

    let language = load_language(experiment_path, language)?;

    let steps = if let Some(workload_config) = workload_config {
        Steps::with_default(&workload_config, &language.steps)
    } else {
        language.steps.clone()
    };

    Ok(Workload {
        name: workload.to_string(),
        language: language.name,
        dir: workload_path,
        properties: vec![],
        variations: vec![],
        strategies: vec![],
        steps,
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
    let language_match = language.is_none_or(|l| {
        m.data
            .get("language")
            .is_some_and(|v| v.as_str() == Some(l))
    });
    let workload_match = workload.is_none_or(|w| {
        m.data
            .get("workload")
            .is_some_and(|v| v.as_str() == Some(w))
    });
    let mutations_match = mutations.is_none_or(|muts| {
        m.data.get("mutations").is_some_and(|v| {
            v.as_array().is_some_and(|arr| {
                arr.iter()
                    .all(|mv| muts.contains(&mv.as_str().unwrap().to_string()))
            })
        })
    });
    let strategy_match = strategy.is_none_or(|s| {
        m.data
            .get("strategy")
            .is_some_and(|v| v.as_str() == Some(s))
    });
    let property_match = property.is_none_or(|p| {
        m.data
            .get("property")
            .is_some_and(|v| v.as_str() == Some(p))
    });
    let timeout_match = timeout.is_none_or(|t| {
        m.data
            .get("timeout")
            .and_then(|v| v.as_f64())
            .is_some_and(|v| v >= t)
    });
    let trial_match = trial.is_none_or(|t| {
        m.data
            .get("trial")
            .and_then(|v| v.as_u64())
            .is_some_and(|v| v as usize == t)
    });
    let cross_match =
        cross.is_none_or(|c| m.data.get("cross").and_then(|v| v.as_bool()) == Some(c));

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
    (0..trials as u64).all(|i| {
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
                        .is_some_and(|v| v.as_str() == Some("timed_out"))
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
    params: &mut HashMap<String, String>,
    tags: &HashMap<String, Vec<String>>,
    metrics: &Vec<Metric>,
) -> anyhow::Result<()> {
    log::trace!("Running with config: {:?}", run_config);
    log::trace!("Run step: {:?}", run_step);

    params.insert("language".to_string(), run_config.language.clone());
    params.insert("workload".to_string(), run_config.workload.clone());
    params.insert("strategy".to_string(), run_config.strategy.clone());
    params.insert("property".to_string(), run_config.property.clone());
    params.insert(
        "workload_path".to_string(),
        run_config.workload_dir.display().to_string(),
    );
    params.insert("cross".to_string(), run_config.cross.to_string());
    params.insert("timeout".to_string(), run_config.timeout.to_string());
    params.insert("mutations".to_string(), run_config.mutations.join(","));
    params.insert("language".to_string(), run_config.language.clone());
    params.insert(
        "experiment_id".to_string(),
        run_config.experiment_id.clone(),
    );

    log::trace!("Final params for step: {:?}", params);

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
                    .is_some_and(|v| v.as_str() == Some("timed_out"))
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

        let old_step = step;
        let step = old_step.decide(params, tags);
        log::trace!("step '{old_step}' is evaluated to '{step}' with params: {params:?}");

        let cmd = std::process::Command::from(&step);

        println!("running trial with '{}'", step);

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
            log::debug!("Running cross-language command: {}", step);
            run_cross(result, cmd, &step, run_config)?
        } else {
            log::debug!("Running default command: {}", step);
            run_default(result, cmd, &step, run_config)?
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
) -> anyhow::Result<serde_json::Value> {
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
    cmd.args(["build", "--release"]);
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
    cmd.args([tests, property]);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    log::trace!("Running canonical serializer command: {:?}", cmd);
    let output = cmd.output();
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse the stdout to find the index of the failing test
            // The JSON output starts at [| and ends at |]
            log::trace!("Canonical serializer output: {}", stdout);

            let json_value: serde_json::Value =
                serde_json::from_str(&stdout).context("Failed to parse JSON output")?;

            Ok(json_value)
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
    mut result: serde_json::Value,
    mut cmd: std::process::Command,
    step: &Command,
    run_config: &RunConfig,
) -> anyhow::Result<serde_json::Value> {
    let timeout = Duration::from_secs_f64(run_config.timeout);

    let mut total_time = Duration::default();
    let mut total_passed = 0;
    let mut total_discards = 0;
    let mut total_samples = 0;

    while total_time < timeout {
        // sample the command
        log::debug!("sampling command: {}", step);
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                log::error!("Failed to spawn command '{}': {}", step, e);
                e
            })
            .with_context(|| format!("Failed to spawn '{}'", step))?
            .wait_with_output()
            .map_err(|e| {
                log::error!("Failed to run command '{}': {}", step, e);
                e
            })
            .with_context(|| format!("Failed to run command '{}'", step));

        match child {
            Ok(output) => {
                let status = output.status;
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::debug!("stdout: {}", &stdout);
                log::debug!("stderr: {}", &stderr);
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
                );

                let Ok(results) = results else {
                    log::error!("Failed to run canonical serializer");
                    log::error!("Results: {:?}", results);
                    result.as_object_mut().unwrap().insert(
                        "result".to_owned(),
                        serde_json::Value::String("aborted".to_owned()),
                    );
                    result.as_object_mut().unwrap().insert(
                        "error".to_owned(),
                        serde_json::Value::String(results.unwrap_err().to_string()),
                    );
                    return Ok(result);
                };

                let status = results
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let passed = results.get("tests").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                let discarded = results
                    .get("discarded")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;

                let time_cutoff = if status == "FoundBug" {
                    passed + discarded + 1
                } else {
                    passed + discarded
                };

                for (i, d) in durations.iter().take(time_cutoff).enumerate() {
                    let d = parse_duration::parse(d)
                        .with_context(|| format!("Failed to parse duration: {}", d))?;

                    total_time += d;

                    if total_time > timeout {
                        // these are approximate values, we divide by the time_cutoff to get the average
                        total_passed += passed * i / time_cutoff;
                        total_discards += discarded * i / time_cutoff;
                        total_samples += i;
                        log::info!("Timeout reached after {:?}, stopping the run", total_time);
                        break;
                    }
                }

                log::debug!(
                    "Total time for this batch: {:?}, total samples: {}",
                    total_time,
                    time_cutoff
                );
                if status == "FoundBug" {
                    log::info!("Found a bug in the canonical serializer, stopping the run");

                    result.as_object_mut().unwrap().insert(
                        "result".to_owned(),
                        serde_json::Value::String("foundbug".to_owned()),
                    );
                    result.as_object_mut().unwrap().insert(
                        "passed".to_owned(),
                        serde_json::Value::Number(total_passed.into()),
                    );
                    result.as_object_mut().unwrap().insert(
                        "discarded".to_owned(),
                        serde_json::Value::Number(total_discards.into()),
                    );
                    result.as_object_mut().unwrap().insert(
                        "tests".to_owned(),
                        serde_json::Value::Number((total_discards + total_passed + 1).into()),
                    );
                    result.as_object_mut().unwrap().insert(
                        "time".to_owned(),
                        serde_json::Value::String(format!("{}ns", total_time.as_nanos())),
                    );
                    result.as_object_mut().unwrap().insert(
                        "counterexample".to_owned(),
                        results
                            .get("counterexample")
                            .unwrap_or(&serde_json::Value::Null)
                            .clone(),
                    );
                    return Ok(result);
                } else {
                    log::info!(
                        "No bugs found in this batch, continuing to the next batch if time allows"
                    );
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
        "time".to_owned(),
        serde_json::Value::String(format!("{}ns", total_time.as_nanos())),
    );
    result.as_object_mut().unwrap().insert(
        "samples".to_owned(),
        serde_json::Value::Number(total_samples.into()),
    );
    result
        .as_object_mut()
        .unwrap()
        .insert("counterexample".to_owned(), serde_json::Value::Null);

    Ok(result)
}

fn run_default(
    mut result: serde_json::Value,
    mut cmd: std::process::Command,
    step: &Command,
    run_config: &RunConfig,
) -> anyhow::Result<serde_json::Value> {
    log::debug!("Running command: {}", step);

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn '{}'", step))?
        .controlled_with_output()
        .time_limit(Duration::from_secs_f64(run_config.timeout))
        .terminate_for_timeout()
        .wait()
        .context(format!("Failed to run command '{}'", step));

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
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::debug!("stdout: {}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::debug!("stderr: {}", stderr);

            if !output.status.success() {
                log::warn!("Command '{}' failed with status: {}", step, output.status);
            }

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
            let metric = serde_json::from_str::<serde_json::Value>(metric)
                .with_context(|| format!("Failed to parse result: '{}'", metric))?;

            log::trace!("result: {}", metric);

            result
                .as_object_mut()
                .context("the printed metric is not a valid json object")?
                .extend(metric.as_object().unwrap().clone());
        }
        Err(err) => {
            log::error!("Aborting! Failed to run command '{}': {}", step, err);

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

        let mut cmd = std::process::Command::from(&step);
        cmd.current_dir(build_dir);

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

        let mut cmd = std::process::Command::from(&step);
        cmd.current_dir(build_dir);

        let output = cmd.output().context("Failed to execute build command")?;

        if !output.status.success() {
            log::info!("[✗] '{}' failed", step);
            log::debug!("command: {}", step);
            log::debug!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            log::debug!("stderr: {}", String::from_utf8_lossy(&output.stderr));
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
) -> anyhow::Result<()> {
    pub(crate) fn aux(
        test: &Test,
        experiment_config: &crate::config::ExperimentConfig,
        snapshot: ExperimentSnapshot,
        short_circuit: bool,
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
        let mut params = HashMap::from([(
            "workload_path".to_string(),
            workload_dir.display().to_string(),
        )]);

        if let Some(params_) = &test.params {
            for (key, value) in params_.iter() {
                log::trace!("Adding parameter: {} = {}", key, value);
                params.insert(key.clone(), value.to_string());
            }
        };

        let config_path = workload_dir.join("config").with_extension("json");
        log::debug!("config path: {}", config_path.display());
        let cfg: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        log::debug!("config: {}", cfg);
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
                test.cross,
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
            &workload.steps.check,
            &workload.steps.build,
            &params,
            &tags,
        )?;

        for (strategy, property) in test.tasks.iter() {
            params.insert("strategy".to_string(), strategy.clone());
            params.insert("property".to_string(), property.clone());

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
                cross: test.cross,
                seeds: None,
            };
            let result = run(
                experiment_config,
                &run_config,
                &workload.steps.run,
                &mut params,
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

    let result = aux(test, experiment_config, snapshot, short_circuit);
    if let Err(e) = &result {
        log::error!("Experiment failed with error: {}", e);
    } else {
        log::info!("Experiment completed successfully");
    }

    marauders::run_reset_command(&experiment_config.path)?;
    result
}
