use std::ops::Deref;
use std::path::Path;

use ast::PySourceType;
use ast::{visitor::walk_expr, StmtReturn, StmtTry};
use ruff_python_ast::{self as ast, ExceptHandler, Stmt};
use ruff_python_ast::visitor::{self, Visitor};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    comparable::ComparableExpr,
    helpers::{self, map_callable},
    statement_visitor::StatementVisitor,
};
use ruff_source_file::Locator;
use ruff_text_size::Ranged;

use crate::importer::Importer;
mod deferred;


pub(crate) struct RaisinPicker<'a> {
   /// The [`Path`] to the file under analysis.
   path: &'a Path,
   /// The [`Path`] to the package containing the current file.
   package: Option<&'a Path>,
   /// The module representation of the current file (e.g., `foo.bar`).
   module_path: Option<&'a [String]>,
   /// The [`PySourceType`] of the current file.
   pub(crate) source_type: PySourceType,
   /// The [`Locator`] for the current file, which enables extraction of source code from byte
   /// offsets.
   locator: &'a Locator<'a>,
   /// The [`Importer`] for the current file, which enables importing of other modules.
   importer: Importer<'a>,
   /// The [`SemanticModel`], built up over the course of the AST traversal.
   semantic: SemanticModel<'a>,
   /// A set of deferred nodes to be visited after the current traversal (e.g., function bodies).
   visit: deferred::Visit<'a>,
   /// A set of deferred nodes to be analyzed after the AST traversal (e.g., `for` loops).
   analyze: deferred::Analyze,
   /// The end offset of the last visited statement.
   last_stmt_end: TextSize,
}

#[derive(Default)]
struct RaisinFunctionBodyVisitor<'a> {
    raisins: Vec<&'a Stmt>,
    calls: Vec<&'a ast::ExprCall>,
    try_blocks: Vec<&'a StmtTry>,
}

impl<'a, 'b> Visitor<'b> for RaisinFunctionBodyVisitor<'a> 
where 'b: 'a {
    fn visit_expr(&mut self, expr: &'b ast::Expr) {
        match expr {
            ast::Expr::Call(call) => {
                self.calls.push(call);
                walk_expr(self, expr);
            },
            _ => walk_expr(self, expr),
        }
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Raise(_) => self.raisins.push(stmt),
            Stmt::Try(try_block) => {
                self.try_blocks.push(try_block);
                visitor::walk_stmt(self, stmt);
            }
            _ => visitor::walk_stmt(self, stmt),
        }
    }
}


pub(crate) fn analyze_fun(body: &[Stmt], name: &ast::Identifier) {
    let mut visitor = RaisinFunctionBodyVisitor::default();
    // visit_body(body, &mut visitor);
    
    visitor.visit_body(body);
    let raisins = visitor.raisins;
    let calls = visitor.calls;
    let try_blocks = visitor.try_blocks;
    println!("{}", name);
    println!("Raisins - {:?}", raisins);
    println!("Calls - {:?}", calls);
    println!("Try Blocks - {:?}", try_blocks);
}
// /// TRY301
// pub(crate) fn raise_within_try(checker: &mut Checker, body: &[Stmt], handlers: &[ExceptHandler]) {
//     if handlers.is_empty() {
//         return;
//     }

//     let raises = {
//         let mut visitor = RaiseStatementVisitor::default();
//         visitor.visit_body(body);
//         visitor.raises
//     };
//     // println!("{:?}", raises);
//     if raises.is_empty() {
//         return;
//     }

//     let handled_exceptions = helpers::extract_handled_exceptions(handlers);
//     let comparables: Vec<ComparableExpr> = handled_exceptions
//         .iter()
//         .map(|handler| ComparableExpr::from(*handler))
//         .collect();

//     for stmt in raises {
//         let Stmt::Raise(ast::StmtRaise {
//             exc: Some(exception),
//             ..
//         }) = stmt
//         else {
//             continue;
//         };

//         // We can't check exception sub-classes without a type-checker implementation, so let's
//         // just catch the blanket `Exception` for now.
//         if comparables.contains(&ComparableExpr::from(map_callable(exception)))
//             || handled_exceptions.iter().any(|expr| {
//                 checker
//                     .semantic()
//                     .resolve_call_path(expr)
//                     .is_some_and(|call_path| {
//                         matches!(call_path.as_slice(), ["", "Exception" | "BaseException"])
//                     })
//             })
//         {
//             checker
//                 .diagnostics
//                 .push(Diagnostic::new(RaiseWithinTry, stmt.range()));
//         }
//     }
// }
