use anyhow::{anyhow, Context, Result};
use std::process::Command;

pub trait Runner {
    fn name(&self) -> &str;
    fn tasks(&self) -> &Vec<String>;
    fn load(&mut self) -> Result<()>;
    fn run(&self, task: &str, args: &[String]) -> Result<i32>;
    fn execute(&self, cmd: &mut Command) -> Result<i32> {
        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn {:?}", cmd.get_program()))?;

        let exit_status = child
            .wait()
            .with_context(|| format!("Failed to wait for command {:?}", cmd.get_program()))?;

        return exit_status
            .code()
            .ok_or_else(|| anyhow!("Failed to get exit code for {:?}", cmd.get_program()));
    }
}
