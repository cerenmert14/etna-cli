use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::{
    config::{EtnaConfig, ExperimentConfig},
    experiment::Experiment,
    git_driver,
    store::Store,
};

/// A new experiment is create in the provided path
/// If the path is not provided, the current directory is used
/// The directory structure is as follows:
///
/// path/name/
/// |-------config.toml
/// |-------Collect.py
/// |-------Query.py
/// |-------Analyze.py
/// |-------Visualize.py
/// |-------workloads/
///         |-------[language-1]/
///                 |-------config.json
///                 |-------[workload-1]
///                         |-------config.json
///                 |-------[workload-2]
///                         |-------config.json
///         |-------...
/// |-------tests
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
/// config.toml - Configuration file for the experiment
/// - name: Name of the experiment
/// - description: Description of the experiment
/// - [workloads]: List of workloads to be executed
///     - language: Language of the workload
///     - path: Name of the workload
///
/// Collect.py - A default script to collect data from the workloads
/// Query.py - A default script to query the collected data
/// Analyze.py - A default script to analyze the collected data
/// Visualize.py - A default script to visualize the collected data
pub fn invoke(
    name: String,
    path: Option<PathBuf>,
    overwrite: bool,
    register: bool,
    description: Option<String>,
    use_local_store: bool,
) -> anyhow::Result<()> {
    // Create a new directory for the experiment
    // If the path is not provided, use the current directory
    log::trace!("creating new experiment with name '{name}'");
    let path = if let Some(path) = path {
        path
    } else {
        std::env::current_dir().context("Failed to get current directory")?
    };

    log::trace!("reading etna configuration");

    let global_store = EtnaConfig::get_etna_config().context("Could not read etna configuration, did you run `etna setup` before creating an experiment?")?.store_path();
    let local_store = if use_local_store {
        log::info!(
            "using a local store for experiment '{name}' at '{}'",
            path.join(&name).join("store.json").display()
        );
        path.join(&name).join("store.json")
    } else {
        global_store.clone()
    };
    // Create the config file
    let experiment_path = path.join(&name);
    let config_path = experiment_path.join("config.toml");
    let description = description.unwrap_or_else(|| "A description of the experiment".to_string());

    let experiment_config = ExperimentConfig::new(
        name.clone(),
        description,
        experiment_path.clone(),
        local_store.clone(),
    );

    log::trace!("loading global store at '{}'", global_store.display());
    let mut global_store = Store::load(&global_store).with_context(|| {
        format!(
            "Failed to load the global store at '{}'",
            global_store.display()
        )
    })?;
    let experiment_ = global_store.get_experiment_by_name(&name);

    log::trace!("running the --register and --overwrite logic");

    match (
        experiment_path.exists(),
        experiment_.is_ok(),
        overwrite,
        register,
    ) {
        (_, _, true, true) => {
            anyhow::bail!("Cannot use both --register and --overwrite at the same time")
        }
        (true, _, true, false) => {
            log::debug!("--overwrite flag is set, removing existing experiment directory");
            fs::remove_dir_all(&experiment_path).with_context(|| {
                format!(
                    "Failed to remove existing experiment directory at '{}'",
                    experiment_path.display()
                )
            })?
        }
        (true, false, false, true) => {
            log::debug!("--register flag is set, registering existing experiment");
            let snapshot = global_store.take_snapshot(&experiment_config)?;
            global_store.experiments.insert(Experiment {
                name: name.clone(),
                id: snapshot.experiment.clone(),
                description: experiment_config.description.clone(),
                path,
                snapshot,
            });
            global_store.save().with_context(|| {
                format!(
                    "Failed to save the etna store at '{}'",
                    global_store.path.display()
                )
            })?;
            log::info!(
                "Experiment '{name}' registered successfully at '{}'",
                experiment_config.path.display()
            );
            return Ok(());
        }
        (true, true, false, true) => {
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
            log::warn!("--overwrite flag is set, but the experiment does not exist. Creating experiment as usual.")
        }
        (false, _, false, true) => {
            anyhow::bail!("--register flag is set, but the experiment does not exist")
        }
        (false, _, false, false) => {}
    };

    log::trace!(
        "creating experiment directory at '{}'",
        experiment_path.display()
    );
    std::fs::create_dir(&experiment_path).with_context(|| {
        format!(
            "Failed to create experiment directory at '{}'",
            experiment_path.display()
        )
    })?;

    log::trace!("writing configuration file at '{}'", config_path.display());
    std::fs::write(
        &config_path,
        toml::to_string(&experiment_config).with_context(|| {
            format!(
                "Failed to serialize configuration at '{}'",
                config_path.display()
            )
        })?,
    )
    .context("Failed to create config file")?;

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
            "tests/t1.json",
            std::include_str!("../../../templates/tests/t1.json"),
        ),
    ];

    log::trace!("creating template files in the experiment directory");
    for (path, content) in template_files.iter() {
        log::trace!(
            "Creating template file at '{}/{}'",
            experiment_config.path.display(),
            path
        );
        let path = experiment_config.path.join(path);
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
    let workloads_path = experiment_config.path.join("workloads");
    log::trace!(
        "creating workloads directory at '{}'",
        workloads_path.display()
    );
    std::fs::create_dir(&workloads_path).with_context(|| {
        format!(
            "Failed to create workloads directory at '{}'",
            workloads_path.display()
        )
    })?;

    log::trace!(
        "initializing git repository at '{}'",
        experiment_config.path.display()
    );
    git_driver::initialize_git_repo(
        &experiment_config.path,
        format!("Automated initialization commit for experiment '{}'", name).as_str(),
    )?;

    log::trace!("taking snapshot for the experiment");
    let snapshot = global_store.take_snapshot(&experiment_config)?;

    global_store.experiments.insert(Experiment {
        name: name.clone(),
        id: snapshot.experiment.clone(),
        description: experiment_config.description.clone(),
        path: experiment_config.path.clone(),
        snapshot: snapshot.clone(),
    });

    log::trace!(
        "saving the global store at '{}'",
        global_store.path.display()
    );
    global_store.save().with_context(|| {
        format!(
            "Failed to save the global store at '{}'",
            global_store.path.display()
        )
    })?;

    if use_local_store {
        let mut local_store = Store::new(local_store);

        local_store.experiments.insert(Experiment {
            name: name.clone(),
            id: snapshot.experiment.clone(),
            description: experiment_config.description,
            path: experiment_config.path.clone(),
            snapshot,
        });

        log::trace!("saving the local store at '{}'", local_store.path.display());
        local_store.save().with_context(|| {
            format!(
                "Failed to save the local store at '{}'",
                local_store.path.display()
            )
        })?;
    }

    log::info!(
        "Experiment '{name}' created successfully at '{}'",
        experiment_config.path.display()
    );

    Ok(())
}
