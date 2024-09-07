use std::{env, io, process};

mod composer;
mod jakefile;
mod npm;
mod runner;
mod scripts;
mod utils;

use composer::ComposerRunner;
use jakefile::JakeRunner;
use npm::NpmRunner;

use runner::Runner;
use scripts::ScriptsRunner;

use std::io::Write;

trait RawStringWriter {
    fn str(&mut self, str: &str) -> ();
    fn strln(&mut self, str: &str) -> ();
}

impl RawStringWriter for io::Stdout {
    fn str(&mut self, str: &str) -> () {
        let _ = self.write(str.as_bytes());
    }

    fn strln(&mut self, str: &str) -> () {
        let _ = self.write(str.as_bytes());
        let _ = self.write("\n".as_bytes());
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn rt() -> Result<(), String> {
    let binary_name = env::current_exe()
        .ok()
        .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
        .and_then(|s| s.into_string().ok())
        .unwrap_or_else(|| "rt".to_string())
        .to_uppercase();


    let default = String::new();
    let args: Vec<String> = env::args().collect();
    let arg = args.get(1).unwrap_or(&default);

    let runners_env = env::var(format!("{}_RUNNERS", binary_name)).unwrap_or_default();
    let active_runners = runners_env.split(",");

    let mut runners: Vec<Box<dyn Runner>> = Vec::new();

    for runner in active_runners {
        let (runner, runner_arg) = runner.split_once(":").unwrap_or((runner, ""));
        match runner {
            "" => {}
            "package.json" => runners.push(Box::new(NpmRunner::new())),
            "jakefile" => runners.push(Box::new(JakeRunner::new())),
            "composer.json" => runners.push(Box::new(ComposerRunner::new())),
            "scripts" => runners.push(Box::new(ScriptsRunner::new(runner_arg.to_string()))),
            _ => eprintln!("Unknown runner '{}' in RT_RUNNERS", runner),
        }
    }

    if runners.len() == 0 {
        runners.push(Box::new(NpmRunner::new()));
        runners.push(Box::new(JakeRunner::new()));
        runners.push(Box::new(ComposerRunner::new()));
        runners.push(Box::new(ScriptsRunner::new("./scripts".to_string())));
        runners.push(Box::new(ScriptsRunner::new("./tools".to_string())));
        runners.push(Box::new(ScriptsRunner::new("./bin".to_string())));
    }

    if arg == "--runners" {
        for runner in runners {
            println!("{}", runner.name());
        }
        return Ok(());
    }

    if arg == "-n" {
        runners.clear();
        let mut runner = Box::new(ScriptsRunner::new("node_modules/.bin".to_string()));
        runner.load()?;
        runners.push(runner);
        run_task(&args[2..], &runners)?;
        return Ok(());
    }

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

    let mut out = io::stdout();

    out.strln(r#"local -a _args"#);
    out.strln(r#"_args=($BUFFER)"#);
    out.strln(r#"_argument_count="${#words[@]}""#);
    out.strln("");
    out.strln(r#"if [ "$_argument_count" = "3" ] && [ "${words[2]}" = "-n" ]; then"#);
    out.strln(r#"    local -a _rt_node_tasks"#);
    out.strln(r#"    _rt_node_tasks=($(ls -1 ./node_modules/.bin))"#);
    out.strln(r#"    _describe 'node task' _rt_node_tasks"#);

    out.strln(r#"elif [ "$_argument_count" = "2" ]; then"#);
    out.strln(r#"    local -a _rt_tasks"#);
    out.str(r#"    _rt_tasks=("#);
    for runner in runners {
        for task in runner.tasks() {
            out.str(&format!("'{}:from {}' ", &zsh_escape(task), runner.name()));
        }
    }
    out.str(")");
    out.strln("");
    out.strln(r#"    _describe 'task' _rt_tasks"#);

    out.strln("else");
    out.strln("    _files .");
    out.strln("fi");

    let _ = out.flush();
}

fn zsh_escape(task: &str) -> String {
    let mut escaped = String::new();

    for a_char in task.chars() {
        if a_char == ':' {
            escaped.push('\\');
        }
        escaped.push(a_char);
    }

    return escaped;
}

fn main() {
    let res = rt();

    if let Err(e) = res {
        eprintln!("{}", e.to_string());
        process::exit(1)
    }
}
