use std::{
    env,
    io::{self},
    process,
};

mod composer;
mod envfile;
mod jakefile;
mod npm;
mod runner;
mod scripts;

use composer::ComposerRunner;
use envfile::EnvFile;
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
    let default = String::new();
    let mut args: Vec<String> = env::args().collect();

    let mut runners_env_name = args.get(1).zip(args.get(2)).and_then(|(arg1, arg2)| {
        if arg1 == "--runners-env" {
            Some(arg2.to_string())
        } else {
            None
        }
    });

    if runners_env_name.is_some() {
        args.drain(1..3);
    } else {
        let binary_name = env::current_exe()
            .ok()
            .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
            .and_then(|s| s.into_string().ok())
            .unwrap_or_else(|| "rt".to_string())
            .to_uppercase();
        runners_env_name = env::var(format!("{}_RUNNERS", binary_name)).ok();
    }

    let runners_env_name = runners_env_name.unwrap_or_else(|| "RT_RUNNERS".to_string());
    let mut active_runners = env::var(&runners_env_name).unwrap_or_default();

    // Get project overrides from the .rtenv file in the current working directory
    let envfile = EnvFile::from_file(".rtenv");
    if let Ok(envfile) = envfile {
        if let Some(local) = envfile.get(&runners_env_name) {
            active_runners = local.to_string();
        }
    }

    let mut runners: Vec<Box<dyn Runner>> = Vec::new();

    for runner in active_runners.split(",") {
        let (runner, runner_arg) = runner.split_once(":").unwrap_or((runner, ""));
        match runner {
            "" => {}
            "package.json" => runners.push(Box::new(NpmRunner::new())),
            "jakefile" => runners.push(Box::new(JakeRunner::new())),
            "composer.json" => runners.push(Box::new(ComposerRunner::new())),
            "scripts" => runners.push(Box::new(ScriptsRunner::new(runner_arg.to_string()))),
            _ => eprintln!("Unknown runner configured: '{}'", runner),
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

    let arg = args.get(1).unwrap_or(&default);

    if arg == "--runners" {
        for runner in runners {
            println!("{}", runner.name());
        }
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
        if let Some(lbuffer) = args.get(2) {
            zsh_autocomplete(&runners, lbuffer);
        } else {
            return Err(
                "Using old .zshrc compdef definition. Please review your .zshrc and rt README.md"
                    .to_string(),
            );
        }

        return Ok(());
    }

    for runner in runners.iter_mut() {
        runner.load()?;
    }

    if arg == "" {
        for runner in runners {
            let mut tasks = runner.tasks().clone();
            if tasks.len() == 0 {
                continue;
            }
            tasks.sort();

            println!("#{}:", runner.name());
            for task in runner.tasks() {
                println!("  {} ", task);
            }
        }
    } else {
        run_task(&args[1..], &runners)?;
    }

    return Ok(());
}

fn run_task(args: &[String], runners: &Vec<Box<dyn Runner>>) -> Result<(), String> {
    let matching_runners: Vec<&Box<dyn Runner>> = runners
        .iter()
        .filter(|runner| runner.tasks().contains(&args[0]))
        .collect();

    let selected_runner = if matching_runners.len() > 1 {
        eprintln!("Multiple runners found for task: {}", args[0]);

        for (index, runner) in matching_runners.iter().enumerate() {
            eprintln!("  {}: {}", index + 1, runner.name());
        }

        let choice = prompt_number(
            &format!("Select runner (1-{}): ", matching_runners.len()),
            matching_runners.len(),
        )?;

        matching_runners.get(choice - 1)
    } else {
        matching_runners.get(0)
    };

    if let Some(runner) = selected_runner {
        runner.run(&args[0], &args[1..]);
        return Ok(());
    }

    return Err(format!("Unknown task '{}'", args[0]));
}

fn prompt_number(prompt: &str, max: usize) -> Result<usize, String> {
    loop {
        io::stdout()
            .write(prompt.as_bytes())
            .map_err(|e| format!("failed to write prompt to stdout: {}", e.to_string()))?;

        io::stdout()
            .flush()
            .map_err(|e| format!("failed to flush stdout {}", e.to_string()))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("failed to read stdin {}", e.to_string()))?;

        let choice = input.trim().chars().next().unwrap_or('1');
        if let Some(digit) = choice.to_digit(10) {
            let digit = digit as usize;
            if digit <= max {
                return Ok(digit);
            }
        }
        eprintln!("Invalid choice: {}", choice);
    }
}

fn zsh_autocomplete(runners: &Vec<Box<dyn Runner>>, lbuffer: &str) {
    if runners.len() == 0 {
        return;
    }

    let arg_count = lbuffer.split_whitespace().count();

    let mut out = io::stdout();

    // rt build<space><tab> aka 'rt build '
    // or
    // rt build tar<tab> aka 'rt build tar'
    if (arg_count > 1 && lbuffer.ends_with(' ')) || arg_count > 2 {
        out.strln("_files .");
    } else {
        out.strln(r#"local -a _rt_tasks"#);
        out.str(r#"_rt_tasks=("#);
        for runner in runners {
            for task in runner.tasks() {
                out.str(&format!("'{}:from {}' ", &zsh_escape(task), runner.name()));
            }
        }
        out.strln(")");
        out.strln(r#"_describe 'task' _rt_tasks"#);
    }

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
