use super::runner::Runner;
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

pub struct ScriptsRunner {
    tasks: Vec<String>,
    dir: String,
    name: String,
}

impl ScriptsRunner {
    pub fn new(dir: String) -> Self {
        let name = format!("scripts:{}", &dir);
        return ScriptsRunner {
            dir,
            tasks: Vec::new(),
            name,
        };
    }

    fn read_scripts(dir: &str) -> Result<Vec<String>> {
        let mut script_names: Vec<String> = Vec::new();

        let dir = Path::new(dir);

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Ok(script_names);
                }

                anyhow::bail!(e);
            }
        };

        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };

            let path = entry.path();
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

    fn load(&mut self) -> Result<()> {
        self.tasks = ScriptsRunner::read_scripts(&self.dir)
            .with_context(|| format!("Failed to read directory {}", self.dir))?;

        return Ok(());
    }

    fn run(&self, task: &str, args: &[String]) -> Result<i32> {
        eprintln!("[rt] Running script {}/{}", self.dir, task);

        let fullpath = Path::new(&self.dir).join(task);
        let mut script = Command::new(fullpath);
        return self.execute(script.args(args));
    }
}
