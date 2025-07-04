use std::{collections::HashSet, path::PathBuf};

use anyhow::{Context, Ok};
use serde_derive::{Deserialize, Serialize};

use crate::{
    config::ExperimentConfig,
    experiment::{Experiment, ExperimentSnapshot},
    snapshot::{self, Snapshot},
    workload::WorkloadMetadata,
};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Store {
    pub path: PathBuf,
    pub experiments: HashSet<Experiment>,
    pub snapshots: HashSet<Snapshot>,
    pub metrics: Vec<Metric>,
}

impl Store {
    pub(crate) fn new(path: PathBuf) -> Self {
        Store {
            experiments: HashSet::new(),
            snapshots: HashSet::new(),
            metrics: Vec::new(),
            path,
        }
    }

    pub(crate) fn load(path: &PathBuf) -> anyhow::Result<Self> {
        log::trace!("loading store from {}", path.display());
        if !path.exists() {
            anyhow::bail!(
                "Failed to load the store, store file does not exist at {}",
                path.display()
            );
        }

        let content = std::fs::read_to_string(path)?;
        let store: Store = serde_json::from_str(&content)?;

        Ok(store)
    }

    pub(crate) fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&self.path, content).context("Failed to write store file")
    }

    pub(crate) fn take_snapshot(
        &mut self,
        experiment_config: &ExperimentConfig,
    ) -> anyhow::Result<ExperimentSnapshot> {
        log::trace!("taking snapshot for experiment {}", experiment_config.name);
        let experiment_snapshot = snapshot::Snapshot::take(
            &experiment_config.path,
            &PathBuf::from("*"),
            snapshot::SnapshotType::Experiment {
                time: chrono::Utc::now().to_rfc3339(),
            },
        )
        .context("Failed to take experiment snapshot")?;
        log::trace!("experiment snapshot taken: {:?}", experiment_snapshot);

        self.snapshots.insert(experiment_snapshot.clone());

        let script_snapshots = experiment_config
            .path
            .join("scripts")
            .read_dir()
            .context("Failed to read scripts directory")?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.path().is_file() {
                    Some(
                        snapshot::Snapshot::take(
                            &experiment_config.path,
                            &entry.path(),
                            snapshot::SnapshotType::Script {
                                name: entry.file_name().to_string_lossy().to_string(),
                            },
                        )
                        .context("Failed to take script snapshot"),
                    )
                } else {
                    None
                }
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        self.snapshots.extend(script_snapshots.clone());

        let test_snapshots = experiment_config
            .path
            .join("tests")
            .read_dir()
            .context("Failed to read tests directory")?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.path().is_file() {
                    Some(
                        snapshot::Snapshot::take(
                            &experiment_config.path,
                            &entry.path(),
                            snapshot::SnapshotType::Test {
                                name: entry.file_name().to_string_lossy().to_string(),
                            },
                        )
                        .context("Failed to take test snapshot"),
                    )
                } else {
                    None
                }
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        self.snapshots.extend(test_snapshots.clone());

        let workload_snapshots: Vec<(WorkloadMetadata, String)> = experiment_config
            .workloads
            .iter()
            .map(|workload| {
                let workload_snapshot = snapshot::Snapshot::take(
                    &experiment_config.path,
                    &PathBuf::from("workloads")
                        .join(PathBuf::from(&workload.language))
                        .join(PathBuf::from(&workload.name))
                        .join("*"),
                    snapshot::SnapshotType::Workload {
                        name: workload.name.to_string(),
                        language: workload.language.to_string(),
                    },
                )
                .context("Failed to take workloads snapshot")?;
                self.snapshots.insert(workload_snapshot.clone());

                Ok((workload.clone(), workload_snapshot.hash))
            })
            .filter_map(Result::ok)
            .collect();

        Ok(ExperimentSnapshot {
            experiment: experiment_snapshot.hash,
            scripts: script_snapshots
                .into_iter()
                .map(|s| (s.typ.name().unwrap(), s.hash))
                .collect(),
            tests: test_snapshots
                .into_iter()
                .map(|s| (s.typ.name().unwrap(), s.hash))
                .collect(),
            workloads: workload_snapshots,
        })
    }
}

