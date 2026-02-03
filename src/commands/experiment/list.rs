use crate::{manager::Manager, service};

pub fn invoke(mgr: Manager) -> anyhow::Result<()> {
    let experiments = service::experiment::list_experiments(&mgr)?;

    if experiments.is_empty() {
        println!("No experiments found.");
        return Ok(());
    }

    println!("Experiments:");
    for exp in experiments {
        println!("- {} (path: {})", exp.name, exp.path.display());
    }

    Ok(())
}
