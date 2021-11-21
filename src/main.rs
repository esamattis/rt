use std::{env, process};

mod jakefile;
mod npm;
mod runner;
mod utils;
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
                    let res = runner.run(task);

                    match res {
                        Ok(exit_code) => {
                            process::exit(exit_code.code().unwrap_or(88));
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        }
        return Err(format!("Unknown task '{}'", arg));
    }

    return Ok(());
}

fn zsh_autocomplete(runners: &Vec<Box<dyn Runner>>) {
    if runners.len() == 0 {
        return;
    }

    // Generating something like:
    //      local -a _rt_tasks
    //      _rt_tasks=('ding:from npm' 'dong:from jake')")
    //      _describe 'task' _rt_tasks

    println!("local -a _rt_tasks");
    print!("_rt_tasks=(");
    for runner in runners {
        for task in runner.tasks() {
            print!("'{}:from {}' ", task, runner.name());
        }
    }
    print!(")");
    println!("");
    println!("_describe 'task' _rt_tasks");
}

fn main() {
    let res = rt();

    if let Err(e) = res {
        eprintln!("{}", e.to_string());
        process::exit(1)
    }
}
