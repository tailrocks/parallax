use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use proc_macro2::Span;
use rustc_lexer::{FrontmatterAllowed, TokenKind};
use syn::{
    BinOp, ExprBinary, ExprCall, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprUnsafe, ExprWhile,
    ItemMod, Macro, Meta, spanned::Spanned, visit::Visit,
};

use crate::diagnostic::Finding;

use super::config::Ratchet;

mod blocking;
mod determinism;
mod suppressions;

#[derive(Debug, Eq, PartialEq)]
struct FunctionMetric {
    line: usize,
    lines: usize,
    name: String,
    cognitive: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct FileMetric {
    logical_lines: usize,
    functions: Vec<FunctionMetric>,
    unsafe_blocks: usize,
    suppression_details: Vec<suppressions::Suppression>,
    assertions: usize,
    inline_test_modules: usize,
    determinism: determinism::Metrics,
}

pub(super) fn health(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut files = Vec::new();
    collect(&root.join("crates"), &mut files)?;
    let mut findings = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let metric =
            analyze(&source).with_context(|| format!("failed to parse {}", path.display()))?;
        let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
        let is_root = matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("lib.rs" | "main.rs")
        );
        let is_test = relative.contains("/tests/") || relative.ends_with("/tests.rs");
        let target = if is_root {
            ratchet.budgets.rust.root_file_lines
        } else if is_test {
            ratchet.budgets.rust.test_file_lines
        } else {
            ratchet.budgets.rust.production_file_lines
        };
        if metric.logical_lines > target {
            findings.push(Finding::warning(
                "health.rust.file-lines",
                &relative,
                1,
                &format!(
                    "{} logical lines exceeds target {target}",
                    metric.logical_lines
                ),
                "split the file by responsibility; required ratchets are shrink-only",
                "cargo xtask health",
            ));
        }
        for function in &metric.functions {
            if function.lines > ratchet.budgets.rust.function_lines {
                findings.push(Finding::warning(
                    "health.rust.function-lines",
                    &relative,
                    function.line,
                    &format!(
                        "{} has {} lines, target {}",
                        function.name, function.lines, ratchet.budgets.rust.function_lines
                    ),
                    "extract focused operations without refreshing the baseline",
                    "cargo xtask health",
                ));
            }
            if function.cognitive > ratchet.budgets.rust.cognitive_complexity {
                findings.push(Finding::warning(
                    "health.rust.function-cognitive-complexity",
                    &relative,
                    function.line,
                    &format!(
                        "{} has cognitive complexity {}, target {}",
                        function.name,
                        function.cognitive,
                        ratchet.budgets.rust.cognitive_complexity
                    ),
                    "reduce nested branching without refreshing the baseline",
                    "cargo xtask health",
                ));
            }
        }
        for (rule, value) in [
            ("health.rust.unsafe-blocks", metric.unsafe_blocks),
            ("health.rust.assertions", metric.assertions),
            (
                "health.rust.inline-test-modules",
                metric.inline_test_modules,
            ),
        ] {
            if value > 0 {
                findings.push(Finding::warning(
                    rule,
                    &relative,
                    1,
                    &format!("count {value} exceeds target 0"),
                    "reduce the scoped presence count; lower the ratchet after removal",
                    "cargo xtask health",
                ));
            }
        }
        if is_test {
            findings.extend(determinism::findings(&relative, &metric.determinism));
        }
    }
    Ok(findings)
}

pub(super) fn check_suppressions(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    suppressions::check(root, ratchet)
}

pub(super) fn check_async_blocking(root: &Path) -> Result<Vec<Finding>> {
    blocking::check(root)
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn analyze(source: &str) -> Result<FileMetric> {
    let syntax = syn::parse_file(source)?;
    let mut visitor = FunctionVisitor::default();
    visitor.visit_file(&syntax);
    let mut presence = PresenceVisitor::default();
    presence.visit_file(&syntax);
    Ok(FileMetric {
        logical_lines: logical_lines(source),
        functions: visitor.functions,
        unsafe_blocks: presence.unsafe_blocks,
        suppression_details: presence.suppression_details,
        assertions: presence.assertions,
        inline_test_modules: presence.inline_test_modules,
        determinism: presence.determinism,
    })
}

fn logical_lines(source: &str) -> usize {
    let mut offset = 0;
    let mut line = 0;
    let mut occupied = vec![false; source.bytes().filter(|byte| *byte == b'\n').count() + 1];
    for token in rustc_lexer::tokenize(source, FrontmatterAllowed::Yes) {
        let end = offset + token.len as usize;
        let newlines = source[offset..end]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        if !matches!(
            token.kind,
            TokenKind::Whitespace | TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
        ) {
            for occupied_line in line..=(line + newlines).min(occupied.len() - 1) {
                occupied[occupied_line] = true;
            }
        }
        line += newlines;
        offset = end;
    }
    occupied.into_iter().filter(|occupied| *occupied).count()
}

#[derive(Default)]
struct FunctionVisitor {
    functions: Vec<FunctionMetric>,
    closures: usize,
}

impl FunctionVisitor {
    fn push(&mut self, name: String, span: Span, cognitive: usize) {
        let start = span.start().line;
        let end = span.end().line;
        self.functions.push(FunctionMetric {
            line: start,
            lines: end.saturating_sub(start) + 1,
            name,
            cognitive,
        });
    }
}

impl<'ast> Visit<'ast> for FunctionVisitor {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.push(
            function.sig.ident.to_string(),
            function.span(),
            complexity(&function.block),
        );
        syn::visit::visit_item_fn(self, function);
    }
    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.push(
            function.sig.ident.to_string(),
            function.span(),
            complexity(&function.block),
        );
        syn::visit::visit_impl_item_fn(self, function);
    }
    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        self.closures += 1;
        let mut branches = BranchVisitor::default();
        branches.visit_expr(&closure.body);
        self.push(
            format!("closure#{}", self.closures),
            closure.span(),
            branches.cognitive,
        );
        syn::visit::visit_expr_closure(self, closure);
    }
}

