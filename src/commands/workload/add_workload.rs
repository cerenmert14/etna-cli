use crate::{
    experiment::ExperimentMetadata, manager::Manager, service::workload::add_workload,
};

/// Add a workload to an experiment using the service layer.
///
/// The CLI handles argument parsing and delegates to the service layer
/// for the actual workload addition logic.
pub fn invoke(
    mgr: Manager,
    experiment: ExperimentMetadata,
    language: String,
    workload: String,
) -> anyhow::Result<()> {
    // Call service layer
    let result = add_workload(&mgr, &experiment, &language, &workload)?;

    tracing::info!(
        "Workload '{}/{}' added to experiment '{}'",
        result.language,
        result.name,
        experiment.name
    );

    Ok(())
}
