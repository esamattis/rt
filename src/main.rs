use std::error::Error;
use std::path::Path;
use std::{fs, io::ErrorKind};
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Node;
use swc_ecma_visit::Visit;

use serde_json::Value;
use swc_ecma_visit::swc_ecma_ast::{CallExpr, Module};

fn read_npm_scripts() -> Result<Vec<String>, std::io::Error> {
    let maybe_content = fs::read_to_string("package.json");
    let mut script_names: Vec<String> = Vec::new();

    match maybe_content {
        Ok(content) => {
            let json: Value = serde_json::from_str(&content)?;
            let maybe_scripts = json["scripts"].as_object();
            if let Some(scripts) = maybe_scripts {
                for (key, value) in scripts.iter() {
                    if let Value::String(_) = value {
                        script_names.push(key.to_string());
                    }
                }
            }
        }
        Err(e) => {
            if ErrorKind::NotFound == e.kind() {
                return Ok(script_names);
            }

            return Err(e);
        }
    }

    return Ok(script_names);
}

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

fn hmm() -> Result<(), String> {
    let module = parse_as_swc_module("jakefile.js")?;
    let tasks = TaskVisitor::tasks_from_module(&module);
    println!("tasks: {:?}", tasks);
    return Ok(());
}

fn main() {
    let res = hmm();

    if let Err(e) = res {
        println!("error {}", e.to_string());
    }

    // swc_main();
    // read_npm_scripts()?;
    // println!("res: {:?}", scripts);

    // Ok(())
}
