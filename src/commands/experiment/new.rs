use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::{experiment::ExperimentMetadata, git_driver, manager::Manager, store::Store};

/// A new experiment is create in the provided path
/// If the path is not provided, the current directory is used
/// The directory structure is as follows:
///
/// path/name/
/// |-------config.toml
/// |-------figures/
///         |-------fig1.png
/// |-------scripts/
///         |-------Collect.py      - A default script to collect data from the workloads
///         |-------Query.py        - A default script to query the collected data
///         |-------Analyze.py      - A default script to analyze the collected data
///         |-------Visualize.py    - A default script to visualize the collected data
/// |-------workloads/
///         |-------[language-1]/
///                 |-------config.json
///                 |-------[workload-1]
///                         |-------config.json
///                 |-------[workload-2]
///                         |-------config.json
///         |-------...
/// |-------tests/
///         |-------t1.json
///         |-------t2.json
///         |-------...
/// |-------.git
/// |-------.gitignore
///
/// # Arguments
/// * `name` - Name of the new experiment
/// * `path` - Path where the new experiment should be created
///
pub fn invoke(
    mut mgr: Manager,
    name: String,
    path: Option<PathBuf>,
    overwrite: bool,
    register: bool,
    use_local_store: bool,
) -> anyhow::Result<()> {
    // Create a new directory for the experiment
    // If the path is not provided, use the current directory
    tracing::trace!("creating new experiment with name '{name}'");
    let path = if let Some(path) = path {
        path
    } else {
        std::env::current_dir().context("Failed to get current directory")?
    };

    if use_local_store {
        let store_path = path.join(&name).join("store.jsonl");
        tracing::info!(
            "using a local store for experiment '{name}' at '{}'",
            path.join(&name).join("store.jsonl").display()
        );
        mgr.store = Store::nkew(store_path)?;
    }

    let experiment_path = path.join(&name);
    
    let experiment_ = mgr.get_experiment(&name);

    tracing::trace!("running the --register and --overwrite logic");

    match (experiment_path.exists(), experiment_, overwrite, register) {
        (_, _, true, true) => {
            anyhow::bail!("Cannot use both --register and --overwrite at the same time")
        }
        (true, _, true, false) => {
            tracing::debug!("--overwrite flag is set, removing existing experiment directory");
            fs::remove_dir_all(&experiment_path).with_context(|| {
                format!(
                    "Failed to remove existing experiment directory at '{}'",
                    experiment_path.display()
                )
            })?
        }
        (true, None, false, true) => {
            tracing::debug!("--register flag is set, registering existing experiment");

            // Check if it has an internal store
            let store_path = experiment_path.join("store.jsonl");
            let local_store = if store_path.exists() {
                Some(store_path)
            } else {
                None
            };

            mgr.add_experiment(
                name.clone(),
                ExperimentMetadata {
                    name: name.clone(),
                    path,
                    store: local_store.unwrap_or(mgr.store.path.clone()),
                },
            )?;

            tracing::info!(
                "Experiment '{name}' registered successfully at '{}'",
                experiment_path.display()
            );

            return Ok(());
        }
        (true, Some(_), false, true) => {
            anyhow::bail!(
                "An experiment named '{name}' already exists in the store, you cannot re-register it");
        }
        (true, _, false, false) => {
            anyhow::bail!(
                    "An experiment named '{name}' already exists in '{}',\n\tuse `etna experiment new --overwrite` to create a new one instead, or `--register` to register the existing one",
                    fs::canonicalize(path)?.display()
                )
        }
        (false, _, true, false) => {
            tracing::warn!("--overwrite flag is set, but the experiment does not exist. Creating experiment as usual.")
        }
        (false, _, false, true) => {
            anyhow::bail!("--register flag is set, but the experiment does not exist")
        }
        (false, _, false, false) => {}
    };

    tracing::trace!(
        "creating experiment directory at '{}'",
        experiment_path.display()
    );
    std::fs::create_dir(&experiment_path).with_context(|| {
        format!(
            "Failed to create experiment directory at '{}'",
            experiment_path.display()
        )
    })?;

    // Create the template files
    let template_files = [
        (
            "Collect.py",
            std::include_str!("../../../templates/experimentation/Collect.pyt"),
        ),
        (
            "Query.py",
            std::include_str!("../../../templates/experimentation/Query.pyt"),
        ),
        (
            "Analyze.py",
            std::include_str!("../../../templates/experimentation/Analyze.pyt"),
        ),
        (
            "Visualize.py",
            std::include_str!("../../../templates/experimentation/Visualize.pyt"),
        ),
        (
            ".gitignore",
            std::include_str!("../../../templates/.gitignoret"),
        ),
        (
            "tests/bst.json",
            std::include_str!("../../../templates/tests/bst.json"),
        ),
        (
            "tests/rbt.json",
            std::include_str!("../../../templates/tests/rbt.json"),
        ),
        (
            "tests/stlc.json",
            std::include_str!("../../../templates/tests/stlc.json"),
        ),
    ];

    tracing::trace!("creating template files in the experiment directory");
    for (path, content) in template_files.iter() {
        tracing::trace!(
            "Creating template file at '{}/{}'",
            experiment_path.display(),
            path
        );
        let path = experiment_path.join(path);
        let parent = path.parent().context(format!(
            "Failed to get parent directory for '{}'",
            path.display()
        ))?;
        std::fs::create_dir_all(parent)?;
        std::fs::write(&path, content).context(format!(
            "Failed to create template file at '{}'",
            path.display()
        ))?;
    }

    // Create the workloads directory
    let workloads_path = experiment_path.join("workloads");
    tracing::trace!(
        "creating workloads directory at '{}'",
        workloads_path.display()
    );
    std::fs::create_dir(&workloads_path).with_context(|| {
        format!(
            "Failed to create workloads directory at '{}'",
            workloads_path.display()
        )
    })?;

    let scripts_path = experiment_path.join("scripts");
    tracing::trace!("creating scripts directory at '{}'", scripts_path.display());
    std::fs::create_dir(&scripts_path).with_context(|| {
        format!(
            "Failed to create scripts directory at '{}'",
            scripts_path.display()
        )
    })?;

    let figures_path = experiment_path.join("figures");
    tracing::trace!("creating figures directory at '{}'", figures_path.display());
    std::fs::create_dir(&figures_path).with_context(|| {
        format!(
            "Failed to create figures directory at '{}'",
            figures_path.display()
        )
    })?;

    tracing::trace!(
        "initializing git repository at '{}'",
        experiment_path.display()
    );
    git_driver::initialize_git_repo(
        &experiment_path,
        format!("Automated initialization commit for experiment '{}'", name).as_str(),
    )?;

    if use_local_store {
        let store_path = experiment_path.join("store.jsonl");
        tracing::trace!("creating local store file at '{}'", store_path.display());
        fs::File::create(&store_path).with_context(|| {
            format!(
                "Failed to create local store file at '{}'",
                store_path.display()
            )
        })?;
    }

    mgr.add_experiment(
        name.clone(),
        ExperimentMetadata {
            name: name.clone(),
            path: experiment_path.clone(),
            store: if use_local_store {
                experiment_path.join("store.jsonl")
            } else {
                mgr.store.path.clone()
            },
        },
    )?;

    tracing::info!(
        "Experiment '{name}' created successfully at '{}'",
        experiment_path.display()
    );

    Ok(())
}
