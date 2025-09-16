use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Ok};

use crate::{experiment::ExperimentMetadata, git_driver, manager::Manager};

/// etna holds a `workloads` directory that has `workloads/<language>/<workload>` directories
/// each workload directory has a `steps.json` file and possibly other files
/// when adding a workload to an experiment, we copy the entire `<workload>` directory,
/// but we also want to copy any relevant files or folders in the `<language>` directory.
/// for this purpose, we copy all files, and any directories that are not workloads (i.e., does
/// not have a `steps.json` file).
fn copy_language(repo_dir: &Path, workloads_dir: &Path, language: &str) -> anyhow::Result<()> {
    git_driver::pull_path(repo_dir, &PathBuf::from("workloads").join(language))?;

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
        } else if path.is_dir() {
            if !path.join("steps.json").exists() {
                // copy the entire directory
                Command::new("cp")
                    .arg("-r")
                    .arg(&path)
                    .arg(
                        &workloads_dir.join(language).join(
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
    }

    Ok(())
}

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

    // Copy the language
    copy_language(&repo_dir, &experiment.path.join("workloads"), &language)?;

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
