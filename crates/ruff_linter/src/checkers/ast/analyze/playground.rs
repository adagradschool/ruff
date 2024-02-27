use std::collections::HashMap;

use ast::comparable::ComparableExpr;
use ast::{helpers, Expr, StmtRaise};
use ast::{visitor::walk_expr, StmtTry};
use ruff_python_ast::{self as ast, Stmt};
use ruff_python_ast::visitor::{self, Visitor};

#[derive(Default)]
struct RaisinVisitor<'a> {
    raises: Vec<&'a Stmt>,
    calls: Vec<&'a ast::ExprCall>,
    suppressions: Vec<&'a Expr>,
    try_blocks: Vec<&'a StmtTry>,
}

impl<'a, 'b> Visitor<'b> for RaisinVisitor<'a> 
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
            Stmt::Raise(_) => {self.raises.push(stmt);
            }
            Stmt::Try(try_block) => {
                self.try_blocks.push(try_block);
            }
            _ => visitor::walk_stmt(self, stmt),
        }
    }
}

#[derive(Debug)]
pub struct TryBlockAnalysisResult<'a> {
    calls: Vec<&'a ast::ExprCall>,
    raises: Vec<&'a Stmt>,
    suppressions: Vec<&'a Expr>,
    inner_try_block_analysis: Vec<TryBlockAnalysisResult<'a>>,
}

#[derive(Debug)]
pub struct FunctionAnalysisResult<'a> {
    calls: Vec<&'a ast::ExprCall>,
    raises: Vec<&'a Stmt>,
    try_block_analysis: Vec<TryBlockAnalysisResult<'a>>,
}

fn analyze_try_blocks<'a>(try_blocks: &[&'a StmtTry]) -> Vec<TryBlockAnalysisResult<'a>> {
    try_blocks.iter().map(|&try_block| {
        let mut calls = Vec::new();
        let mut raises = Vec::new();
        let mut suppressions = Vec::new();
        let mut inner_try_blocks = Vec::new();

        let StmtTry { body, handlers, orelse, finalbody, .. } = try_block;
        
        let mut visitor = RaisinVisitor::default();
        visitor.suppressions = helpers::extract_handled_exceptions(handlers);
        visitor.visit_body(body);
        calls.extend(visitor.calls.iter().copied());
        raises.extend(visitor.raises.iter().copied());
        inner_try_blocks.extend(analyze_try_blocks(&visitor.try_blocks));
        suppressions.extend(visitor.suppressions.iter().copied());

        handlers.iter().for_each(|handler| match handler {
            ast::ExceptHandler::ExceptHandler(ast::ExceptHandlerExceptHandler{body, ..}) => {
                let mut visitor = RaisinVisitor::default();
                visitor.visit_body(body);
                calls.extend(visitor.calls.iter().copied());
                raises.extend(visitor.raises.iter().copied());
                suppressions.extend(visitor.suppressions.iter().copied());
            }
        });

        let mut or_else_visitor = RaisinVisitor::default();
        or_else_visitor.visit_body(orelse);
        calls.extend(or_else_visitor.calls.iter().copied());
        raises.extend(or_else_visitor.raises.iter().copied());
        inner_try_blocks.extend(analyze_try_blocks(&or_else_visitor.try_blocks));

        let mut final_visitor = RaisinVisitor::default();
        final_visitor.visit_body(finalbody);
        calls.extend(final_visitor.calls.iter().copied());
        raises.extend(final_visitor.raises.iter().copied());
        inner_try_blocks.extend(analyze_try_blocks(&final_visitor.try_blocks));

        TryBlockAnalysisResult {
            calls,
            raises,
            suppressions,
            inner_try_block_analysis: inner_try_blocks,
        }
    }).collect()
}

pub(crate) fn analyze_fun(body: &[Stmt]) -> FunctionAnalysisResult {
    let mut visitor = RaisinVisitor::default();
    visitor.visit_body(body);
    FunctionAnalysisResult {
        calls: visitor.calls,
        raises: visitor.raises,
        try_block_analysis: analyze_try_blocks(&visitor.try_blocks[..]),
    }
}

// fn process_function_analysis(
//     analysis: &FunctionAnalysisResult,
//     analysis_map: &HashMap<String, FunctionAnalysisResult>,
// ) -> Vec<StmtRaise> {
//     let mut exceptions = Vec::new();

//     // Direct raises
//     for raise in &analysis.raises {
//         if let Some(exc) = &raise.as_raise_stmt() {
//             exceptions.push(exc.clone());
//         }
//     }

//     // Function calls
//     for call in &analysis.calls {
//         if let Some(called_analysis) = analysis_map.get(&call.func.id) {
//             let mut called_exceptions = process_function_analysis(called_analysis, analysis_map);
//             exceptions.extend(&mut called_exceptions);
//         }
//     }

//     // Try-Except block analysis, including nested try blocks
//     for try_block in &analysis.try_block_analysis {
//         process_try_block(try_block, &mut exceptions, analysis_map);
//     }

//     exceptions.sort();
//     exceptions.dedup();
//     exceptions
// }

// // New helper function to process a single try-except block, including any nested ones
// fn process_try_block(
//     try_block: &TryBlockAnalysisResult,
//     exceptions: &mut Vec<&StmtRaise>,
//     analysis_map: &HashMap<String, FunctionAnalysisResult>,
// ) {

//     // Process calls within the try block
//     for call in &try_block.calls {
//         if let Some(called_analysis) = analysis_map.get(&call.func.id) {
//             let mut called_exceptions = process_function_analysis(called_analysis, analysis_map);
//             exceptions.extend(&mut called_exceptions);
//         }
//     }

//     // Process raises within the try block
//     for raise in &try_block.raises {
//         if let Some(exc) = &raise.as_raise_stmt() {
//             exceptions.push(exc.clone());
//         }
//     }

//     let comparables: Vec<ComparableExpr> = try_block.suppressions
//         .iter()
//         .map(|handler| ComparableExpr::from(*handler))
//         .collect();
    
//     // Process suppressions
//     for suppression in &try_block.suppressions {
//         exceptions.retain(|exc| exc != suppression);
//     }

//     // Recursively process any inner try blocks
//     for inner_try_block in &try_block.inner_try_block_analysis {
//         process_try_block(inner_try_block, exceptions, analysis_map);
//     }
// }
