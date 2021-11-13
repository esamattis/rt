use std::{fs, io::ErrorKind};
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Node;
use swc_ecma_visit::Visit;

use serde_json::Value;
use swc_ecma_visit::swc_ecma_ast::CallExpr;

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

struct MyVisitor {}

impl Visit for MyVisitor {
    fn visit_call_expr(&mut self, n: &CallExpr, _parent: &dyn Node) {
        use swc_ecma_visit::swc_ecma_ast::{Expr::*, ExprOrSuper::*, Lit::Str};
        println!("#############");

        if let Expr(e) = &n.callee {
            let unboxed = *e.clone();

            let caller_name = unboxed.ident().and_then(|i| Some(i.sym.to_string()));

            if let Some(name) = caller_name {
                println!("CALL: {:?}", name);
            }
        }

        let arg = n.args.get(0).and_then(|a| Some(*a.expr.clone()));

        if let Some(Lit(Str(hmm))) = arg {
            println!("ARG: {:?}", hmm.value.to_string());
        }
    }
}

fn swc_main() {
    let cm: Lrc<SourceMap> = Default::default();
    // let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    // Real usage
    // let fm = cm
    //     .load_file(Path::new("test.js"))
    //     .expect("failed to load test.js");
    let code = "hello('myarg')";
    let fm = cm.new_source_file(FileName::Custom("test.js".into()), code.into());
    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(Default::default()),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        // e.into_diagnostic(&handler).emit();
        println!("compilerrrr: {:?}", e);
    }

    let _module = parser
        .parse_module()
        .map_err(|e| {
            println!("compilerrrr: {:?}", e);
            // Unrecoverable fatal error occurred
            // e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parser module");

    let mut my: MyVisitor = MyVisitor {};
    my.visit_module(&_module, &_module);

    return ();
}

fn main() -> Result<(), std::io::Error> {
    swc_main();
    read_npm_scripts()?;
    // println!("res: {:?}", scripts);

    Ok(())
}
