use crate::{experiment::ExperimentMetadata, service::workload::remove_workload};

/// Remove a workload from an experiment using the service layer.
///
/// The CLI handles argument parsing and delegates to the service layer
/// for the actual workload removal logic.
pub fn invoke(
    experiment: ExperimentMetadata,
    language: String,
    workload: String,
) -> anyhow::Result<()> {
    // Call service layer
    remove_workload(&experiment, &language, &workload)?;

    tracing::info!(
        "Workload '{}/{}' removed from experiment '{}'",
        language,
        workload,
        experiment.name
    );

    Ok(())
}
