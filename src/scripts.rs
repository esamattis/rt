use crate::utils::run_command;

use super::runner::Runner;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, ExitStatus};

pub struct ScriptsRunner {
    tasks: Vec<String>,
    dir: String,
    name: String,
}

impl ScriptsRunner {
    pub fn new(dir: String) -> Self {
        let name = format!("scripts:./{}", &dir);
        return ScriptsRunner {
            dir,
            tasks: Vec::new(),
            name,
        };
    }

    fn read_scripts(dir: &str) -> Result<Vec<String>, String> {
        let mut script_names: Vec<String> = Vec::new();

        let dir = Path::new(dir);
        if !dir.exists() {
            return Ok(script_names);
        }

        let Ok(entries) = fs::read_dir(dir) else {
            return Ok(script_names);
        };

        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };

            let path = entry.path();
            if path.is_file() {
                let is_executable = path
                    .metadata()
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false);

                if !is_executable {
                    continue;
                }

                let Some(file_name) = path.file_name() else {
                    continue;
                };

                script_names.push(file_name.to_string_lossy().to_string());
            }
        }

        return Ok(script_names);
    }
}

impl Runner for ScriptsRunner {
    fn name(&self) -> &str {
        return &self.name;
    }

    fn tasks(&self) -> &Vec<String> {
        return &self.tasks;
    }

    fn load(&mut self) -> Result<(), String> {
        self.tasks = ScriptsRunner::read_scripts(&self.dir)?;
        return Ok(());
    }

    fn run(&self, task: &str, args: &[String]) -> Result<ExitStatus, String> {
        eprintln!("[rt] Using ./{}/{}", self.dir, task);

        let fullpath = Path::new(&self.dir).join(task);
        let mut script = Command::new(fullpath);
        return run_command(script.arg(task).args(args));
    }
}
