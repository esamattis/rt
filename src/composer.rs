use super::runner::Runner;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::process::Command;
use std::{fs, io::ErrorKind};

pub struct ComposerRunner {
    tasks: Vec<String>,
}

impl ComposerRunner {
    pub fn new() -> Self {
        return ComposerRunner { tasks: Vec::new() };
    }

    fn read_composer_json() -> Result<Vec<String>> {
        let content = fs::read_to_string("composer.json");
        let mut script_names: Vec<String> = Vec::new();

        let content = match content {
            Ok(content) => content,
            Err(e) => {
                if ErrorKind::NotFound == e.kind() {
                    return Ok(script_names);
                }

                bail!(e);
            }
        };

        let json: Value = serde_json::from_str(&content).context("Failed to parse JSON")?;

        let Some(scripts) = json["scripts"].as_object() else {
            return Ok(script_names);
        };

        for (key, value) in scripts.iter() {
            if let Value::String(_) = value {
                script_names.push(key.to_string());
            }
        }

        return Ok(script_names);
    }
}

impl Runner for ComposerRunner {
    fn name(&self) -> &'static str {
        return "composer.json";
    }

    fn tasks(&self) -> &Vec<String> {
        return &self.tasks;
    }

    fn load(&mut self) -> Result<()> {
        let scripts =
            ComposerRunner::read_composer_json().context("Failed to read composer.json")?;
        self.tasks = scripts;
        return Ok(());
    }

    fn run(&self, task: &str, args: &[String]) -> Result<i32> {
        eprintln!("[rt] Using composer");
        let mut composer = Command::new("composer");
        return self.execute(composer.arg(task).arg("--").args(args));
    }
}
