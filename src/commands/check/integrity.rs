use std::{fmt::Display, path::PathBuf};

use anyhow::Context;

use crate::{config::EtnaConfig, store};

enum IntegrityFault {
    ExperimentNotFound { name: String, path: PathBuf },
}

impl Display for IntegrityFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrityFault::ExperimentNotFound { name, path } => {
                write!(
                    f,
                    "Experiment {} at {} does not exist",
                    name,
                    path.display()
                )
            }
        }
    }
}

/// ETNA does not simply rely on the file system as its source of truth
/// but keeps a separate bookkeeping of workloads, experiments, tests etc.
/// We do this because explicit records are helpful for logging, listing,
/// and debugging; but we expect users to use the file system capabilities without
/// ETNA's help, at times.
/// In such a case, we need to identify divergences between the file system
/// and ETNA's bookkeeping; and offer mechanisms for reconciliation.
/// integrity check is one such mechanism.
pub fn invoke(restore: bool, remove: bool) -> anyhow::Result<()> {
    log::debug!("checking integrity of the store");
    if restore && remove {
        anyhow::bail!("Cannot use both --restore and --remove at the same time in integrity check");
    }
    let mut integrity_faults = vec![];
    // Get etna configuration
    let etna_config = EtnaConfig::get_etna_config().context("Failed to get etna config")?;
    // Load the store
    let mut store =
        store::Store::load(&etna_config.store_path()).context("Failed to load the store")?;
    // Check the integrity of the store
    for experiment in store.experiments.iter() {
        // check that the experiment exists at the path
        match std::fs::metadata(&experiment.path) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    log::debug!(
                        "Experiment {} at {} is not a directory",
                        experiment.name,
                        experiment.path.display()
                    );
                    integrity_faults.push(IntegrityFault::ExperimentNotFound {
                        name: experiment.name.clone(),
                        path: experiment.path.clone(),
                    })
                }
            }
            Err(_) => {
                log::debug!(
                    "Experiment {} at {} does not exist",
                    experiment.name,
                    experiment.path.display()
                );
                integrity_faults.push(IntegrityFault::ExperimentNotFound {
                    name: experiment.name.clone(),
                    path: experiment.path.clone(),
                });
            }
        }
    }

    if restore {
        todo!("restore is not implemented yet");
    }
    if remove {
        for integrity_fault in integrity_faults.iter() {
            log::info!("fixing '{}'", integrity_fault);
            match integrity_fault {
                IntegrityFault::ExperimentNotFound { name, .. } => {
                    store.experiments.retain(|e| e.name != *name);
                    store.save().context("Failed to save the store")?;
                    log::info!("\tremoved experiment {} from the store", name);
                }
            }
        }
    }
    if !restore && !remove {
        for integrity_fault in integrity_faults.iter() {
            log::info!("{}", integrity_fault);
        }
        if !integrity_faults.is_empty() {
            log::info!(
            "Integrity check found {} issues. Use `etna check integrity --restore` to restore or `--remove` to remove the faulty entries.",
            integrity_faults.len()
        );
        }
    }
    if integrity_faults.is_empty() {
        log::info!("No integrity issues found!");
    }
    Ok(())
}
