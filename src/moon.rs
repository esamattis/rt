use super::runner::Runner;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::{fs, io::ErrorKind};

#[derive(Debug, Deserialize, Serialize)]
struct MoonConfig {
    #[serde(default)]
    tasks: HashMap<String, MoonTask>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MoonTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

pub struct MoonRunner {
    tasks: Vec<String>,
}

impl MoonRunner {
    pub fn new() -> Self {
        return MoonRunner { tasks: Vec::new() };
    }

    fn read_moon_yml() -> Result<Vec<String>> {
        let content = fs::read_to_string("moon.yml");
        let mut task_names: Vec<String> = Vec::new();

        let content = match content {
            Ok(content) => content,
            Err(e) => {
                if ErrorKind::NotFound == e.kind() {
                    return Ok(task_names);
                }

                bail!(e);
            }
        };

        let config: MoonConfig =
            serde_yaml::from_str(&content).context("Failed to parse moon.yml")?;

        for (key, _value) in config.tasks.iter() {
            task_names.push(key.to_string());
        }

        return Ok(task_names);
    }
}

impl Runner for MoonRunner {
    fn name(&self) -> &'static str {
        return "moon.yml";
    }

    fn tasks(&self) -> &Vec<String> {
        return &self.tasks;
    }

    fn load(&mut self) -> Result<()> {
        let tasks = MoonRunner::read_moon_yml().context("Failed to read moon.yml")?;
        self.tasks = tasks;
        return Ok(());
    }

    fn run(&self, task: &str, args: &[String]) -> Result<i32> {
        eprintln!("[rt] Using moon");
        let mut moon = Command::new("moon");
        return self.execute(moon.arg("run").arg(task).args(args));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_moon_yml(content: &str) -> Result<Vec<String>> {
        let config: MoonConfig = serde_yaml::from_str(content)?;
        let mut task_names: Vec<String> = Vec::new();

        for (key, _value) in config.tasks.iter() {
            task_names.push(key.to_string());
        }

        Ok(task_names)
    }

    #[test]
    fn test_parse_simple_tasks() {
        let yaml = r#"
language: "rust"
type: "application"

tasks:
  build:
    description: "Build the project"
    command: "cargo build"

  test:
    description: "Run tests"
    command: "cargo test"
"#;
        let tasks = parse_moon_yml(yaml).unwrap();

        assert_eq!(tasks.len(), 2);
        assert!(tasks.contains(&"build".to_string()));
        assert!(tasks.contains(&"test".to_string()));
    }

    #[test]
    fn test_parse_complex_tasks() {
        let yaml = r#"
language: "rust"
type: "application"

fileGroups:
  sources:
    - "src/**/*"
  configs:
    - "Cargo.toml"

tasks:
  build:
    description: "Build server binary"
    command: "cargo build --bin server"
    inputs:
      - "@group(sources)"
      - "@group(configs)"
    outputs:
      - "target/debug/server"
    options:
      cache: true
      runInCI: true
    deps:
      - "ui:build"

  test-unit:
    description: "Run unit tests"
    command: "cargo nextest run"
    inputs:
      - "@group(sources)"
    options:
      cache: false

  lint:
    command: "cargo clippy"
"#;
        let tasks = parse_moon_yml(yaml).unwrap();

        assert_eq!(tasks.len(), 3);
        assert!(tasks.contains(&"build".to_string()));
        assert!(tasks.contains(&"test-unit".to_string()));
        assert!(tasks.contains(&"lint".to_string()));
    }

    #[test]
    fn test_parse_tasks_without_description() {
        let yaml = r#"
tasks:
  build:
    command: "cargo build"

  test:
    command: "cargo test"
"#;
        let tasks = parse_moon_yml(yaml).unwrap();

        assert_eq!(tasks.len(), 2);
        assert!(tasks.contains(&"build".to_string()));
        assert!(tasks.contains(&"test".to_string()));
    }

    #[test]
    fn test_parse_empty_tasks() {
        let yaml = r#"
language: "rust"
tasks: {}
"#;
        let tasks = parse_moon_yml(yaml).unwrap();

        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_parse_tasks_only() {
        let yaml = r#"
tasks:
  dev:
    command: "npm run dev"
  start:
    command: "npm start"
  deploy:
    command: "npm run deploy"
"#;
        let tasks = parse_moon_yml(yaml).unwrap();

        assert_eq!(tasks.len(), 3);
        assert!(tasks.contains(&"dev".to_string()));
        assert!(tasks.contains(&"start".to_string()));
        assert!(tasks.contains(&"deploy".to_string()));
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = r#"
this is not: valid: yaml:::
  - invalid
"#;
        let result = parse_moon_yml(yaml);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yaml_without_tasks() {
        let yaml = r#"
language: "rust"
type: "application"
fileGroups:
  sources:
    - "src/**/*"
"#;
        let tasks = parse_moon_yml(yaml).unwrap();

        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_runner_name() {
        let runner = MoonRunner::new();
        assert_eq!(runner.name(), "moon.yml");
    }

    #[test]
    fn test_runner_initial_tasks() {
        let runner = MoonRunner::new();
        assert_eq!(runner.tasks().len(), 0);
    }
}
