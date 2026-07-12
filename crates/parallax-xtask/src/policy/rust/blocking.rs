use std::{fs, path::Path};

use anyhow::{Context, Result};
use syn::{Expr, ExprCall, ImplItemFn, ItemFn, spanned::Spanned, visit::Visit};

use crate::diagnostic::Finding;

pub(super) fn check(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    for crate_name in [
        "parallax-api",
        "parallax-cli",
        "parallax-evidence",
        "parallax-metadata",
        "parallax-server",
        "parallax-storage",
    ] {
        collect(
            &root.join("crates").join(crate_name).join("src"),
            root,
            &mut findings,
        )?;
    }
    Ok(findings)
}

fn collect(directory: &Path, root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, root, findings)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("tests.rs" | "test_support.rs")
            ) {
                continue;
            }
            let source = fs::read_to_string(&path)?;
            let syntax = syn::parse_file(&source)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
            let mut visitor = BlockingVisitor {
                file: &relative,
                async_depth: 0,
                findings,
            };
            visitor.visit_file(&syntax);
        }
    }
    Ok(())
}

struct BlockingVisitor<'a> {
    file: &'a str,
    async_depth: usize,
    findings: &'a mut Vec<Finding>,
}

impl BlockingVisitor<'_> {
    fn visit_async_body(&mut self, is_async: bool, body: &syn::Block) {
        self.async_depth += usize::from(is_async);
        self.visit_block(body);
        self.async_depth -= usize::from(is_async);
    }
}

impl<'ast> Visit<'ast> for BlockingVisitor<'_> {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.visit_async_body(function.sig.asyncness.is_some(), &function.block);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.visit_async_body(function.sig.asyncness.is_some(), &function.block);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        let Some(path) = call_path(&call.func) else {
            syn::visit::visit_expr_call(self, call);
            return;
        };
        if path.ends_with("spawn_blocking") {
            return;
        }
        if self.async_depth > 0 && is_blocking(&path) {
            self.findings.push(Finding::error(
                "rust.async-blocking",
                self.file,
                call.span().start().line,
                &format!("blocking `{path}` call is reachable inside async code"),
                "use Tokio I/O or move the complete blocking operation behind tokio::task::spawn_blocking",
                "cargo xtask policy --only structural",
            ));
        }
        syn::visit::visit_expr_call(self, call);
    }
}

fn call_path(expression: &Expr) -> Option<String> {
    let Expr::Path(expression) = expression else {
        return None;
    };
    Some(
        expression
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
    )
}

fn is_blocking(path: &str) -> bool {
    path.starts_with("std::fs::")
        || path == "std::process::Command::new"
        || path == "std::net::TcpListener::bind"
        || path == "std::thread::sleep"
        || path.starts_with("reqwest::blocking::")
}

#[cfg(test)]
mod tests;
