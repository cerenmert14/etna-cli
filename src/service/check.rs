use anyhow::bail;

use crate::{manager::Manager, service::types::IntegrityFaultType};

use super::types::{IntegrityCheckOptions, IntegrityCheckResult, IntegrityFault, ServiceResult};

/// Run an integrity check on the experiment store
pub fn integrity_check(
    mgr: &mut Manager,
    options: IntegrityCheckOptions,
) -> ServiceResult<IntegrityCheckResult> {
    let IntegrityCheckOptions { restore, remove } = options;

    tracing::debug!("checking integrity of the store");

    if restore && remove {
        bail!("Cannot use both --restore and --remove at the same time in integrity check");
    }

    let mut faults = vec![];

    // Check the integrity of the store
    for (name, experiment) in mgr.experiments.iter() {
        // check that the experiment exists at the path
        match std::fs::metadata(&experiment.path) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    tracing::debug!(
                        "Experiment {} at {} is not a directory",
                        name,
                        experiment.path.display()
                    );
                    faults.push(IntegrityFault {
                        fault_type: IntegrityFaultType::ExperimentNotDirectory,
                        name: name.clone(),
                        path: experiment.path.clone(),
                        message: format!(
                            "Experiment {} at {} is not a directory",
                            name,
                            experiment.path.display()
                        ),
                    });
                }
            }
            Err(_) => {
                tracing::debug!(
                    "Experiment {} at {} does not exist",
                    name,
                    experiment.path.display()
                );
                faults.push(IntegrityFault {
                    fault_type: IntegrityFaultType::ExperimentNotFound,
                    name: name.clone(),
                    path: experiment.path.clone(),
                    message: format!(
                        "Experiment {} at {} does not exist",
                        name,
                        experiment.path.display()
                    ),
                });
            }
        }
    }

    let mut fixed = false;

    if restore {
        // TODO: implement restore functionality
        tracing::warn!("restore is not implemented yet");
    }

    if remove {
        for fault in faults.iter() {
            tracing::info!("fixing '{}'", fault.message);
            match fault.fault_type {
                IntegrityFaultType::ExperimentNotFound
                | IntegrityFaultType::ExperimentNotDirectory => {
                    mgr.retain_experiments(|e| e.name != fault.name)?;
                    tracing::info!("\tremoved experiment {} from the store", fault.name);
                }
            }
        }
        fixed = true;
    }

    if !restore && !remove {
        for fault in faults.iter() {
            tracing::info!("{}", fault.message);
        }
        if !faults.is_empty() {
            tracing::info!(
                "Integrity check found {} issues. Use restore or remove options to fix.",
                faults.len()
            );
        }
    }

    if faults.is_empty() {
        tracing::info!("No integrity issues found!");
    }

    Ok(IntegrityCheckResult { faults, fixed })
}
