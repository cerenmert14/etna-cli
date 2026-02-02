use crate::{
    manager::Manager,
    service::{check::integrity_check, types::IntegrityCheckOptions},
};

/// Run an integrity check on the experiment store using the service layer.
///
/// The CLI handles argument parsing and delegates to the service layer
/// for the actual integrity check logic.
pub fn invoke(mut mgr: Manager, restore: bool, remove: bool) -> anyhow::Result<()> {
    // Convert CLI args to service options
    let options = IntegrityCheckOptions { restore, remove };

    // Call service layer
    let result = integrity_check(&mut mgr, options)?;

    // Format output based on result
    if result.faults.is_empty() {
        tracing::info!("No integrity issues found!");
    } else if result.fixed {
        tracing::info!("Fixed {} integrity issues", result.faults.len());
    } else {
        tracing::info!(
            "Integrity check found {} issues. Use `etna check integrity --restore` to restore or `--remove` to remove the faulty entries.",
            result.faults.len()
        );
    }

    Ok(())
}
