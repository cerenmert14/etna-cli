use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize as _, Serialize as _};
use serde_derive::{Deserialize, Serialize};

use crate::workload::WorkloadMetadata;

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub(crate) struct Experiment {
    pub name: String,
    pub id: String,
    pub description: String,
    pub path: PathBuf,
    pub snapshot: ExperimentSnapshot,
}

impl Experiment {
    pub(crate) fn with_snapshot(&self, snapshot: ExperimentSnapshot) -> Self {
        Self {
            id: snapshot.experiment.clone(),
            snapshot,
            ..self.clone()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub(crate) struct ExperimentSnapshot {
    pub experiment: String,
    #[serde(default)]
    pub scripts: Vec<(String, String)>,
    #[serde(default)]
    pub tests: Vec<(String, String)>,
    #[serde(default)]
    pub workloads: Vec<(WorkloadMetadata, String)>,
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
            "(language: {}, workload: {}, trials: {}, timeout: {}, mutations: {:?}, tasks: {:?})",
            self.language, self.workload, self.trials, self.timeout, self.mutations, self.tasks
        )
    }
}