fn complexity(block: &syn::Block) -> usize {
    let mut visitor = BranchVisitor::default();
    visitor.visit_block(block);
    visitor.cognitive
}

#[derive(Default)]
struct BranchVisitor {
    cognitive: usize,
    nesting: usize,
}

impl BranchVisitor {
    fn enter(&mut self) {
        self.cognitive += 1 + self.nesting;
        self.nesting += 1;
    }
    fn leave(&mut self) {
        self.nesting = self.nesting.saturating_sub(1);
    }
}

impl<'ast> Visit<'ast> for BranchVisitor {
    fn visit_expr_if(&mut self, expression: &'ast ExprIf) {
        self.enter();
        syn::visit::visit_expr_if(self, expression);
        self.leave();
    }
    fn visit_expr_for_loop(&mut self, expression: &'ast ExprForLoop) {
        self.enter();
        syn::visit::visit_expr_for_loop(self, expression);
        self.leave();
    }
    fn visit_expr_while(&mut self, expression: &'ast ExprWhile) {
        self.enter();
        syn::visit::visit_expr_while(self, expression);
        self.leave();
    }
    fn visit_expr_loop(&mut self, expression: &'ast ExprLoop) {
        self.enter();
        syn::visit::visit_expr_loop(self, expression);
        self.leave();
    }
    fn visit_expr_match(&mut self, expression: &'ast ExprMatch) {
        self.enter();
        self.cognitive += expression.arms.len().saturating_sub(1);
        syn::visit::visit_expr_match(self, expression);
        self.leave();
    }
    fn visit_expr_binary(&mut self, expression: &'ast ExprBinary) {
        if matches!(expression.op, BinOp::And(_) | BinOp::Or(_)) {
            self.cognitive += 1;
        }
        syn::visit::visit_expr_binary(self, expression);
    }
    fn visit_item_fn(&mut self, _function: &'ast syn::ItemFn) {}
    fn visit_impl_item_fn(&mut self, _function: &'ast syn::ImplItemFn) {}
    fn visit_expr_closure(&mut self, _closure: &'ast syn::ExprClosure) {}
}

#[derive(Default)]
struct PresenceVisitor {
    unsafe_blocks: usize,
    suppression_details: Vec<suppressions::Suppression>,
    assertions: usize,
    inline_test_modules: usize,
    determinism: determinism::Metrics,
}

impl<'ast> Visit<'ast> for PresenceVisitor {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        suppressions::collect(&attribute.meta, &mut self.suppression_details);
        syn::visit::visit_attribute(self, attribute);
    }
    fn visit_expr_unsafe(&mut self, expression: &'ast ExprUnsafe) {
        self.unsafe_blocks += 1;
        syn::visit::visit_expr_unsafe(self, expression);
    }
    fn visit_macro(&mut self, mac: &'ast Macro) {
        if mac.path.segments.last().is_some_and(|segment| {
            matches!(
                segment.ident.to_string().as_str(),
                "assert"
                    | "assert_eq"
                    | "assert_ne"
                    | "debug_assert"
                    | "debug_assert_eq"
                    | "debug_assert_ne"
            )
        }) {
            self.assertions += 1;
        }
        syn::visit::visit_macro(self, mac);
    }
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let is_test = module.attrs.iter().any(|attribute| match &attribute.meta {
            Meta::List(list) if attribute.path().is_ident("cfg") => list
                .parse_args::<Meta>()
                .is_ok_and(|meta| matches!(meta, Meta::Path(path) if path.is_ident("test"))),
            _ => false,
        });
        if is_test && module.content.is_some() {
            self.inline_test_modules += 1;
        }
        syn::visit::visit_item_mod(self, module);
    }
    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.determinism.visit_call(call);
        syn::visit::visit_expr_call(self, call);
    }
}

#[cfg(test)]
mod tests;
