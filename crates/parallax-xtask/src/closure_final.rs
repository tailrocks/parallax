use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

const CLOSURE_PATHS: [&str; 8] = [
    "AGENTS.md",
    "PROJECT_STRUCTURE.md",
    "plans/107-program-closure-audits.md",
    "plans/ENGINEERING-STANDARDS.md",
    "plans/GOAL.md",
    "plans/IMPLEMENTATION.md",
    "plans/OXC-IMPLEMENTATION.md",
    "plans/README.md",
];

#[derive(Debug, Eq, PartialEq)]
struct Attestation {
    auditor: String,
    c0: String,
    c1: String,
    tree: String,
    digest: String,
}

pub(super) fn run(root: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        dry_run_fixtures()?;
        println!("closure-final passing and tampered fixtures passed");
        return Ok(());
    }
    verify_repository(root)?;
    println!("closure-final commit, evidence, tree, and plan state passed");
    Ok(())
}

fn verify_repository(root: &Path) -> Result<()> {
    ensure!(
        !root.join("plans/107-program-closure-audits.md").exists(),
        "Plan 107 still exists"
    );
    ensure!(
        !root.join("plans/GOAL.md").exists(),
        "plans/GOAL.md still exists"
    );
    let head = git(root, &["rev-parse", "HEAD"])?;
    let parent = git(root, &["rev-parse", "HEAD^"])?;
    let c0 = git(root, &["rev-parse", "HEAD^^"])?;
    let tree = git(root, &["rev-parse", "HEAD^{tree}"])?;
    let message = git(root, &["show", "-s", "--format=%B", "HEAD"])?;
    let a = trailer(&message, "Closure-Audit-A")?;
    let b = trailer(&message, "Closure-Audit-B")?;
    validate_commit_trailers(&message)?;
    validate_attestations(&a, &b, &c0, &parent, &tree)?;
    let evidence_paths = git_paths(root, &c0, &parent)?;
    let evidence_date = validate_evidence_paths(&evidence_paths)?;
    validate_paths(&git_paths(root, &parent, &head)?, &CLOSURE_PATHS, false)?;
    validate_evidence(root, &evidence_date, &c0, &a, &b)?;
    validate_program_files_retired(root)?;
    validate_remaining_plans(root)?;
    Ok(())
}

fn validate_commit_trailers(message: &str) -> Result<()> {
    let signed = message
        .lines()
        .filter(|line| line.starts_with("Signed-off-by: "))
        .count();
    let coauthors = message
        .lines()
        .filter(|line| line.starts_with("Co-authored-by: "))
        .collect::<Vec<_>>();
    ensure!(
        signed == 1,
        "closure commit must contain exactly one DCO trailer"
    );
    ensure!(
        coauthors == ["Co-authored-by: Codex <codex@openai.com>"],
        "closure commit must contain exactly the Codex co-author trailer"
    );
    Ok(())
}

fn validate_attestations(
    a: &Attestation,
    b: &Attestation,
    c0: &str,
    c1: &str,
    tree: &str,
) -> Result<()> {
    ensure!(
        a.auditor != b.auditor,
        "auditor identities are not independent"
    );
    for attestation in [a, b] {
        ensure!(attestation.c0 == c0, "auditor C0 does not match HEAD^^");
        ensure!(attestation.c1 == c1, "auditor C1 does not match HEAD^");
        ensure!(
            attestation.tree == tree,
            "auditor tree does not match HEAD tree"
        );
        ensure!(
            is_hex(&attestation.digest, 64),
            "auditor digest is not lowercase SHA-256"
        );
    }
    Ok(())
}

fn trailer(message: &str, name: &str) -> Result<Attestation> {
    let prefix = format!("{name}: ");
    let values = message
        .lines()
        .filter_map(|line| line.strip_prefix(&prefix))
        .collect::<Vec<_>>();
    ensure!(
        values.len() == 1,
        "commit must contain exactly one `{name}` trailer"
    );
    parse_attestation(values[0])
}

