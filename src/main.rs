use std::{env, process};

mod jakefile;
mod npm;
mod runner;
use jakefile::JakeRunner;
use npm::NpmRunner;
use runner::Runner;

fn rt() -> Result<(), String> {
    let default = String::new();
    let args: Vec<String> = env::args().collect();
    let arg = args.get(1).unwrap_or(&default);

    let mut runners: Vec<Box<dyn Runner>> = Vec::new();

    runners.push(Box::new(NpmRunner::new()));
    runners.push(Box::new(JakeRunner::new()));

    if arg == "--zsh-complete" {
        for runner in runners.iter_mut() {
            // Silence any loading errors intentionally. We do not want to see
            // any errors when autocompleting
            let _maybe_error = runner.load();
        }
        zsh_autocomplete(&runners);
        return Ok(());
    }

    for runner in runners.iter_mut() {
        runner.load()?;
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
        eprintln!("No such task");
    }

    return Ok(());
}

fn zsh_autocomplete(runners: &Vec<Box<dyn Runner>>) {
    if runners.len() == 0 {
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
    let res = rt();

    if let Err(e) = res {
        eprintln!("{}", e.to_string());
        process::exit(1)
    }
}
