use std::{collections::HashMap, hash::Hash, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{property::Property, strategy::Strategy};
use marauders::Variation;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Step {
    pub(crate) command: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) args: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) params: Vec<String>,
}

impl Step {
    pub(crate) fn realize(&self, params: &HashMap<String, String>) -> anyhow::Result<Vec<Step>> {
        let mut step = self.clone();

        for (key, value) in params.iter() {
            step.command = step.command.replace(&format!("${}", key), value);
            step.args = step
                .args
                .iter()
                .map(|arg| arg.replace(&format!("${}", key), value))
                .collect();
            step.params = step
                .params
                .iter()
                .map(|param| param.replace(&format!("${}", key), value))
                .collect();
        }

        Ok(step)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub(crate) struct WorkloadMetadata {
    pub(crate) name: String,
    pub(crate) language: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct Workload {
    pub(crate) name: String,
    pub(crate) language: String,
    pub(crate) dir: PathBuf,
    pub(crate) properties: Vec<Property>,
    pub(crate) variations: Vec<Variation>,
    pub(crate) strategies: Vec<Strategy>,
    pub(crate) build_steps: Vec<Step>,
    pub(crate) check_steps: Vec<Step>,
    pub(crate) run_step: Step,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct Language {
    pub(crate) name: String,
    pub(crate) build_steps: Vec<Step>,
    pub(crate) check_steps: Vec<Step>,
    pub(crate) run_step: Step,
}
