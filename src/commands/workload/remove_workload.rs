use std::fs;

use anyhow::Context;

use crate::{experiment::ExperimentMetadata, git_driver};

pub fn invoke(
    experiment: ExperimentMetadata,
    language: String,
    workload: String,
) -> anyhow::Result<()> {
    // Check if the workload already exists
    if !experiment.has_workload(&language, &workload) {
        anyhow::bail!("Workload '{}/{}' does not exist", language, workload);
    }

    // Remove the workload from the experiment directory
    let dest_path = experiment
        .path
        .join("workloads")
        .join(&language)
        .join(&workload);

    fs::remove_dir_all(&dest_path).context(format!(
        "Failed to remove workload at '{}'",
        dest_path.display()
    ))?;

    // Create a commit
    git_driver::commit(
        &experiment.path,
        format!("remove '{language}/{workload}'").as_str(),
    )?;

    Ok(())
}
