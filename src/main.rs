use std::{fs, io::ErrorKind};

use serde_json::Value;

fn read_npm_scripts() -> Result<Vec<String>, std::io::Error> {
    let maybe_content = fs::read_to_string("package.json");
    let mut script_names: Vec<String> = Vec::new();

    match maybe_content {
        Ok(content) => {
            let json: Value = serde_json::from_str(&content)?;
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

            return Err(e);
        }
    }

    return Ok(script_names);
}

fn main() -> Result<(), std::io::Error> {
    let scripts = read_npm_scripts()?;

    println!("res: {:?}", scripts);

    Ok(())
}
