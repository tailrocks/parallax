use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;

use crate::cli::Output;
use crate::diagnostic::{Finding, Format, Severity, render};
use crate::policy;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Selection {
    Rust,
    Ui,
    All,
}

pub(crate) fn run(root: &Path, selection: Selection, output: Output) -> Result<()> {
    let mut findings = Vec::new();
    if matches!(selection, Selection::Rust | Selection::All) {
        findings.extend(rust(root)?);
    }
    if matches!(selection, Selection::Ui | Selection::All) {
        findings.extend(ui(root)?);
    }
    let format = match output {
        Output::Human => Format::Human,
        Output::Json => Format::Json,
        Output::Github => Format::Github,
    };
    println!("{}", render(&findings, format)?);
    let errors = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    if errors > 0 {
        bail!("dependency policy found {errors} violation(s)");
    }
    Ok(())
}

fn rust(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    for (rule, program, args) in [
        ("dependencies.rust.audit", "cargo", vec!["audit"]),
        ("dependencies.rust.deny", "cargo", vec!["deny", "check"]),
        ("dependencies.rust.shear", "cargo", vec!["shear"]),
        (
            "dependencies.rust.features",
            "cargo",
            vec![
                "hack",
                "check",
                "--workspace",
                "--feature-powerset",
                "--exclude-features",
                "embed-ui,cross-release-vendored",
                "--locked",
            ],
        ),
    ] {
        if let Err(reason) = command(root, program, &args) {
            findings.push(failure(
                rule,
                "Cargo.lock",
                &reason,
                "cargo xtask dependencies --rust",
            ));
        }
    }
    findings.extend(metadata_policy(root)?);
    findings.extend(tls_policy(root)?);
    Ok(findings)
}

fn metadata_policy(root: &Path) -> Result<Vec<Finding>> {
    let metadata = MetadataCommand::new().current_dir(root).exec()?;
    let members = metadata
        .workspace_members
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    let mut findings = Vec::new();
    for package in metadata
        .packages
        .iter()
        .filter(|package| members.contains(&package.id))
    {
        if package
            .publish
            .as_ref()
            .is_none_or(|registries| !registries.is_empty())
        {
            findings.push(failure(
                "dependencies.rust.publish",
                package.manifest_path.as_str(),
                &format!("workspace package `{}` is publishable", package.name),
                "cargo xtask dependencies --rust",
            ));
        }
        for dependency in &package.dependencies {
            if dependency.req.to_string() == "*" && dependency.path.is_none() {
                findings.push(failure(
                    "dependencies.rust.wildcard",
                    package.manifest_path.as_str(),
                    &format!(
                        "dependency `{}` uses a wildcard requirement",
                        dependency.name
                    ),
                    "cargo xtask dependencies --rust",
                ));
            }
        }
    }
    Ok(findings)
}

fn tls_policy(root: &Path) -> Result<Vec<Finding>> {
    let host = capture(root, "cargo", &["tree", "--workspace", "-e", "features"])?;
    let cross = capture(
        root,
        "cargo",
        &[
            "tree",
            "-p",
            "parallax-cli",
            "--features",
            "cross-release-vendored",
            "-e",
            "features",
        ],
    )?;
    let mut findings = Vec::new();
    for forbidden in [
        "rustls v",
        "tokio-rustls v",
        "hyper-rustls v",
        "webpki-roots v",
    ] {
        if host.contains(forbidden) || cross.contains(forbidden) {
            findings.push(failure(
                "dependencies.rust.rustls",
                "Cargo.lock",
                &format!("active dependency graph contains `{forbidden}`"),
                "cargo xtask dependencies --rust",
            ));
        }
    }
    if host.contains("openssl-src v") {
        findings.push(failure(
            "dependencies.rust.host-vendored",
            "Cargo.toml",
            "host graph activates vendored OpenSSL",
            "cargo xtask dependencies --rust",
        ));
    }
    if !cross.contains("openssl-src v") {
        findings.push(failure(
            "dependencies.rust.cross-native-tls",
            "Cargo.toml",
            "cross-release graph does not activate vendored native OpenSSL",
            "cargo xtask dependencies --rust",
        ));
    }
    Ok(findings)
}

fn ui(root: &Path) -> Result<Vec<Finding>> {
    let directory = root.join("ui");
    let mut findings = Vec::new();
    if let Err(reason) = command(
        &directory,
        "bun",
        &["install", "--frozen-lockfile", "--ignore-scripts"],
    ) {
        findings.push(failure(
            "dependencies.ui.lock",
            "ui/bun.lock",
            &reason,
            "cargo xtask dependencies --ui",
        ));
    }
    match capture(&directory, "bun", &["audit", "--json"]) {
        Ok(report) if report.trim() == "{}" => {}
        Ok(report) => findings.push(failure(
            "dependencies.ui.audit",
            "ui/bun.lock",
            &format!("Bun audit reported advisories: {report}"),
            "cargo xtask dependencies --ui",
        )),
        Err(error) => findings.push(failure(
            "dependencies.ui.audit",
            "ui/bun.lock",
            &error.to_string(),
            "cargo xtask dependencies --ui",
        )),
    }
    findings.extend(ui_manifest_policy(&directory)?);
    findings.extend(ui_unused_policy(root, &directory)?);
    if let Err(reason) = command(&directory, "bun", &["pm", "untrusted"]) {
        findings.push(failure(
            "dependencies.ui.lifecycle",
            "ui/package.json",
            &reason,
            "cargo xtask dependencies --ui",
        ));
    }
    Ok(findings)
}