fn parse_attestation(value: &str) -> Result<Attestation> {
    let fields = value
        .split(';')
        .filter_map(|field| field.split_once('='))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        fields.len() == 6,
        "attestation must contain exactly six fields"
    );
    ensure!(
        fields.get("result") == Some(&"pass"),
        "auditor result is not pass"
    );
    let get = |key: &str| {
        fields
            .get(key)
            .map(|value| (*value).to_owned())
            .with_context(|| format!("attestation is missing `{key}`"))
    };
    let attestation = Attestation {
        auditor: get("auditor")?,
        c0: get("c0")?,
        c1: get("c1")?,
        tree: get("tree")?,
        digest: get("digest")?,
    };
    ensure!(
        !attestation.auditor.trim().is_empty(),
        "auditor identity is empty"
    );
    for (label, value) in [
        ("c0", &attestation.c0),
        ("c1", &attestation.c1),
        ("tree", &attestation.tree),
    ] {
        ensure!(
            is_hex(value, 40),
            "attestation `{label}` is not a full Git object ID"
        );
    }
    Ok(attestation)
}

fn validate_paths(actual: &[String], allowed: &[&str], exact: bool) -> Result<()> {
    ensure!(!actual.is_empty(), "validated diff is empty");
    if exact {
        let expected = allowed
            .iter()
            .map(|path| (*path).to_owned())
            .collect::<Vec<_>>();
        ensure!(actual == expected, "evidence diff paths differ: {actual:?}");
    } else {
        for path in actual {
            ensure!(
                allowed.contains(&path.as_str()),
                "non-mechanical closure path `{path}`"
            );
        }
    }
    Ok(())
}

fn validate_evidence_paths(actual: &[String]) -> Result<String> {
    ensure!(
        actual.len() == 4,
        "evidence commit must change exactly four paths"
    );
    let prefix = "docs/research/validation/";
    let suffixes = [
        "-active-plans-closure-a.json",
        "-active-plans-closure-a.md",
        "-active-plans-closure-b.json",
        "-active-plans-closure-b.md",
    ];
    let first = actual[0]
        .strip_prefix(prefix)
        .context("closure evidence is outside the validation directory")?;
    let date = first
        .strip_suffix(suffixes[0])
        .context("closure evidence filename is invalid")?;
    ensure!(valid_date(date), "closure evidence date is not YYYY-MM-DD");
    let expected = suffixes
        .iter()
        .map(|suffix| format!("{prefix}{date}{suffix}"))
        .collect::<Vec<_>>();
    ensure!(actual == expected, "evidence diff paths differ: {actual:?}");
    Ok(date.to_owned())
}

fn valid_date(value: &str) -> bool {
    value.len() == 10
        && value.bytes().enumerate().all(|(index, byte)| match index {
            4 | 7 => byte == b'-',
            _ => byte.is_ascii_digit(),
        })
}

fn validate_evidence(
    root: &Path,
    date: &str,
    c0: &str,
    a: &Attestation,
    b: &Attestation,
) -> Result<()> {
    for (suffix, attestation) in [("a", a), ("b", b)] {
        let path = root.join(format!(
            "docs/research/validation/{date}-active-plans-closure-{suffix}.json"
        ));
        let source = fs::read(&path)?;
        let digest = format!("{:x}", Sha256::digest(&source));
        ensure!(digest == attestation.digest, "closure packet digest drift");
        let value: serde_json::Value = serde_json::from_slice(&source)?;
        ensure!(
            value["schema_version"] == 1,
            "invalid closure packet schema"
        );
        ensure!(
            value["audited_commit"] == c0,
            "closure packet names wrong C0"
        );
        ensure!(
            value["auditor"] == attestation.auditor,
            "closure packet auditor drift"
        );
        ensure!(
            value["result"] == "pass",
            "closure packet result is not pass"
        );
    }
    Ok(())
}

