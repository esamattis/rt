use std::process::ExitStatus;

pub trait Runner {
    fn name(&self) -> &str;
    fn tasks(&self) -> &Vec<String>;
    fn run(&self, task: &str) -> Result<ExitStatus, String>;
    fn load(&mut self) -> Result<(), String>;
}
