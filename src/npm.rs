use std::error::Error;
use std::path::Path;
use std::{fs, io::ErrorKind};
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Node;
use swc_ecma_visit::Visit;

use serde_json::Value;
use swc_ecma_visit::swc_ecma_ast::{CallExpr, Module};

use super::runner::Runner;

pub struct NpmRunner {
    tasks: Vec<String>,
}

impl NpmRunner {
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

                return Err(e.to_string());
            }
        }

        return Ok(script_names);
    }
}

impl Runner for NpmRunner {
    fn name(&self) -> &str {
        return "npm";
    }
    fn tasks(&self) -> &Vec<String> {
        return &self.tasks;
    }

    fn new() -> Self {
        return NpmRunner { tasks: Vec::new() };
    }

    fn load(&mut self) -> Result<(), String> {
        self.tasks = Vec::new();
        match NpmRunner::read_package_json() {
            Ok(scripts) => {
                self.tasks = scripts;
            }
            Err(e) => {
                return Err(format!("Failed to read package.json: {}", e));
            }
        }

        return Ok(());
    }

    fn run(&self, task: &str) {
        println!("npm run {}", task);
    }
}
