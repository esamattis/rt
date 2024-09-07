use std::{
    env,
    io::{self},
    process,
};

mod composer;
mod jakefile;
mod npm;
mod runner;
mod scripts;

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
                println!("{} -- {}", task, runner.name());
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
