use super::runner::Runner;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::Path;
use std::process::Command;
use std::{fs, io::ErrorKind};

pub struct NpmRunner {
    tasks: Vec<String>,
}

impl NpmRunner {
    pub fn new() -> Self {
        return NpmRunner { tasks: Vec::new() };
    }

    fn read_package_json() -> Result<Vec<String>> {
        let content = fs::read_to_string("package.json");
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

impl Runner for NpmRunner {
    fn name(&self) -> &'static str {
        return "package.json";
    }

    fn tasks(&self) -> &Vec<String> {
        return &self.tasks;
    }

    fn load(&mut self) -> Result<()> {
        let scripts = NpmRunner::read_package_json().context("Failed to read package.json")?;
        self.tasks = scripts;
        return Ok(());
    }

    fn run(&self, task: &str, args: &[String]) -> Result<i32> {
        // Detect pnpm usage from the pnpm lock file. The ../../ is for
        // packges/* style monorepo
        let is_pnpm =
            Path::new("pnpm-lock.yaml").exists() || Path::new("../../pnpm-lock.yaml").exists();

        if is_pnpm {
            eprintln!("[rt] Using pnpnm");
            let mut pnpm = Command::new("pnpm");
            return self.execute(pnpm.arg("run").arg(task).args(args));
        }

        let is_yarn1 = Path::new("yarn.lock").exists() || Path::new("../../yarn.lock").exists();
        if is_yarn1 {
            eprintln!("[rt] Using  yarn");
            let mut yarn = Command::new("yarn");
            return self.execute(yarn.arg("run").arg(task).args(args));
        }

        eprintln!("[rt] Using npm");
        let mut npm = Command::new("npm");
        return self.execute(npm.arg("run").arg(task).arg("--").args(args));
    }
}