impl Store {
    pub(crate) fn get_experiment_by_name(&self, name: &str) -> anyhow::Result<&Experiment> {
        log::trace!("getting latest experiment by name '{name}'");
        let experiments = self
            .experiments
            .iter()
            .filter(|experiment| experiment.name == name)
            .collect::<Vec<&Experiment>>();
        log::trace!(
            "found {} experiments with name '{}'",
            experiments.len(),
            name
        );
        let experiment_hashes = experiments
            .iter()
            .map(|experiment| experiment.id.clone())
            .collect::<Vec<String>>();
        log::trace!("experiment hashes: {}", experiment_hashes.join(", "));

        let snapshots = self
            .snapshots
            .iter()
            .filter(|snapshot| {
                snapshot.typ.is_experiment() && experiment_hashes.contains(&snapshot.hash)
            })
            .collect::<Vec<&Snapshot>>();
        log::trace!(
            "found {} snapshots for experiments with name '{}'",
            snapshots.len(),
            name
        );

        let latest_snapshot = snapshots
            .iter()
            .max_by(|a, b| a.typ.time().cmp(&b.typ.time()))
            .context("No snapshots found")?;

        log::trace!(
            "found latest experiment for snapshot '{}'",
            latest_snapshot.hash
        );

        let latest_experiment = self
            .experiments
            .iter()
            .find(|experiment| experiment.id == latest_snapshot.hash)
            .context("No experiment found")?;

        log::trace!("found latest experiment with id '{}'", latest_experiment.id);
        Ok(latest_experiment)
    }

    pub(crate) fn get_all_experiments_by_name(&self, name: &str) -> Vec<&Experiment> {
        log::trace!("getting all experiments by name '{name}'");
        self.experiments
            .iter()
            .filter(|experiment| experiment.name == name)
            .collect::<Vec<&Experiment>>()
    }

    pub(crate) fn get_experiment_by_id(&self, hash: &str) -> anyhow::Result<&Experiment> {
        log::trace!("getting the experiment by id '{hash}'");
        self.experiments
            .iter()
            .find(|experiment| experiment.id == hash)
            .with_context(|| format!("experiment with id '{hash}' not found"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Metric {
    pub data: serde_json::Value,
    pub experiment_id: String,
}

pub(crate) trait Queriable {
    fn query(&self, store: &Store) -> anyhow::Result<Vec<String>>;
}

pub(crate) enum SpecializedQuery {
    Experiment(ExperimentQuery),
    Metric(MetricQuery),
    Snapshot(SnapshotQuery),
}

impl Queriable for SpecializedQuery {
    fn query(&self, store: &Store) -> anyhow::Result<Vec<String>> {
        match self {
            SpecializedQuery::Experiment(query) => query.query(store),
            SpecializedQuery::Metric(query) => query.query(store),
            SpecializedQuery::Snapshot(query) => query.query(store),
        }
    }
}

pub(crate) enum ExperimentQuery {
    Id(String),
    NameLast(String),
    NameAll(String),
}

impl Queriable for ExperimentQuery {
    fn query(&self, store: &Store) -> anyhow::Result<Vec<String>> {
        match self {
            ExperimentQuery::Id(hash) => {
                let experiment = store.get_experiment_by_id(hash)?;
                Ok(vec![serde_json::to_string(experiment)?])
            }
            ExperimentQuery::NameLast(name) => {
                let experiment = store.get_experiment_by_name(name)?;
                Ok(vec![serde_json::to_string(experiment)?])
            }
            ExperimentQuery::NameAll(name) => {
                let experiments = store.get_all_experiments_by_name(name);
                experiments
                    .iter()
                    .map(|e| serde_json::to_string(e).context("Failed to serialize experiment"))
                    .collect()
            }
        }
    }
}

pub(crate) enum MetricQuery {
    ByExperimentId(String),
}

impl Queriable for MetricQuery {
    fn query(&self, store: &Store) -> anyhow::Result<Vec<String>> {
        match self {
            MetricQuery::ByExperimentId(hash) => {
                let metrics = store
                    .metrics
                    .iter()
                    .filter(|metric| metric.experiment_id == *hash)
                    .collect::<Vec<&Metric>>();

                metrics
                    .iter()
                    .map(|m| serde_json::to_string(m).context("Failed to serialize metric"))
                    .collect()
            }
        }
    }
}

pub(crate) enum SnapshotQuery {
    ByName(String),
    ByHash(String),
}

impl Queriable for SnapshotQuery {
    fn query(&self, store: &Store) -> anyhow::Result<Vec<String>> {
        match self {
            SnapshotQuery::ByName(name) => {
                let snapshots = store
                    .snapshots
                    .iter()
                    .filter(|snapshot| snapshot.typ.name().unwrap_or("".to_string()) == *name)
                    .collect::<Vec<&Snapshot>>();

                snapshots
                    .iter()
                    .map(|s| serde_json::to_string(s).context("Failed to serialize snapshot"))
                    .collect()
            }
            SnapshotQuery::ByHash(hash) => {
                let snapshot = store
                    .snapshots
                    .iter()
                    .find(|snapshot| snapshot.hash == *hash)
                    .context("Snapshot not found")?;

                Ok(vec![
                    serde_json::to_string(snapshot).context("Failed to serialize snapshot")?
                ])
            }
        }
    }
}
