use std::env;

mod jakefile;
mod npm;
mod runner;
use jakefile::JakefileRunner;
use npm::NpmRunner;
use runner::Runner;

fn hmm() -> Result<(), String> {
    let default = String::new();
    let args: Vec<String> = env::args().collect();
    let arg = args.get(1).unwrap_or(&default);

    let mut runners: Vec<Box<dyn Runner>> = Vec::new();

    runners.push(Box::new(NpmRunner::new()));
    runners.push(Box::new(JakefileRunner::new()));

    for runner in runners.iter_mut() {
        runner.load()?;
    }

    if arg == "--zsh-complete" {
        zsh_autocomplete(&runners);
        return Ok(());
    }

    if arg == "" {
        for runner in runners {
            for task in runner.tasks() {
                println!("{}", task);
            }
        }
    } else {
        for runner in runners {
            for task in runner.tasks() {
                if task == arg {
                    runner.run(task);
                    return Ok(());
                }
            }
        }
        println!("No such task");
    }

    return Ok(());
}

fn zsh_autocomplete(runners: &Vec<Box<dyn Runner>>) {
    if runners.len() == 0 {
        println!("return 0");
        return;
    }

    println!("local -a rt_tasks");

    print!("rt_tasks=(");
    for runner in runners {
        for task in runner.tasks() {
            print!("'{}:from {}' ", task, runner.name());
        }
    }
    print!(")");
    println!("");

    // println!("rt_tasks=('ding:description for c command' 'dong:description for d command')");
    println!("_describe 'task' rt_tasks");
}

fn main() {
    let res = hmm();

    if let Err(e) = res {
        println!("error {}", e.to_string());
    }

    // swc_main();
    // read_npm_scripts()?;
    // println!("res: {:?}", scripts);

    // Ok(())
}
