use std::{collections::HashMap, fmt::Display, hash::Hash, path::PathBuf};

use anyhow::Context;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{property::Property, strategy::Strategy};
use marauders::Variation;

pub(crate) struct Command {
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.args.join(" "))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum Step {
    #[serde()]
    Command {
        command: String,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        args: Vec<String>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        params: Vec<String>,
    },
    Match {
        value: String,
        options: HashMap<String, Step>,
    },
}

impl Step {
    pub(crate) fn params(&self) -> Vec<&str> {
        match self {
            Step::Command { params, .. } => params.iter().map(|p| p.as_str()).collect(),
            Step::Match { value, options } => [
                vec![value.as_str()],
                options.values().map(|o| o.params()).flatten().collect(),
            ]
            .concat(),
        }
    }
    pub(crate) fn decide(
        &self,
        params: &HashMap<String, String>,
        tags: &HashMap<String, Vec<String>>,
    ) -> Command {
        match self {
            Step::Command { command, args, .. } => Command {
                command: command.clone(),
                args: args.clone(),
            },
            Step::Match { value, options } => {
                let guard = params.get(value).unwrap();
                log::debug!("obtaining guard '{guard}' for tags_ {tags:?}");
                let tags_ = tags.get(guard).unwrap();
                for (k, step) in options {
                    if tags_.contains(k) {
                        return step.decide(params, tags);
                    }
                }

                panic!("None of the options fit")
            }
        }
    }

    pub(crate) fn contains(&self, k: &str) -> bool {
        match self {
            Step::Command { command, args, .. } => {
                command.contains(k) && args.iter().any(|a| a.contains(k))
            }
            Step::Match { options, .. } => options.values().any(|s| s.contains(k)),
        }
    }

    pub(crate) fn replace(&mut self, s1: &str, s2: &str) {
        match self {
            Step::Command { command, args, .. } => {
                *command = command.replace(s1, s2);
                *args = args.iter().map(|arg| arg.replace(s1, s2)).collect();
            }
            Step::Match { options, .. } => {
                for step in options.values_mut() {
                    step.replace(s1, s2)
                }
            }
        }
    }

    pub(crate) fn realize(
        &self,
        params: &HashMap<String, String>,
        tags: &HashMap<String, Vec<String>>,
    ) -> anyhow::Result<Vec<Step>> {
        let step = self.clone();
        let mut steps = vec![];
        // elaboration step
        let mut elaborates = vec![];
        for param in step.params() {
            if step.contains(&format!("!{}", param)) {
                elaborates.push(param);
            }
        }
        println!("elobrates {:?}", elaborates);
        let all_elaborations = elaborates
            .iter()
            .map(|key| {
                tags.get(*key)
                    .expect(format!("missing tag {}", key).as_str())
                    .clone()
            })
            .multi_cartesian_product()
            .collect::<Vec<Vec<_>>>();

        for elaboration_set in all_elaborations {
            let mut step = step.clone();
            for (i, val) in elaboration_set.iter().enumerate() {
                println!("wtf is happening");
                step.replace(&format!("!{}", val), &elaborates[i]);
            }
            steps.push(step);
        }

        for step in steps.iter_mut() {
            for (key, value) in params.iter() {
                step.replace(&format!("${}", key), value);
            }
        }

        Ok(steps)
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Command { command, args, .. } => write!(f, "{} {}", command, args.join(" ")),
            Step::Match { value, options } => {
                write!(f, "match '{}': ", value)?;
                for (k, step) in options {
                    write!(f, "({} => {})", k, step)?;
                }
                Ok(())
            }
        }
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
