use std::process::{Command, ExitStatus};

pub fn run_command(cmd2: &mut Command) -> Result<ExitStatus, String> {
    let maybe_child = cmd2.spawn();

    match maybe_child {
        Ok(mut child) => {
            let res = child.wait();

            match res {
                Ok(exit_status) => {
                    return Ok(exit_status);
                }
                Err(e) => {
                    return Err(format!("Failed waiting command: {}", e.to_string()));
                }
            }
        }

        Err(e) => {
            return Err(format!("Failed to spawn: {}", e.to_string()));
        }
    }
}
