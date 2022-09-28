use crate::utils::run_command;

use super::runner::Runner;
use serde_json::Value;
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::{fs, io::ErrorKind};

pub struct NpmRunner {
    tasks: Vec<String>,
}

impl NpmRunner {
    pub fn new() -> Self {
        return NpmRunner { tasks: Vec::new() };
    }

    fn read_package_json() -> Result<Vec<String>, String> {
        let maybe_content = fs::read_to_string("package.json");
        let mut script_names: Vec<String> = Vec::new();

        match maybe_content {
            Ok(content) => {
                let json: Value = serde_json::from_str(&content).map_err(|e| {
                    return format!("Failed to parse package.json: {:?}", e);
                })?;

                let maybe_scripts = json["scripts"].as_object();
                if let Some(scripts) = maybe_scripts {
                    for (key, value) in scripts.iter() {
                        if let Value::String(_) = value {
                            script_names.push(key.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                if ErrorKind::NotFound == e.kind() {
                    return Ok(script_names);
                }

                return Err(format!("Failed to read package.json: {}", e.to_string()));
            }
        }

        return Ok(script_names);
    }
}

impl Runner for NpmRunner {
    fn name(&self) -> &'static str {
        return "npm";
    }

    fn tasks(&self) -> &Vec<String> {
        return &self.tasks;
    }

    fn load(&mut self) -> Result<(), String> {
        let scripts = NpmRunner::read_package_json()?;
        self.tasks = scripts;
        return Ok(());
    }

    fn run(&self, task: &str, args: &[String]) -> Result<ExitStatus, String> {
        // Detect pnpm usage from the pnpm lock file. The ../../ is for
        // packges/* style monorepo
        let is_pnpm =
            Path::new("pnpm-lock.yaml").exists() || Path::new("../../pnpm-lock.yaml").exists();

        if is_pnpm {
            let mut pnpm = Command::new("pnpm");
            return run_command(pnpm.arg("run").arg(task).args(args));
        }

        let mut npm = Command::new("npm");
        return run_command(npm.arg("run").arg(task).arg("--").args(args));
    }
}