fn ui_unused_policy(root: &Path, directory: &Path) -> Result<Vec<Finding>> {
    let package: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("package.json"))?)?;
    let policy: toml::Value = toml::from_str(&std::fs::read_to_string(
        root.join("dependency-policy.toml"),
    )?)?;
    let reviewed = policy
        .get("ui")
        .and_then(|ui| ui.get("reviewed-non-ast"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let imported = policy::typescript_package_imports(root)?;
    let scripts = package
        .get("scripts")
        .and_then(serde_json::Value::as_object)
        .map_or_else(String::new, |scripts| {
            scripts
                .values()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join("\n")
        });
    let mut findings = Vec::new();
    for section in ["dependencies", "devDependencies"] {
        for name in package
            .get(section)
            .and_then(serde_json::Value::as_object)
            .into_iter()
            .flatten()
            .map(|(name, _)| name)
        {
            let executable = name.rsplit('/').next().unwrap_or(name);
            if !imported.contains(name)
                && !scripts.contains(executable)
                && !reviewed.contains(name.as_str())
            {
                findings.push(failure(
                    "dependencies.ui.unused",
                    "ui/package.json",
                    &format!("direct dependency `{name}` has no resolved import, script use, or reviewed non-AST exception"),
                    "cargo xtask dependencies --ui",
                ));
            }
        }
    }
    Ok(findings)
}

fn ui_manifest_policy(directory: &Path) -> Result<Vec<Finding>> {
    let package_path = directory.join("package.json");
    let package: serde_json::Value = serde_json::from_slice(&std::fs::read(&package_path)?)?;
    let mut findings = Vec::new();
    if package
        .get("trustedDependencies")
        .and_then(serde_json::Value::as_array)
        .is_none_or(|trusted| !trusted.is_empty())
    {
        findings.push(failure(
            "dependencies.ui.trust",
            "ui/package.json",
            "trustedDependencies must be an explicit empty list",
            "cargo xtask dependencies --ui",
        ));
    }
    if package
        .get("packageManager")
        .and_then(serde_json::Value::as_str)
        != Some("bun@1.3.14")
    {
        findings.push(failure(
            "dependencies.ui.runtime",
            "ui/package.json",
            "packageManager must match the mise-pinned Bun",
            "cargo xtask dependencies --ui",
        ));
    }
    if let Some(scripts) = package
        .get("scripts")
        .and_then(serde_json::Value::as_object)
    {
        for (name, value) in scripts {
            let script = value.as_str().unwrap_or_default();
            if script.contains("bunx")
                && !(script.contains("--bun") && script.contains("--no-install"))
            {
                findings.push(failure(
                    "dependencies.ui.executable",
                    "ui/package.json",
                    &format!("script `{name}` permits a non-Bun or implicit executable"),
                    "cargo xtask dependencies --ui",
                ));
            }
            if script.contains("@latest") || script.contains("node ") || script.contains("npx ") {
                findings.push(failure(
                    "dependencies.ui.mutable-executable",
                    "ui/package.json",
                    &format!("script `{name}` uses a forbidden runtime or mutable executable"),
                    "cargo xtask dependencies --ui",
                ));
            }
        }
    }
    let lock = std::fs::read_to_string(directory.join("bun.lock"))?;
    if !lock.contains("\"packages\": {") || !lock.contains("sha512-") {
        findings.push(failure(
            "dependencies.ui.integrity",
            "ui/bun.lock",
            "lockfile has no registry package integrity coverage",
            "cargo xtask dependencies --ui",
        ));
    }
    for forbidden in [
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "pnpm-workspace.yaml",
        ".npmrc",
    ] {
        if directory.join(forbidden).exists() {
            findings.push(failure(
                "dependencies.ui.foreign-lock",
                &format!("ui/{forbidden}"),
                "foreign package-manager state is forbidden",
                "cargo xtask dependencies --ui",
            ));
        }
    }
    Ok(findings)
}

fn command(directory: &Path, program: &str, args: &[&str]) -> std::result::Result<(), String> {
    capture(directory, program, args)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn capture(directory: &Path, program: &str, args: &[&str]) -> Result<String> {
    eprintln!("==> {program} {}", args.join(" "));
    let output = Command::new(program)
        .args(args)
        .current_dir(directory)
        .output()
        .with_context(|| format!("failed to start {program}"))?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "{program} {} exited with {}: {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn failure(rule: &str, file: &str, reason: &str, rerun: &str) -> Finding {
    Finding::error(
        rule,
        file,
        1,
        reason,
        "restore the reviewed dependency contract",
        rerun,
    )
}

#[cfg(test)]
mod tests;
