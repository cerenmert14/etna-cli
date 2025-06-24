use std::{fs, path::PathBuf, process::Command};

use anyhow::Context;

use crate::{
    config::{EtnaConfig, ExperimentConfig},
    experiment, git_driver, store,
    workload::WorkloadMetadata,
};

pub fn invoke(
    experiment_name: Option<String>,
    language: String,
    workload: String,
) -> anyhow::Result<()> {
    log::debug!(
        "adding workload '{}/{}' to {:?}",
        language,
        workload,
        experiment_name
    );
    // Get etna configuration
    let etna_config = EtnaConfig::get_etna_config().context("Failed to get etna config")?;
    // Get the current experiment
    let mut experiment_config = experiment_name
        .ok_or(anyhow::anyhow!("No experiment name provided"))
        .and_then(|n| ExperimentConfig::from_etna_config(&n, &etna_config))
        .or_else(|_| ExperimentConfig::from_current_dir())
        .context("No experiment name is provided, and the current directory is not an experiment directory")
        .context("Try running `etna workload add` in an experiment directory, or explicitly specify the experiment name with `etna workload add --experiment <NAME>`")?;

    // Check if the workload already exists
    if experiment_config.has_workload(&language, &workload) {
        anyhow::bail!("Workload '{}/{}' already exists", language, workload);
    }

    // get etna directory
    let repo_dir = std::env::var("ETNA_DIR")
        .and_then(|repo_dir| Ok(PathBuf::from(repo_dir)))
        .context("ETNA_DIR environment variable not set")?;

    // Get the workload path
    let workload_path = repo_dir.join("workloads").join(&language).join(&workload);

    // Check if the workload exists
    if !workload_path.exists() {
        anyhow::bail!("Workload '{}' not found", workload_path.display());
    }

    // Copy the workload to the experiment directory
    let dest_path = experiment_config
        .path
        .join("workloads")
        .join(&language)
        .join(&workload);

    std::fs::create_dir_all(
        dest_path
            .parent()
            .context("Failed to get parent directory")?,
    )
    .context("Failed to create parent directory")?;

    Command::new("cp")
        .arg("-r")
        .arg(&workload_path)
        .arg(&dest_path)
        .status()
        .context(format!(
            "Failed to copy workload at '{}' to '{}'",
            fs::canonicalize(workload_path)
                .context("Failed to get canonical path")?
                .display(),
            dest_path.display()
        ))?;

    // if language config exists, copy it
    let language_config_path = repo_dir
        .join("workloads")
        .join(&language)
        .join("config.json");

    if language_config_path.exists() {
        log::debug!(
            "Copying language config from '{}' to '{}'",
            language_config_path.display(),
            dest_path.display()
        );

        let dest_language_config_path = dest_path
            .canonicalize()?
            .parent()
            .context("Parent does not exist")?
            .join("config.json");

        std::fs::copy(&language_config_path, &dest_language_config_path).context(format!(
            "Failed to copy language config from '{}' to '{}'",
            fs::canonicalize(language_config_path)
                .context("Failed to get canonical path")?
                .display(),
            dest_language_config_path.display()
        ))?;
    }
    // Add the workload to the config
    experiment_config.workloads.push(WorkloadMetadata {
        language: language.clone(),
        name: workload.clone(),
    });

    // Write the updated config file
    let config_path = experiment_config.path.join("config.toml");
    std::fs::write(
        &config_path,
        toml::to_string(&experiment_config).context("Failed to serialize configuration")?,
    )
    .context("Failed to write config file")?;

    // Create a commit
    git_driver::commit_add_workload(&experiment_config.path, &language, &workload)
        .with_context(|| format!("Failed to commit adding '{language}/{workload}'"))?;

    // Add the snapshot to the store
    let mut store = store::Store::load(&experiment_config.store).context("Failed to load store")?;

    let snapshot = store.take_snapshot(&experiment_config)?;

    store.experiments.insert(experiment::Experiment {
        name: experiment_config.name.clone(),
        id: snapshot.experiment.clone(),
        description: experiment_config.description,
        path: experiment_config.path,
        snapshot,
    });

    store.save().context("Failed to save store")?;

    log::info!(
        "Workload '{}/{}' added to experiment '{}'",
        language,
        workload,
        experiment_config.name
    );

    Ok(())
}
