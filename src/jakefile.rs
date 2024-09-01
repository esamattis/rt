use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, ExitStatus};
use swc_common::{sync::Lrc, SourceMap};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::swc_ecma_ast::{CallExpr, Callee, Expr, Lit, Module};
use swc_ecma_visit::Visit;

use super::runner::Runner;
use super::utils::run_command;

struct TaskVisitor {
    tasks: Vec<String>,
}

impl TaskVisitor {
    fn tasks_from_module(module: &Module) -> Vec<String> {
        let mut visitor: TaskVisitor = TaskVisitor { tasks: Vec::new() };
        visitor.visit_module(module);
        return visitor.tasks;
    }
}

impl Visit for TaskVisitor {
    fn visit_call_expr(&mut self, n: &CallExpr) {
        if let Callee::Expr(e) = &n.callee {
            let unboxed = *e.clone();
            if let Expr::Fn(d) = unboxed {
                if let Some(ident) = d.ident {
                    if ident.sym.to_string() != "task" {
                        return;
                    }
                }
            }
        }

        let arg = n
            .args
            .get(0)
            .and_then(|a| (*a.expr.clone()).lit())
            .and_then(|literal| {
                if let Lit::Str(s) = literal {
                    return Some(s.value.to_string());
                } else {
                    return None;
                }
            });

        if let Some(value) = arg {
            self.tasks.push(value);
        }
    }
}

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

    // let code = "task('myarg'); hehe('joopa');";
    // let fm = cm.new_source_file(FileName::Custom("test.js".into()), code.into());
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
            self.tasks = TaskVisitor::tasks_from_module(&module);
        }

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

    #[test]
    fn test_parse_task_with_function() {
        let code = r#"task("ding", () => {});"#;
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
        let tasks = TaskVisitor::tasks_from_module(&module);

        assert_eq!(tasks, vec!["ding".to_string()]);
    }
}