fn validate_program_files_retired(root: &Path) -> Result<()> {
    for path in [
        "plans/107-program-closure-audits.md",
        "plans/ENGINEERING-STANDARDS.md",
        "plans/GOAL.md",
        "plans/IMPLEMENTATION.md",
        "plans/OXC-IMPLEMENTATION.md",
    ] {
        ensure!(
            !root.join(path).exists(),
            "program file `{path}` still exists"
        );
    }
    for (path, forbidden) in [
        (
            "AGENTS.md",
            ["plans/GOAL.md", "codex/active-plan-closure-7f3c"],
        ),
        (
            "PROJECT_STRUCTURE.md",
            ["plans/GOAL.md", "codex/active-plan-closure-7f3c"],
        ),
        (
            "plans/README.md",
            ["GOAL.md", "107-program-closure-audits.md"],
        ),
    ] {
        let source = fs::read_to_string(root.join(path))?;
        for marker in forbidden {
            ensure!(
                !source.contains(marker),
                "retired program marker `{marker}` remains in `{path}`"
            );
        }
    }
    Ok(())
}

fn validate_remaining_plans(root: &Path) -> Result<()> {
    let mut files = BTreeSet::new();
    for entry in fs::read_dir(root.join("plans"))? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.len() < 4
            || !name[..3].chars().all(|char| char.is_ascii_digit())
            || !name.ends_with(".md")
        {
            continue;
        }
        let source = fs::read_to_string(&path)?;
        ensure!(
            source.contains("- **Status**: BLOCKED"),
            "remaining plan `{name}` is not BLOCKED"
        );
        files.insert(name.to_owned());
    }
    let index = fs::read_to_string(root.join("plans/README.md"))?;
    let mut rows = BTreeSet::new();
    for line in index.lines().filter(|line| line.starts_with("| [")) {
        let Some((_, target)) = line.split_once("](") else {
            continue;
        };
        let Some((target, _)) = target.split_once(')') else {
            continue;
        };
        if target.len() >= 4
            && target[..3].chars().all(|char| char.is_ascii_digit())
            && target.ends_with(".md")
        {
            ensure!(
                line.contains("| BLOCKED"),
                "remaining plan index row `{target}` is not BLOCKED"
            );
            rows.insert(target.to_owned());
        }
    }
    ensure!(files == rows, "remaining plan file/index bijection differs");
    Ok(())
}

fn git_paths(root: &Path, from: &str, to: &str) -> Result<Vec<String>> {
    let mut paths = git(root, &["diff", "--name-only", from, to])?
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    ensure!(output.status.success(), "git {} failed", args.join(" "));
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn dry_run_fixtures() -> Result<()> {
    let sha = "0123456789abcdef0123456789abcdef01234567";
    let digest = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let a = parse_attestation(&format!(
        "auditor=source-a;c0={sha};c1={sha};tree={sha};result=pass;digest={digest}"
    ))?;
    let b = parse_attestation(&format!(
        "auditor=source-b;c0={sha};c1={sha};tree={sha};result=pass;digest={digest}"
    ))?;
    validate_attestations(&a, &b, sha, sha, sha)?;
    ensure!(
        parse_attestation(&format!(
            "auditor=source-a;c0={sha};c1={sha};tree={sha};result=fail;digest={digest}"
        ))
        .is_err(),
        "tampered result fixture passed"
    );
    ensure!(
        validate_paths(&["src/main.rs".into()], &CLOSURE_PATHS, false).is_err(),
        "tampered path fixture passed"
    );
    let trailers = "closure\n\nSigned-off-by: Codex <codex@openai.com>\nCo-authored-by: Codex <codex@openai.com>\n";
    validate_commit_trailers(trailers)?;
    ensure!(
        validate_commit_trailers(&trailers.replace("Signed-off-by: ", "")).is_err(),
        "missing DCO fixture passed"
    );
    Ok(())
}

#[cfg(test)]
mod tests;
