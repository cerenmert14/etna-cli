use std::{fmt::Display, path::PathBuf};

use anyhow::Context as _;
use serde::{Deserialize as _, Serialize as _};
use serde_derive::{Deserialize, Serialize};

use crate::{git_driver, manager::Manager, workload::WorkloadMetadata};

/// Experiment Configuration
/// It contains the name of the experiment, a description of the experiment, and a list of workloads
/// to be executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMetadata {
    pub name: String,
    pub path: PathBuf,
    pub store: PathBuf,
}

impl ExperimentMetadata {
    pub(crate) fn has_workload(&self, language: &str, name: &str) -> bool {
        self.path
            .join("workloads")
            .join(language)
            .join(name)
            .exists()
    }
    pub(crate) fn workloads(&self) -> Vec<WorkloadMetadata> {
        todo!()
    }

    pub(crate) fn hash(&self) -> anyhow::Result<String> {
        // get the git head at the top
        git_driver::_head_hash(&self.path)
    }
}

impl ExperimentMetadata {
    pub fn from_current_dir(mgr: &Manager) -> anyhow::Result<Self> {
        // Find an experiment in the manager's list that is a parent of the current directory
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;

        let experiment = mgr
            .experiments
            .values()
            .find(|exp| current_dir.starts_with(&exp.path))
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Current directory is not inside any known experiment. Current dir: {}",
                    current_dir.display()
                )
            })?;

        Ok(experiment)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub(crate) struct Experiment {
    pub name: String,
    pub path: PathBuf,
    pub store: PathBuf,

    #[serde(default)]
    pub readme: Option<String>,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub tests: Vec<String>,
    #[serde(default)]
    pub workloads: Vec<WorkloadMetadata>,
}

#[derive(Serialize, Deserialize)]
struct TestWrapper<'a> {
    strategy: &'a str,
    property: &'a str,
}

fn serialize_test<S>(test: &[(String, String)], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let test_wrapper: Vec<TestWrapper> = test
        .iter()
        .map(|(strategy, property)| TestWrapper { strategy, property })
        .collect();
    test_wrapper.serialize(serializer)
}
fn deserialize_test<'de, D>(deserializer: D) -> Result<Vec<(String, String)>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let test_wrapper: Vec<TestWrapper> = Vec::deserialize(deserializer)?;
    Ok(test_wrapper
        .into_iter()
        .map(|TestWrapper { strategy, property }| (strategy.to_string(), property.to_string()))
        .collect())
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub(crate) struct Test {
    pub(crate) language: String,
    pub(crate) workload: String,
    pub(crate) trials: usize,
    pub(crate) timeout: f64,
    pub(crate) mutations: Vec<String>,
    #[serde(default)]
    pub(crate) cross: bool,
    #[serde(default)]
    pub(crate) params: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(
        serialize_with = "serialize_test",
        deserialize_with = "deserialize_test"
    )]
    pub(crate) tasks: Vec<(String, String)>,
}

impl Display for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(language: {}, workload: {}, trials: {}, timeout: {}, cross: {}, mutations: {:?}, tasks: {:?})",
            self.language, self.workload, self.trials, self.timeout, self.cross, self.mutations, self.tasks
        )
    }
}
