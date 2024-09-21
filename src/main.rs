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
mod zsh_autocomplete;

use anyhow::{bail, Context, Result};
use composer::ComposerRunner;
use envfile::EnvFile;
use jakefile::JakeRunner;
use npm::NpmRunner;
use runner::Runner;
use scripts::ScriptsRunner;

use std::io::Write;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn rt() -> Result<()> {
    let default = String::new();
    let mut args: Vec<String> = env::args().collect();

    let env_flag = args.get(1).zip(args.get(2));

    let runners_env_name = env_flag.and_then(|(flag, value)| {
        if flag == "--runners-env" {
            Some(value.to_string())
        } else {
            None
        }
    });

    if runners_env_name.is_some() {
        args.drain(1..3);
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
            runner.load().ok();
        }

        if let Some(lbuffer) = args.get(2) {
            let mut out = io::stdout();
            let completion = zsh_autocomplete::get_zsh_autocompletion(
                &runners,
                lbuffer,
                args.get(3).unwrap_or(&"".to_string()),
            );

            out.write(completion.as_bytes()).ok();
            out.flush().ok();
        } else {
            bail!(
                "Using old .zshrc compdef definition. Please review your .zshrc and rt README.md"
            );
        }

        return Ok(());
    }

    let mut errors: Vec<anyhow::Error> = Vec::new();

    for runner in runners.iter_mut() {
        if let Err(e) = runner.load() {
            errors.push(e.context(format!("loading runner '{}'", runner.name())));
        }
    }

    if arg == "" {
        for error in &errors {
            eprintln!("");
            print_anyhow_error(error);
            eprintln!("");
        }

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

        if errors.len() > 0 {
            bail!("Some runners failed to load");
        }
    } else {
        run_task(&args[1..], &runners)?;
    }

    return Ok(());
}

fn run_task(args: &[String], runners: &Vec<Box<dyn Runner>>) -> Result<()> {
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
        )
        .context("reading user input failed")?;

        matching_runners.get(choice - 1)
    } else {
        matching_runners.get(0)
    };

    if let Some(runner) = selected_runner {
        runner.run(&args[0], &args[1..])?;
        return Ok(());
    }

    bail!("Unknown task '{}'", args[0]);
}

fn prompt_number(prompt: &str, max: usize) -> Result<usize> {
    let mut out = io::stdout();
    loop {
        out.write(prompt.as_bytes())?;
        out.flush()?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("failed to read stdin")?;

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

fn print_anyhow_error(e: &anyhow::Error) {
    eprintln!("Error: {}", e);
    let mut source = e.source();
    while let Some(err) = source {
        eprintln!("Caused by: {}", err);
        source = err.source();
    }
}

fn main() {
    let res = rt();

    if let Err(e) = res {
        print_anyhow_error(&e);
        process::exit(1)
    }
}
