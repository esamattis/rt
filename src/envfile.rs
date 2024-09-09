use std::collections::HashMap;

pub struct EnvFile {
    vars: HashMap<String, String>,
}

impl EnvFile {
    pub fn new(contents: &str) -> Self {
        let mut vars = HashMap::new();
        let lines = contents.lines();

        for line in lines {
            let parts = line.split_once('=');
            if let Some((key, value)) = parts {
                let key = key.trim().trim_start_matches("export ");
                let value = value.trim();

                let value = if (value.starts_with('\'') && value.ends_with('\''))
                    || (value.starts_with('"') && value.ends_with('"'))
                {
                    &value[1..value.len() - 1]
                } else {
                    value
                };

                vars.insert(key.to_string(), value.to_string());
            }
        }

        EnvFile { vars }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(|s| s.as_str())
    }

    pub fn from_file(file: &str) -> Result<Self, std::io::Error> {
        let contents = std::fs::read_to_string(file)?;
        Ok(EnvFile::new(&contents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_file() {
        let env_file = EnvFile::new("KEY1=value1\nKEY2=value2");

        assert_eq!(env_file.get("KEY1"), Some("value1"));
        assert_eq!(env_file.get("KEY2"), Some("value2"));
        assert_eq!(env_file.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_ignores_exports() {
        let env_file = EnvFile::new("export KEY1=value1\nexport KEY2=value2");

        assert_eq!(env_file.get("KEY1"), Some("value1"));
        assert_eq!(env_file.get("KEY2"), Some("value2"));
    }

    #[test]
    fn test_handles_quotes() {
        let env_file = EnvFile::new("KEY1=\"value1 foo\"\nKEY2='value2 bar'");

        assert_eq!(env_file.get("KEY1"), Some("value1 foo"));
        assert_eq!(env_file.get("KEY2"), Some("value2 bar"));
    }
}
