
use std::collections::HashMap;

use ast::{Expr, ExprCall, ExprName, StmtExpr, StmtFunctionDef, StmtRaise, StmtTry};
use ruff_python_parser::{self as parser};
use ruff_python_ast as ast;
mod exceptions;


#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDiagnostic {
    raises: Vec<String>,
    excepts: Vec<String>,
    child_functions: Vec<String>,
    called_functions: Vec<String>,
}
fn analyze_function_body(name: &String, body: &[ast::Stmt], store: &mut HashMap<String, FunctionDiagnostic>) {
    for stmt in body {
        match stmt {
            ast::Stmt::Raise(StmtRaise {  exc  , ..}) => {
                let exc = *(exc.clone().unwrap());
                match exc {
                    Expr::Name(ExprName { id ,.. }) => {
                        store.get_mut(name).unwrap().raises.push(id.clone());
                    },
                    Expr::Call(e) => {
                        store.get_mut(name).unwrap().raises.push(extract_from_call(&e).unwrap());
                    }
                    _ => {println!("Unknown raise: {:?}", exc);}
                };
            }
            ast::Stmt::Try(StmtTry { body, handlers, orelse, finalbody, .. }) => {
                analyze_function_body(name, body, store);
                for handler in handlers {
                    match handler {
                        ast::ExceptHandler::ExceptHandler(ast::ExceptHandlerExceptHandler{type_, body, ..}) => {
                            match *(type_.clone().unwrap()) {
                                Expr::Name(ExprName { id, .. }) => {
                                    store.get_mut(name).unwrap().excepts.push(id.clone());
                                },
                                _ => {}
                            }
                            analyze_function_body(name, body, store);
                        }
                    }
                }
                analyze_function_body(name, orelse, store);
                analyze_function_body(name, finalbody, store);
            }
            ast::Stmt::Expr(StmtExpr{value, ..}) => {
                match *(value.clone()) {
                    Expr::Call(e) => {
                        store.get_mut(name).unwrap().called_functions.push(extract_from_call(&e).unwrap());
                    }
                    _ => {}
                }

            }
            _ => {
                println!("Unknown: {:?}", stmt);
            }
        }
    }
}

fn extract_from_call(call: &ExprCall) -> Option<String> {
    match call.func.as_ref() {
        Expr::Name(ExprName { id, .. }) => Some(id.clone()),
        _ => None
    }
}
fn populate_function_diagnostics(stmts: &[ast::Stmt], map: &mut HashMap<String, FunctionDiagnostic>) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(StmtFunctionDef { name, body, .. }) => {
                if !map.contains_key(name.as_str()) {
                    map.insert(name.to_string(), FunctionDiagnostic {
                        raises: Vec::new(),
                        excepts: Vec::new(),
                        child_functions: Vec::new(),
                        called_functions: Vec::new(),
                    });
                    analyze_function_body(&name.to_string(), body, map);
                }
            }
            _ => {}
        }
    }
}

fn show_possible_raises(map: &HashMap<String, FunctionDiagnostic>) {
    for (name, diagnostic) in map {
        println!("Function: {}", name);
        let called_raises = diagnostic.called_functions.iter().filter(|f| map.contains_key(*f)).map(|f| map.get(f).unwrap().raises.clone()).flatten().collect::<Vec<String>>();
        let possible_raises = diagnostic.raises.clone().into_iter().chain(called_raises).filter(|r| !diagnostic.excepts.contains(r)).collect::<Vec<String>>();
        if !possible_raises.is_empty() {
            possible_raises.iter().for_each(|r| println!("Possible raise: {}", r));
        }
        println!("");
    }
}


fn main() {
    // let builtins = exceptions::get_builtins();
    // let runtime_error = builtins.get("RuntimeError").unwrap();
    // let exception = builtins.get("Exception").unwrap();
    // println!("{:?}", exception.is_superclass_of(runtime_error));

    let mut store: HashMap<String, FunctionDiagnostic> = HashMap::new();
    let path = std::env::args().nth(1).expect("Missing path");
    let source = std::fs::read_to_string(path).expect("Failed to read file");
    let program = parser::parse_program(source.as_str()).expect("Failed to parse program");
    // analyze(&program.body);
    populate_function_diagnostics(&program.body, &mut store);
    // println!("{:?}", store);
    show_possible_raises(&store);
}
