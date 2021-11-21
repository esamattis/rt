use std::error::Error;
use std::path::Path;
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Node;
use swc_ecma_visit::Visit;

use swc_ecma_visit::swc_ecma_ast::{CallExpr, Module};

use super::runner::Runner;

struct TaskVisitor {
    tasks: Vec<String>,
}

impl TaskVisitor {
    fn tasks_from_module(module: &Module) -> Vec<String> {
        let mut visitor: TaskVisitor = TaskVisitor { tasks: Vec::new() };
        visitor.visit_module(module, module);
        return visitor.tasks;
    }
}

impl Visit for TaskVisitor {
    fn visit_call_expr(&mut self, n: &CallExpr, _parent: &dyn Node) {
        use swc_ecma_visit::swc_ecma_ast::{Expr::*, ExprOrSuper::*, Lit::Str};

        if let Expr(e) = &n.callee {
            let unboxed = *e.clone();
            if let Ident(d) = unboxed {
                if d.sym.to_string() != "task" {
                    return;
                }
            }
        }

        let arg = n
            .args
            .get(0)
            .and_then(|a| (*a.expr.clone()).lit())
            .and_then(|literal| {
                if let Str(s) = literal {
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

fn parse_as_swc_module(path: &str) -> Result<Module, String> {
    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm.load_file(Path::new(path)).map_err(|op| {
        return format!("Failed to load js file from '{}' because: {:?}", path, op);
    })?;

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

    return Ok(module);
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
        let module = parse_as_swc_module("jakefile.js")?;

        let tasks = TaskVisitor::tasks_from_module(&module);

        self.tasks = tasks;

        return Ok(());
    }

    fn run(&self, task: &str) {
        println!("jake run {}", task);
    }
}
