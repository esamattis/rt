use std::process::{Command, ExitStatus};

pub fn run_command(cmd: &mut Command) -> Result<ExitStatus, String> {
    let maybe_child = cmd.spawn();

    match maybe_child {
        Ok(mut child) => {
            let res = child.wait();

            match res {
                Ok(exit_status) => {
                    return Ok(exit_status);
                }
                Err(e) => {
                    return Err(format!(
                        "Failed waiting command {:?}: {}",
                        cmd.get_program(),
                        e.to_string()
                    ));
                }
            }
        }

        Err(e) => {
            return Err(format!(
                "Failed to spawn {:?}: {}",
                cmd.get_program(),
                e.to_string()
            ));
        }
    }
}
