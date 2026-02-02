use std::{fs, path::Path, process::Command};

use anyhow::{bail, Context};

use crate::{
    experiment::ExperimentMetadata, git_driver, manager::Manager, workload::WorkloadMetadata,
};

use super::types::ServiceResult;

/// Copy language files to the experiment workloads directory
fn copy_language(repo_dir: &Path, workloads_dir: &Path, language: &str) -> anyhow::Result<()> {
    git_driver::pull_via_cli(repo_dir)?;

    for entry in repo_dir.join("workloads").join(language).read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            fs::copy(
                &path,
                workloads_dir.join(language).join(
                    path.file_name()
                        .context("Failed to get file name")?
                        .to_str()
                        .context("Failed to convert file name to string")?,
                ),
            )
            .with_context(|| {
                format!(
                    "Failed to copy file '{}' to '{}'",
                    path.display(),
                    workloads_dir
                        .join(language)
                        .join(
                            path.file_name()
                                .context("Failed to get file name")
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or_default()
                        )
                        .display()
                )
            })?;
        } else if path.is_dir() && !path.join("steps.json").exists() {
            // copy the entire directory
            Command::new("cp")
                .arg("-r")
                .arg(&path)
                .arg(
                    workloads_dir.join(language).join(
                        path.file_name()
                            .context("Failed to get directory name")?
                            .to_str()
                            .context("Failed to convert directory name to string")?,
                    ),
                )
                .status()
                .with_context(|| {
                    format!(
                        "Failed to copy directory '{}' to '{}'",
                        path.display(),
                        workloads_dir
                            .join(language)
                            .join(
                                path.file_name()
                                    .context("Failed to get directory name")
                                    .unwrap_or_default()
                                    .to_str()
                                    .unwrap_or_default()
                            )
                            .display()
                    )
                })?;
        }
    }

    Ok(())
}

/// Add a workload to an experiment
pub fn add_workload(
    mgr: &Manager,
    experiment: &ExperimentMetadata,
    language: &str,
    workload: &str,
) -> ServiceResult<WorkloadMetadata> {
    tracing::debug!(
        "adding workload '{}/{}' to {:?}",
        language,
        workload,
        experiment.name
    );

    // Check if the workload already exists
    if experiment.has_workload(language, workload) {
        bail!("Workload already exists: {}/{}", language, workload);
    }

    // Get etna directory
    let repo_dir = mgr.config.repo_dir();

    // Get the workload path
    let workload_path = repo_dir.join("workloads").join(language).join(workload);

    // Check if the workload exists, pull from remote if not
    if !workload_path.exists() {
        tracing::warn!(
            "Workload '{}' not found, pulling from remote",
            workload_path.display()
        );
        git_driver::pull_via_cli(&repo_dir)?;
    }

    let dest_path = experiment
        .path
        .join("workloads")
        .join(language)
        .join(workload);

    std::fs::create_dir_all(
        dest_path
            .parent()
            .context("Failed to get parent directory")?,
    )
    .context("Failed to create parent directory")?;

    // Copy the language files
    copy_language(&repo_dir, &experiment.path.join("workloads"), language)?;

    // Copy the workload
    Command::new("cp")
        .arg("-r")
        .arg(&workload_path)
        .arg(&dest_path)
        .status()
        .context(format!(
            "Failed to copy workload at '{}' to '{}'",
            fs::canonicalize(&workload_path)
                .context("Failed to get canonical path")?
                .display(),
            dest_path.display()
        ))?;

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

    Ok(WorkloadMetadata {
        name: workload.to_string(),
        language: language.to_string(),
    })
}

/// Remove a workload from an experiment
pub fn remove_workload(
    experiment: &ExperimentMetadata,
    language: &str,
    workload: &str,
) -> ServiceResult<()> {
    // Check if the workload exists
    if !experiment.has_workload(language, workload) {
        bail!("Workload not found: {}/{}", language, workload);
    }

    // Remove the workload from the experiment directory
    let dest_path = experiment
        .path
        .join("workloads")
        .join(language)
        .join(workload);

    fs::remove_dir_all(&dest_path).context(format!(
        "Failed to remove workload at '{}'",
        dest_path.display()
    ))?;

    // Create a commit
    git_driver::commit(
        &experiment.path,
        format!("remove '{language}/{workload}'").as_str(),
    )?;

    tracing::info!(
        "Workload '{}/{}' removed from experiment '{}'",
        language,
        workload,
        experiment.name
    );

    Ok(())
}

/// List workloads in an experiment
pub fn list_workloads(
    experiment: &ExperimentMetadata,
    language_filter: Option<&str>,
) -> ServiceResult<Vec<WorkloadMetadata>> {
    let mut workloads: Vec<WorkloadMetadata> = experiment
        .workloads()
        .into_iter()
        .filter(|wl| {
            language_filter.is_none()
                || language_filter == Some("all")
                || language_filter == Some(&wl.language)
        })
        .collect();

    workloads.sort_by(|a, b| a.language.cmp(&b.language).then(a.name.cmp(&b.name)));

    Ok(workloads)
}
