use std::{fs, path::PathBuf, process::Command};

use anyhow::Context;

use crate::{experiment::ExperimentMetadata, git_driver, manager::Manager};

pub fn invoke(
    mgr: Manager,
    experiment: ExperimentMetadata,
    language: String,
    workload: String,
) -> anyhow::Result<()> {
    tracing::debug!(
        "adding workload '{}/{}' to {:?}",
        language,
        workload,
        experiment.name
    );

    // Check if the workload already exists
    if experiment.has_workload(&language, &workload) {
        anyhow::bail!("Workload '{}/{}' already exists", language, workload);
    }

    // get etna directory
    let repo_dir = mgr.etna_dir().join(".etna_cache");

    // Get the workload path
    let workload_path = repo_dir.join("workloads").join(&language).join(&workload);

    // Check if the workload exists
    if !workload_path.exists() {
        tracing::warn!(
            "Workload '{}' not found, pulling from remote",
            workload_path.display()
        );
        git_driver::pull_workload(&repo_dir, &language, &workload)
            .context("Failed to pull from remote")?;
    }

    // Check if language steps exists
    let language_steps_path = repo_dir
        .join("workloads")
        .join(&language)
        .join("steps.json");

    if !language_steps_path.exists() {
        tracing::warn!(
            "Language steps '{}' not found, pulling from remote",
            language_steps_path.display()
        );

        git_driver::pull_path(
            &repo_dir,
            &PathBuf::from("workloads")
                .join(&language)
                .join("steps.json"),
        )
        .context("Failed to pull language steps from remote")?;
    }

    let dest_path = experiment
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

    // copy language steps exists
    let experiment_language_steps_path = experiment
        .path
        .join("workloads")
        .join(&language)
        .join("steps.json");

    if !experiment_language_steps_path.exists() {
        tracing::debug!(
            "Copying language steps from '{}' to '{}'",
            language_steps_path.display(),
            experiment_language_steps_path.display()
        );

        std::fs::copy(&language_steps_path, &experiment_language_steps_path).context(format!(
            "Failed to copy language steps from '{}' to '{}'",
            fs::canonicalize(language_steps_path)
                .context("Failed to get canonical path")?
                .display(),
            experiment_language_steps_path.display()
        ))?;
    }

    // Create a commit
    git_driver::commit(
        &experiment.path,
        format!("add workload '{}/{}'", language, workload).as_str(),
    )
    .with_context(|| format!("Failed to commit adding '{language}/{workload}'"))?;

    tracing::info!(
        "Workload '{}/{}' added to experiment '{}'",
        language,
        workload,
        experiment.name
    );

    Ok(())
}
