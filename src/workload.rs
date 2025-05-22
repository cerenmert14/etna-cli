use std::{collections::HashMap, hash::Hash, path::PathBuf};

use anyhow::Context;
use itertools::Itertools;
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
    pub(crate) fn realize(
        &self,
        params: &HashMap<String, String>,
        tags: &HashMap<String, Vec<String>>,
    ) -> anyhow::Result<Vec<Step>> {
        let step = self.clone();
        let mut steps = vec![];
        // elaboration step
        let mut elaborates = vec![];
        for (key, value) in params.iter() {
            println!("key {}", key);
            println!("step.command {}", step.command);
            println!("step.args {:?}", step.args);
            if step.command.contains(&format!("!{}", key)) {
                elaborates.push(key.clone());
            }
            if step
                .args
                .iter()
                .any(|arg| arg.contains(&format!("!{}", key)))
            {
                elaborates.push(key.clone());
            }
        }
        println!("elobrates {:?}", elaborates);
        let all_elaborations = elaborates
            .iter()
            .map(|key| {
                tags.get(key)
                    .expect(format!("missing tag {}", key).as_str())
                    .clone()
            })
            .multi_cartesian_product()
            .collect::<Vec<Vec<_>>>();

        for elaboration_set in all_elaborations {
            let mut step = step.clone();
            for (i, val) in elaboration_set.iter().enumerate() {
                println!("wtf is happening");
                step.command = step.command.replace(&format!("!{}", val), &elaborates[i]);
                step.args = step
                    .args
                    .iter()
                    .map(|arg| arg.replace(&format!("!{}", val), &elaborates[i]))
                    .collect();
            }
            steps.push(step);
        }

        for step in steps.iter_mut() {
            for (key, value) in params.iter() {
                step.command = step.command.replace(&format!("${}", key), value);
                step.args = step
                    .args
                    .iter()
                    .map(|arg| arg.replace(&format!("${}", key), value))
                    .collect();
            }
        }

        Ok(steps)
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
