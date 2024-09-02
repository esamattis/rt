use std::borrow::Borrow;
use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, ExitStatus};
use swc_common::{sync::Lrc, SourceMap};
use swc_ecma_ast::{Callee, Expr, Lit, Module, ModuleItem, Stmt};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

use super::runner::Runner;
use super::utils::run_command;

fn parse_as_swc_module(path: &str) -> Result<Option<Module>, String> {
    let cm: Lrc<SourceMap> = Default::default();

    let maybe_file = cm.load_file(Path::new(path));

    let fm = match maybe_file {
        Ok(f) => f,
        Err(e) => {
            if ErrorKind::NotFound == e.kind() {
                return Ok(None);
            }
            return Err(format!("Failed to read {}: {}", path, e.to_string()));
        }
    };

    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(Default::default()),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    // TODO: what errors are these?
    // for e in parser.take_errors() {
    //     // e.into_diagnostic(&handler).emit();
    // }

    let module = parser.parse_module().map_err(|e| {
        return format!("Failed to parse '{}' because: {:?}", path, e);
        // Unrecoverable fatal error occurred
        // e.into_diagnostic(&handler).emit()
    })?;

    return Ok(Some(module));
}

fn get_task_fn_calls(module: &Module) -> Vec<String> {
    let mut tasks: Vec<String> = Vec::new();

    for item in module.body.iter() {
        // Is statement, eg. not an export declaration etc.
        let ModuleItem::Stmt(stmt) = item else {
            continue;
        };

        // Is expression statament, eg. not a variable declaration etc.
        let Stmt::Expr(expr) = stmt else {
            continue;
        };

        // is call experssion
        let Expr::Call(call) = expr.expr.borrow() else {
            continue;
        };

        // Is regular function call, eg. not super()  or dynamic import() call
        let Callee::Expr(expr) = &call.callee else {
            continue;
        };

        // get the caller identifier eg. "fn" from fn()
        let Expr::Ident(ident) = expr.borrow() else {
            continue;
        };

        // is task();
        if ident.sym.to_string() != "task" {
            continue;
        }

        // get the first argument
        let Some(arg) = call.args.get(0) else {
            continue;
        };

        // The first must be a literal. Eg. not a task(ding)
        let Expr::Lit(literal) = arg.expr.borrow() else {
            continue;
        };

        // It must the a string literal
        let Lit::Str(string_literal) = literal else {
            continue;
        };

        tasks.push(string_literal.value.to_string());
    }

    return tasks;
}

pub struct JakeRunner {
    tasks: Vec<String>,
}

impl JakeRunner {
    pub fn new() -> Self {
        return JakeRunner { tasks: Vec::new() };
    }
}

impl Runner for JakeRunner {
    fn name(&self) -> &'static str {
        return "jake";
    }

    fn tasks(&self) -> &Vec<String> {
        return &self.tasks;
    }

    fn load(&mut self) -> Result<(), String> {
        let maybe_module = parse_as_swc_module("jakefile.js")?;

        if let Some(module) = maybe_module {
            self.tasks = get_task_fn_calls(&module);
        };

        return Ok(());
    }

    fn run(&self, task: &str, _args: &[String]) -> Result<ExitStatus, String> {
        let mut jake = Command::new("./node_modules/.bin/jake");
        eprintln!("[rt] using jake");
        return run_command(jake.arg(task));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    fn parse_tasks(code: &str) -> Vec<String> {
        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(
            Rc::new(swc_common::FileName::Custom("test.js".into())),
            code.into(),
        );
        let lexer = Lexer::new(
            Syntax::Es(Default::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().unwrap();

        return get_task_fn_calls(&module);
    }

    #[test]
    fn test_parse_task_with_function() {
        let code = r#"task("ding", () => {});"#;
        let tasks = parse_tasks(code);

        assert_eq!(tasks, vec!["ding".to_string()]);
    }

    #[test]
    fn test_parse_multiple_tasks() {
        let code = r#"
            task("ding", () => {});
            task("dong", () => {});
        "#;
        let tasks = parse_tasks(code);

        assert_eq!(tasks, vec!["ding".to_string(), "dong".to_string()]);
    }

    #[test]
    fn test_parse_task_with_function2() {
        let code = r#"task ( "ding", aFunction);"#;
        let tasks = parse_tasks(code);

        assert_eq!(tasks, vec!["ding".to_string()]);
    }

    #[test]
    fn test_parse_task_with_async_function() {
        let code = r#"task("ding", async ()=>{});"#;
        let tasks = parse_tasks(code);

        assert_eq!(tasks, vec!["ding".to_string()]);
    }

    #[test]
    fn test_does_not_parse_wrong_functions() {
        let code = r#"wrong( "ding", aFunction);"#;
        let tasks = parse_tasks(code);

        assert!(tasks.is_empty());
    }
    #[test]
    fn test_does_not_parse_nested_tasks() {
        let code = r#"
            function aFunction() {
                task("ding", () => {});
            }
            "#;
        let tasks = parse_tasks(code);

        assert!(tasks.is_empty());
    }
}
