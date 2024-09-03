use std::{env, process};

mod jakefile;
mod npm;
mod runner;
mod utils;
use jakefile::JakeRunner;
use npm::NpmRunner;
use runner::Runner;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn rt() -> Result<(), String> {
    let default = String::new();
    let args: Vec<String> = env::args().collect();
    let arg = args.get(1).unwrap_or(&default);

    let mut runners: Vec<Box<dyn Runner>> = Vec::new();

    runners.push(Box::new(NpmRunner::new()));
    runners.push(Box::new(JakeRunner::new()));

    if arg == "--version" || arg == "-v" || arg == "-V" {
        println!("{}", VERSION);
        return Ok(());
    }

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
        run_task(&args[1..], &runners)?;
    }

    return Ok(());
}

fn run_task(args: &[String], runners: &Vec<Box<dyn Runner>>) -> Result<(), String> {
    for runner in runners {
        for task in runner.tasks() {
            if task.to_string() == args[0] {
                let res = runner.run(task, &args[1..]);

                match res {
                    Ok(exit_code) => {
                        let code = exit_code.code().unwrap_or(88);
                        if code != 0 {
                            process::exit(code);
                        } else {
                            return Ok(());
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
        }
    }

    return Err(format!("Unknown task '{}'", args[0]));
}

fn zsh_autocomplete(runners: &Vec<Box<dyn Runner>>) {
    if runners.len() == 0 {
        return;
    }

    // Generating something like:
    //   local -a _args
    //   _args=($BUFFER)
    //   _argument_count="${#words[@]}"

    //   if [ "$_argument_count" = "2" ]; then
    //       local -a _rt_tasks
    //       _rt_tasks=('comman1:from npm' 'comman2:from jake' )
    //       _describe 'task' _rt_tasks
    //   else
    //       _files .
    //   fi

    println!("local -a _args");
    println!("_args=($BUFFER)");
    println!("_argument_count=\"${{#words[@]}}\"");
    println!("");

    println!("if [ \"$_argument_count\" = \"2\" ]; then");
    println!("    local -a _rt_tasks");
    print!("    _rt_tasks=(");
    for runner in runners {
        for task in runner.tasks() {
            let mut escaped = String::new();

            for a_char in task.chars() {
                if a_char == ':' {
                    escaped.push('\\');
                }
                escaped.push(a_char);
            }

            print!("'{}:from {}' ", escaped, runner.name());
        }
    }
    print!(")");
    println!("");
    println!("    _describe 'task' _rt_tasks");

    println!("else");
    println!("    _files .");
    println!("fi");
}

fn main() {
    let res = rt();

    if let Err(e) = res {
        eprintln!("{}", e.to_string());
        process::exit(1)
    }
}
