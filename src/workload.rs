use std::{collections::HashMap, fmt::Display, hash::Hash, path::PathBuf};

use itertools::Itertools as _;
use serde::{Deserialize, Serialize};

use crate::{commands::experiment::run, property::Property, strategy::Strategy};
use marauders::Variation;

/// Represents a command that can be executed in the context of a workload.
pub(crate) struct Command {
    /// The command to be executed.
    pub(crate) command: String,
    /// The arguments to the command.
    pub(crate) args: Vec<String>,
    /// The directory where the command should be run.
    pub(crate) run_at: Option<String>,
    /// Optional mitigation information for the command.
    /// This can be used to specify how to handle potential issues or failures.
    pub(crate) mitigation: Option<String>,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(run_at) = &self.run_at {
            write!(f, "cd {} && ", run_at)?;
        }
        write!(f, "{} {}", self.command, self.args.join(" "))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Represents a step in a workload, which can either be a command to run or a match condition
/// that decides which command to run based on parameters and tags.
pub enum Step {
    #[serde()]
    Command {
        command: String,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        args: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        run_at: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        mitigation: Option<String>,
    },
    Match {
        value: String,
        options: HashMap<String, Step>,
    },
}

impl Step {
    pub(crate) fn decide(
        &self,
        params: &HashMap<String, String>,
        tags: &HashMap<String, Vec<String>>,
    ) -> Command {
        log::debug!("deciding step: {self} with params: {params:?} and tags: {tags:?}");
        match self {
            Step::Command {
                command,
                args,
                run_at,
                mitigation,
            } => Command {
                command: command.clone(),
                args: args.clone(),
                run_at: run_at.clone(),
                mitigation: mitigation.clone(),
            },
            Step::Match { value, options } => {
                let guard = params.get(value).unwrap();
                log::debug!("obtaining guard '{guard}' for tags_ {tags:?}");

                if options.get(guard).is_some() {
                    return options.get(guard).unwrap().decide(params, tags);
                }

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
        let result = match self {
            Step::Command { command, args, .. } => {
                command.contains(k) || args.iter().any(|a| a.contains(k))
            }
            Step::Match { options, .. } => options.values().any(|s| s.contains(k)),
        };
        if result {
            log::trace!("step '{self}' contains key '{k}'");
        } else {
            log::trace!("step '{self}' does not contain key '{k}'");
        }
        result
    }

    pub(crate) fn replace(&mut self, s1: &str, s2: &str) {
        let original_step = self.clone();
        match self {
            Step::Command {
                command,
                args,
                run_at,
                ..
            } => {
                *command = command.replace(s1, s2);
                *args = args.iter().map(|arg| arg.replace(s1, s2)).collect();
                *run_at = run_at.as_ref().map(|r| r.replace(s1, s2));
            }
            Step::Match { options, .. } => {
                for step in options.values_mut() {
                    step.replace(s1, s2)
                }
            }
        }
        log::debug!("replaced step: '{}' with '{}'", original_step, self);
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

        // Find all parameters that need elaboration
        for (param, _) in params.iter().sorted_by(|a, b| b.0.len().cmp(&a.0.len())) {
            if step.contains(&format!("!{}", param)) {
                elaborates.push(param);
            }
        }
        for (tag, _) in tags.iter() {
            if step.contains(&format!("!{}", tag)) {
                elaborates.push(tag);
            }
        }
        let all_elaborations = elaborates
            .iter()
            .map(|key| {
                tags.get(*key)
                    .unwrap_or_else(|| panic!("missing tag {}", key))
                    .clone()
            })
            .multi_cartesian_product()
            .collect::<Vec<Vec<_>>>();

        for elaboration_set in all_elaborations {
            let mut step = step.clone();
            for (i, val) in elaboration_set.iter().enumerate() {
                step.replace(&format!("!{}", elaborates[i]), val);
            }
            steps.push(step);
        }

        for step in steps.iter_mut() {
            let params = params
                .iter()
                .sorted_by(|a, b| b.0.len().cmp(&a.0.len()));

            for (key, value) in params {
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
                    write!(f, "\n\t({} => {})", k, step)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct WorkloadMetadata {
    pub name: String,
    pub language: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Workload {
    pub name: String,
    pub language: String,
    pub dir: PathBuf,
    pub properties: Vec<Property>,
    pub variations: Vec<Variation>,
    pub strategies: Vec<Strategy>,
    pub build_steps: Vec<Step>,
    pub check_steps: Vec<Step>,
    pub run_step: Step,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Language {
    pub name: String,
    pub build_steps: Vec<Step>,
    pub check_steps: Vec<Step>,
    pub run_step: Step,
}
