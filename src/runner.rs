use std::process::{self, Command};

pub trait Runner {
    fn name(&self) -> &str;
    fn tasks(&self) -> &Vec<String>;
    fn load(&mut self) -> Result<(), String>;
    fn run(&self, task: &str, args: &[String]) -> ();
    fn execute(&self, cmd: &mut Command) -> () {
        match cmd.spawn() {
            Ok(mut child) => {
                let res = child.wait();

                match res {
                    Ok(code) => {
                        process::exit(code.code().unwrap_or(88));
                    }
                    Err(e) => {
                        eprintln!(
                            "[rt] Failed waiting command {:?}: {}",
                            cmd.get_program(),
                            e.to_string()
                        );
                        process::exit(88);
                    }
                }
            }

            Err(e) => {
                eprintln!(
                    "[rt] Failed to spawn {:?}: {}",
                    cmd.get_program(),
                    e.to_string()
                );
                process::exit(88);
            }
        }
    }
}
