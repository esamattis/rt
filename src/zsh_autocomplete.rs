use crate::runner::Runner;

type TaskList<'a> = Vec<(&'a str, &'a str)>;

pub enum CompletionItems<'a> {
    Files,
    Tasks(TaskList<'a>),
}

fn get_zsh_autocomplete_code(items: &CompletionItems) -> String {
    let mut out = String::new();

    let tasks = match items {
        CompletionItems::Tasks(tasks) => tasks,
        CompletionItems::Files => {
            out.push_str("_files .");
            out.push('\n');
            return out;
        }
    };

    out.push_str(r#"local -a _rt_tasks"#);
    out.push('\n');

    out.push_str(r#"_rt_tasks=( "#);
    for (name, task) in tasks {
        out.push_str(&format!("'{}:from {}' ", &zsh_escape(task), name));
    }
    out.push_str(")");
    out.push('\n');

    out.push_str(r#"_describe 'task' _rt_tasks"#);
    out.push('\n');

    return out;
}

fn get_completion_items<'a>(
    runners: &'a Vec<Box<dyn Runner>>,
    lbuffer: &str,
) -> CompletionItems<'a> {
    let lbuffer = lbuffer.split("&&").last().unwrap_or(lbuffer);
    let lbuffer = lbuffer.split(";").last().unwrap_or(lbuffer);

    let arg_count = lbuffer.split_whitespace().count();

    // rt build<space><tab> aka 'rt build '
    // or
    // rt build tar<tab> aka 'rt build tar'
    if (arg_count > 1 && lbuffer.ends_with(' ')) || arg_count > 2 {
        return CompletionItems::Files;
    }

    let mut tasks: TaskList<'a> = Vec::new();

    for runner in runners {
        for task in runner.tasks() {
            tasks.push((runner.name(), task));
        }
    }

    return CompletionItems::Tasks(tasks);
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

pub fn get_zsh_autocompletion(
    runners: &Vec<Box<dyn Runner>>,
    lbuffer: &str,
    _rbuffer: &str,
) -> String {
    let items = get_completion_items(runners, lbuffer);
    return get_zsh_autocomplete_code(&items);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    pub struct TestRunner {
        tasks: Vec<String>,
        name: String,
    }

    impl TestRunner {
        pub fn new(name: String, tasks: Vec<String>) -> Self {
            return TestRunner { tasks, name };
        }
    }

    impl Runner for TestRunner {
        fn name(&self) -> &str {
            return &self.name;
        }

        fn tasks(&self) -> &Vec<String> {
            return &self.tasks;
        }

        fn load(&mut self) -> Result<()> {
            return Ok(());
        }

        fn run(&self, _task: &str, _args: &[String]) -> Result<i32> {
            return Ok(0);
        }
    }

    #[test]
    fn test_get_completion_tasks() {
        let runner1 = Box::new(TestRunner::new(
            "runner1".to_string(),
            vec!["foo".to_string(), "bar".to_string()],
        ));

        let runner2 = Box::new(TestRunner::new(
            "runner2".to_string(),
            vec!["foobar".to_string()],
        ));
        let runners: Vec<Box<dyn Runner>> = vec![runner1, runner2];

        let result = get_completion_items(&runners, "rt ");
        let CompletionItems::Tasks(tasks) = result else {
            panic!("Expected CompletionItems::Tasks");
        };
        assert_eq!(tasks.len(), 3);

        // lbuffer with two words and space
        let result = get_completion_items(&runners, "rt build ");
        assert!(matches!(result, CompletionItems::Files));

        // lbuffer with three words
        let result = get_completion_items(&runners, "rt build something");
        assert!(matches!(result, CompletionItems::Files));

        // lbuffer with partial word
        let result = get_completion_items(&runners, "rt fo");

        let CompletionItems::Tasks(tasks) = result else {
            panic!("Expected CompletionItems::Tasks");
        };

        // Intentionally return bar to because zsh will do the final filtering
        assert_eq!(
            tasks,
            vec![
                ("runner1", "foo"),
                ("runner1", "bar"),
                ("runner2", "foobar")
            ]
        );
    }

    #[test]
    fn test_combined_with_other_commands() {
        let runner1 = Box::new(TestRunner::new(
            "runner1".to_string(),
            vec!["foo".to_string(), "bar".to_string()],
        ));

        let runners: Vec<Box<dyn Runner>> = vec![runner1];

        let result = get_completion_items(&runners, "ls && rt fo");
        let CompletionItems::Tasks(tasks) = result else {
            panic!("Expected CompletionItems::Tasks");
        };
        assert_eq!(tasks.len(), 2);

        let result = get_completion_items(&runners, "ls && ls && rt fo");
        let CompletionItems::Tasks(tasks) = result else {
            panic!("Expected CompletionItems::Tasks");
        };
        assert_eq!(tasks.len(), 2);

        let result = get_completion_items(&runners, "ls&&rt fo");
        let CompletionItems::Tasks(tasks) = result else {
            panic!("Expected CompletionItems::Tasks");
        };
        assert_eq!(tasks.len(), 2);

        let result = get_completion_items(&runners, "ls;rt fo");
        let CompletionItems::Tasks(tasks) = result else {
            panic!("Expected CompletionItems::Tasks");
        };
        assert_eq!(tasks.len(), 2);

        let result = get_completion_items(&runners, "ls; rt fo");
        let CompletionItems::Tasks(tasks) = result else {
            panic!("Expected CompletionItems::Tasks");
        };
        assert_eq!(tasks.len(), 2);

        let result = get_completion_items(&runners, "ls ; rt fo");
        let CompletionItems::Tasks(tasks) = result else {
            panic!("Expected CompletionItems::Tasks");
        };
        assert_eq!(tasks.len(), 2);
    }
}
