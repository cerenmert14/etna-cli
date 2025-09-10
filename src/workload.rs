use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use itertools::Itertools as _;
use serde::{ser::SerializeStruct as _, Deserialize, Serialize};

use crate::{property::Property, strategy::Strategy};
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
    #[allow(dead_code)]
    pub(crate) mitigation: Option<String>,
    /// Environment variables to set when running the command.
    pub(crate) env: HashMap<String, String>,
}

impl From<&Command> for std::process::Command {
    fn from(cmd: &Command) -> Self {
        let mut command = std::process::Command::new(&cmd.command);
        command.args(&cmd.args).envs(&cmd.env);
        if let Some(run_at) = &cmd.run_at {
            command.current_dir(run_at);
        }
        command
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(run_at) = &self.run_at {
            write!(f, "cd {} && ", run_at)?;
        }
        write!(
            f,
            "{} {} {}",
            self.env
                .iter()
                .map(|(k, v)| format!("\"{}\"=\"{}\"", k, v))
                .collect::<Vec<_>>()
                .join(" "),
            self.command,
            self.args.join(" ")
        )
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
        #[serde(skip_serializing_if = "HashMap::is_empty", default)]
        env: HashMap<String, String>,
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
                env,
            } => Command {
                command: command.clone(),
                args: args.clone(),
                run_at: run_at.clone(),
                mitigation: mitigation.clone(),
                env: env.clone(),
            },
            Step::Match { value, options } => {
                let guard = params.get(value).unwrap();
                log::debug!("obtaining guard '{guard}' for tags_ {tags:?}");

                if let Some(step) = options.get(guard) {
                    return step.decide(params, tags);
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
                env,
                mitigation: _,
            } => {
                *command = command.replace(s1, s2);
                *args = args.iter().map(|arg| arg.replace(s1, s2)).collect();
                *run_at = run_at.as_ref().map(|r| r.replace(s1, s2));
                *env = env
                    .iter()
                    .map(|(k, v)| (k.replace(s1, s2), v.replace(s1, s2)))
                    .collect();
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
            let params = params.iter().sorted_by(|a, b| b.0.len().cmp(&a.0.len()));

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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct Steps {
    pub(crate) check: Vec<Step>,
    pub(crate) build: Vec<Step>,
    pub(crate) run: Step,
}

impl Steps {
    pub(crate) fn get_steps(config: &serde_json::Value, step_index: &str) -> Option<Vec<Step>> {
        let step = config.get(step_index);

        if let Some(step) = step {
            let steps = serde_json::from_value::<Vec<Step>>(step.clone());
            if let Ok(steps) = steps {
                return Some(steps);
            } else {
                log::debug!("Step: {}", step);
                log::debug!("Error: {}", steps.unwrap_err());
                log::error!("Failed to parse step: '{}'", step_index);
            }
        }
        None
    }

    pub(crate) fn get_step(config: &serde_json::Value, step_index: &str) -> Option<Step> {
        let step = config.get(step_index);

        if let Some(step) = step {
            let steps = serde_json::from_value::<Step>(step.clone());
            if let Ok(steps) = steps {
                return Some(steps);
            } else {
                log::error!("Failed to parse step: '{}'", step_index);
                log::debug!("Step: {}", step);
                log::debug!("Error: {}", steps.unwrap_err());
            }
        }

        log::debug!("Step '{}' not found, using default step", step_index);
        None
    }

    pub(crate) fn with_default(workload_config: &serde_json::Value, default: &Steps) -> Self {
        let check =
            Self::get_steps(workload_config, "check_steps").unwrap_or(default.check.clone());
        let build =
            Self::get_steps(workload_config, "build_steps").unwrap_or(default.build.clone());
        let run = Self::get_step(workload_config, "run_step").unwrap_or(default.run.clone());

        Self { check, build, run }
    }

    pub(crate) fn from_config(workload_config: &serde_json::Value) -> anyhow::Result<Self> {
        let check = Self::get_steps(workload_config, "check_steps")
            .context("could not find check_steps")?;
        let build = Self::get_steps(workload_config, "build_steps")
            .context("could not find build_steps")?;
        let run = Self::get_step(workload_config, "run_step").context("could not find run_step")?;

        Ok(Self { check, build, run })
    }

    pub(crate) fn from_path(path: &Path) -> anyhow::Result<Self> {
        let config_path = if path.is_dir() {
            &path.join("config").with_extension("json")
        } else {
            path
        };

        let config = serde_json::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();

        Self::from_config(&config)
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
    pub(crate) steps: Steps,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Language {
    pub name: String,
    pub(crate) steps: Steps,
}

impl Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Language", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("check_steps", &self.steps.check)?;
        state.serialize_field("build_steps", &self.steps.build)?;
        state.serialize_field("run_step", &self.steps.run)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct LanguageHelper {
            name: String,
            check_steps: Vec<Step>,
            build_steps: Vec<Step>,
            run_step: Step,
        }

        let helper = LanguageHelper::deserialize(deserializer)?;
        Ok(Language {
            name: helper.name,
            steps: Steps {
                check: helper.check_steps,
                build: helper.build_steps,
                run: helper.run_step,
            },
        })
    }
}
