use std::{collections::HashMap, os::unix::fs::PermissionsExt as _, path::PathBuf};

use crate::workload::{Step, Steps};

use anyhow::Result;
use minijinja::{path_loader, Environment};
use regex::Regex;
use serde::Serialize;
use serde_json::{self as json, Map as JsonMap, Value as JsonValue};
use std::collections::HashSet;

/// Detect occurrences like `$var` or `${var}` in an arbitrary string.
fn contains_var_token(s: &str, var: &str, re_cache: &mut Vec<(String, Regex)>) -> bool {
    // Cache compiled regex per variable to avoid recompiling
    if let Some((_, re)) = re_cache.iter().find(|(v, _)| v == var) {
        return re.is_match(s);
    }
    let pat = format!(r"\$\{{?\b{}\b\}}?", regex::escape(var));
    let re = Regex::new(&pat).unwrap();
    re_cache.push((var.to_string(), re));
    re_cache.last().unwrap().1.is_match(s)
}

/// Recursively walk a Step to collect used variable names (from CANDIDATE_VARIABLES).
fn collect_used_from_step<'a>(
    step: &'a Step,
    used: &mut HashSet<&'a str>,
    re_cache: &mut Vec<(String, Regex)>,
) {
    match step {
        Step::Command {
            command,
            args,
            run_at,
            mitigation,
            env,
        } => {
            for &v in &CANDIDATE_VARIABLES {
                log::trace!("Checking command '{}' for var '{}'", command, v);
                if contains_var_token(command, v, re_cache) {
                    log::trace!("Found usage of var '{}'", v);
                    used.insert(v);
                }
            }
            // args
            for a in args {
                for &v in &CANDIDATE_VARIABLES {
                    if contains_var_token(a, v, re_cache) {
                        used.insert(v);
                    }
                }
            }
            // run_at
            if let Some(dir) = run_at {
                for &v in &CANDIDATE_VARIABLES {
                    if contains_var_token(dir, v, re_cache) {
                        used.insert(v);
                    }
                }
            }
            // mitigation text (might reference vars)
            if let Some(m) = mitigation {
                for &v in &CANDIDATE_VARIABLES {
                    if contains_var_token(m, v, re_cache) {
                        used.insert(v);
                    }
                }
            }
            // env values (keys are literal names; values may reference vars)
            for (_k, val) in env {
                for &v in &CANDIDATE_VARIABLES {
                    if contains_var_token(val, v, re_cache) {
                        used.insert(v);
                    }
                }
            }
        }
        Step::Match { value, options } => {
            // The match "value" is directly a variable name in your to_bash()
            used.insert(value.as_str());
            // Recurse into branches
            for (_opt, st) in options {
                collect_used_from_step(st, used, re_cache);
            }
        }
    }
}

/// Scan the entire Steps to find all actually-used candidate variables.
fn collect_used_variables(cfg: &Steps) -> HashSet<&str> {
    let mut used: HashSet<&str> = HashSet::new();
    let mut re_cache: Vec<(String, Regex)> = Vec::new();

    for st in &cfg.check {
        collect_used_from_step(st, &mut used, &mut re_cache);
    }
    for st in &cfg.build {
        collect_used_from_step(st, &mut used, &mut re_cache);
    }

    collect_used_from_step(&cfg.run, &mut used, &mut re_cache);

    used
}

#[derive(Serialize)]
struct VarCtx<'a> {
    name: &'a str,
}

#[derive(Serialize)]
struct TemplateCtx<'a> {
    vars: Vec<VarCtx<'a>>,
    check: Vec<String>,
    build: Vec<String>,
    run: String,
}

/// The path should point to a valid run configuration
/// This command produces a bash script `steps.sh` that executes the steps defined in the configuration
pub fn invoke(path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = path.unwrap_or_else(|| std::env::current_dir().unwrap().join("config.toml"));

    // Load full JSON config to access tags for !expansion
    let config_path = if path.is_dir() {
        path.join("config").with_extension("json")
    } else {
        path.clone()
    };
    let config_str = std::fs::read_to_string(&config_path)?;
    let config_json: JsonValue = json::from_str(&config_str)?;

    // Build steps from JSON, same as previous behavior
    let cfg = Steps::from_config(&config_json)?;

    // Extract tags for realization (expansion of !generator style tokens)
    let tags: HashMap<String, Vec<String>> = json::from_value(
        config_json
            .get("tags")
            .cloned()
            .unwrap_or(JsonValue::Object(JsonMap::new())),
    )
    .unwrap_or_default();

    // Use empty params so $vars remain for runtime CLI; only !tags expand here
    let params: HashMap<String, String> = HashMap::new();

    // Expand and pre-render the step commands into lines for the template
    let check_lines: Vec<String> = cfg
        .check
        .iter()
        .map(|s| s.realize(&params, &tags))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .map(|s| to_bash(&s, 1))
        .collect();

    let build_lines: Vec<String> = cfg
        .build
        .iter()
        .map(|s| s.realize(&params, &tags))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .map(|s| to_bash(&s, 1))
        .collect();

    let run_line: String = cfg
        .run
        .realize(&params, &tags)?
        .into_iter()
        .map(|s| to_bash(&s, 1))
        .collect::<Vec<_>>()
        .join("\n");

    // collect used candidate variables across all steps (on original cfg is sufficient)
    let used = collect_used_variables(&cfg);

    // render template
    let vars: Vec<VarCtx> = used.iter().map(|&name| VarCtx { name }).collect();

    let ctx = TemplateCtx {
        vars,
        check: check_lines,
        build: build_lines,
        run: run_line,
    };

    // 4) minijinja env + render
    let mut env = Environment::empty();
    // Load templates/… from disk
    env.set_loader(path_loader(format!(
        "{}/templates/scripts",
        std::env::var("ETNA_DIR")?
    )));
    // Disable autoescaping for shell (minijinja auto-escapes only when configured;
    // with path_loader it defaults to no autoescape, so nothing else needed here.)
    let tmpl = env.get_template("steps.sh.j2")?;
    let script = tmpl.render(ctx)?;

    std::fs::write("steps.sh", script)?;
    std::fs::set_permissions(
        "steps.sh",
        std::fs::Permissions::from_mode(0o755), // -rwxr-xr-x
    )?;
    log::info!(
        "Bash script {} generated successfully.",
        PathBuf::from("steps.sh").canonicalize()?.display()
    );
    Ok(())
}

