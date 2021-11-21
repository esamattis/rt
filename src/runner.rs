pub trait Runner {
    fn name(&self) -> &str;
    fn tasks(&self) -> &Vec<String>;
    fn run(&self, task: &str);
    fn load(&mut self) -> Result<(), String>;
}