const CANDIDATE_VARIABLES: [&str; 9] = [
    "language",
    "workload_path",
    "workload",
    "strategy",
    "property",
    "cross",
    "timeout",
    "mutations",
    "experiment_id",
];

fn to_bash(s: &Step, depth: usize) -> String {
    match s {
        Step::Command {
            command,
            args,
            run_at,
            mitigation,
            env,
        } => {
            let mut cmd = String::new();
            cmd.push_str(&" ".repeat(depth * 4));
            for (key, value) in env {
                cmd.push_str(&format!("{}=\"{}\" ", key, value));
            }
            cmd.push_str(command);
            for arg in args {
                cmd.push(' ');
                cmd.push_str(arg);
            }
            if let Some(run_at) = run_at {
                cmd = format!("(cd {} && {})", run_at, cmd);
            }

            if let Some(mitigation) = mitigation {
                cmd = format!(
                    "{} || echo \"command failed, please try: {}\"",
                    cmd, mitigation
                );
            }
            cmd.push('\n');
            cmd
        }
        Step::Match { value, options } => {
            let mut cmd = String::new();
            let mut options = options.iter();
            let (first_option, first_step) =
                options.next().expect("Match must have at least one option");
            cmd.push_str(&format!(
                "{}if [[ ${value} == \"{first_option}\" ]]; then\n",
                " ".repeat(depth * 4)
            ));
            cmd.push_str(&format!("{}", to_bash(first_step, depth + 1)));

            for (option, step) in options {
                // Check if option is equal to the value
                cmd.push_str(&format!(
                    "{}elif [[ ${value} == \"{option}\" ]]; then\n",
                    " ".repeat(depth * 4)
                ));
                cmd.push_str(&format!("{}", to_bash(step, depth + 1)));
            }
            cmd.push_str(&format!(
                "{}else\n{}echo \"Unknown option: ${value}\" >&2; usage; exit 2;\n",
                " ".repeat(depth * 4),
                " ".repeat((depth + 1) * 4)
            ));
            cmd.push_str(&format!("{}fi\n", " ".repeat(depth * 4)));
            cmd
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tabled::grid::config;

    use crate::commands::bash::invoke;

    const CONFIG_CONTENT: &str = r#"
{
    "check_steps": [],
    "build_steps": [],
    "run_step": {
        "Command": { "command": "ls" }
    }
}
    "#;

    #[test]
    fn test_bash_script_creation() {
        // We do this because there is a race condition over `steps.sh` while running
        // the invocation in parallel.
        test_invoke_creates_script();
        test_invoke_example_script();
        test_invoke_expands_generator_tags();
    }

    fn test_invoke_creates_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        fs::write(&config_path, CONFIG_CONTENT).unwrap();

        let result = invoke(Some(config_path));
        assert!(result.is_ok(), "{result:?}");

        let script = fs::read_to_string("steps.sh").unwrap();
        assert!(script.contains("#!/bin/bash"));
        // Run the script and check if the result shows `steps.sh`
        let output = std::process::Command::new("bash")
            .arg("steps.sh")
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("Run steps are completed"),
            "Unexpected output: {}\\Stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn test_invoke_example_script() {
        let config_path = format!(
            "{}/templates/configs/example.json",
            std::env::var("ETNA_DIR").unwrap()
        );
        let config_path = std::path::PathBuf::from(config_path);
        let result = invoke(Some(config_path));
        assert!(result.is_ok(), "{result:?}");
        let output = std::process::Command::new("bash")
            .arg("steps.sh")
            .args(["--choice=life", "--stages=run"])
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("it lives!"),
            "Unexpected output: {}\\Stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn test_invoke_expands_generator_tags() {
        // Config with a build step that uses !generator and tags supplying two variants
        let cfg = serde_json::json!({
            "check_steps": [],
            "build_steps": [
                { "Command": { "command": "$workload_path/build_generator", "args": ["!generator"] } }
            ],
            "run_step": { "Command": { "command": "echo", "args": ["done"] } },
            "tags": {
                "generator": ["Alpha", "Beta"]
            }
        });

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&cfg).unwrap()).unwrap();

        let result = invoke(Some(config_path));
        assert!(result.is_ok(), "{result:?}");

        let script = std::fs::read_to_string("steps.sh").unwrap();
        // Expect expansions for both tag values and no literal !generator
        assert!(script.contains("$workload_path/build_generator Alpha"));
        assert!(script.contains("$workload_path/build_generator Beta"));
        assert!(!script.contains("!generator"));
    }
}
