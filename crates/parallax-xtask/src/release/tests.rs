use std::{
    io::Read,
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;
use tar::EntryType;

use super::*;

#[test]
fn deterministic_archive_has_exact_public_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let binary = temp.path().join("input-parallax");
    std::fs::write(&binary, b"fixture-binary")?;
    let first = temp.path().join("first.tar.gz");
    let second = temp.path().join("second.tar.gz");
    archive::write(&binary, &first, 1_700_000_000)?;
    archive::write(&binary, &second, 1_700_000_000)?;
    let first_bytes = std::fs::read(&first)?;
    let second_bytes = std::fs::read(&second)?;
    if first_bytes != second_bytes {
        return Err("identical inputs produced different archives".into());
    }
    if first_bytes[..10] != [0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 2, 255] {
        return Err(format!("gzip header drifted: {:x?}", &first_bytes[..10]).into());
    }

    let mut tar = tar::Archive::new(GzDecoder::new(first_bytes.as_slice()));
    let mut entries = tar.entries()?;
    let mut entry = entries.next().ok_or("archive entry missing")??;
    let header = entry.header();
    let actual = (
        entry.path()?.to_string_lossy().into_owned(),
        header.mode()?,
        header.uid()?,
        header.gid()?,
        header.mtime()?,
        header.username()?.map(str::to_owned),
        header.groupname()?.map(str::to_owned),
        header.entry_type(),
    );
    let expected = (
        "parallax".to_string(),
        0o755,
        0,
        0,
        1_700_000_000,
        Some("root".to_string()),
        Some("root".to_string()),
        EntryType::Regular,
    );
    if actual != expected {
        return Err(format!("archive contract mismatch: {actual:?}").into());
    }
    let mut payload = Vec::new();
    entry.read_to_end(&mut payload)?;
    if payload != b"fixture-binary" {
        return Err("archive payload changed".into());
    }
    if entries.next().is_some() {
        return Err("archive contains more than one entry".into());
    }
    Ok(())
}

#[test]
fn rehearsal_promotes_one_verified_archive_and_checksum() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let binary = temp.path().join("parallax");
    std::fs::write(&binary, b"rehearsal fixture")?;
    let output_dir = temp.path().join("dist");
    let target = "x86_64-unknown-linux-gnu";
    let version = "0.1.0-preview.1+abcdef0";
    rehearse_archives(
        &binary,
        target,
        version,
        Channel::Rehearsal,
        1_700_000_000,
        &output_dir,
    )?;

    let archive = output_dir.join(format!("parallax-{version}-{target}.tar.gz"));
    let checksum = PathBuf::from(format!("{}.sha256", archive.display()));
    let digest = archive::digest(&archive)?;
    let actual = (
        archive.is_file(),
        std::fs::read_to_string(checksum)? == format!("{digest}\n"),
        output_dir.read_dir()?.all(|entry| {
            entry.is_ok_and(|entry| !entry.file_name().to_string_lossy().contains(".second."))
        }),
    );
    if actual != (true, true, true) {
        return Err(format!("release rehearsal contract mismatch: {actual:?}").into());
    }
    Ok(())
}

#[test]
fn rehearsal_rejects_non_release_and_oversized_binaries() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let target = host_target()?;
    let version = env!("CARGO_PKG_VERSION");
    let invalid = temp.path().join("invalid");
    std::fs::write(&invalid, b"not an executable")?;
    rehearse(
        &invalid,
        target,
        version,
        Channel::Rehearsal,
        1_700_000_000,
        temp.path(),
    )
    .expect_err("non-object release binary must fail");

    let oversized = temp.path().join("oversized");
    std::fs::File::create(&oversized)?.set_len(MAX_RELEASE_BINARY_BYTES + 1)?;
    let error = rehearse(
        &oversized,
        target,
        version,
        Channel::Rehearsal,
        1_700_000_000,
        temp.path(),
    )
    .expect_err("oversized release binary must fail before reading");
    if !error.to_string().contains("exceeds 512 MiB") {
        return Err(error.into());
    }
    Ok(())
}

fn host_target() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-gnu"),
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        pair => Err(format!("unsupported test host {pair:?}")),
    }
}

#[test]
fn identity_rejects_unsupported_targets_and_ambiguous_versions() -> Result<(), String> {
    let actual = [
        validate_identity("x86_64-unknown-linux-gnu", "0.1.0").is_ok(),
        validate_identity("powerpc-unknown-linux-gnu", "0.1.0").is_err(),
        validate_identity("x86_64-unknown-linux-gnu", "v0.1.0").is_err(),
        validate_identity("x86_64-unknown-linux-gnu", "0.1.0 bad").is_err(),
        validate_identity("x86_64-unknown-linux-gnu", "0.1").is_err(),
        validate_identity("x86_64-unknown-linux-gnu", "0.1.0;echo").is_err(),
        validate_channel_version("0.1.0-preview.1+abcdef0", Channel::Preview).is_ok(),
        validate_channel_version("0.1.0", Channel::Stable).is_ok(),
        validate_channel_version("0.1.0-dev+abcdef0", Channel::Rehearsal).is_ok(),
        validate_channel_version("0.1.0", Channel::Preview).is_err(),
        validate_channel_version("0.1.0-preview.1+abcdef0", Channel::Stable).is_err(),
        validate_archive_name(
            Path::new("parallax-x86_64-unknown-linux-gnu.tar.gz"),
            "x86_64-unknown-linux-gnu",
            "0.1.0-preview.1+abcdef0",
            Channel::Preview,
        )
        .is_ok(),
        validate_archive_name(
            Path::new("parallax-0.1.0-x86_64-unknown-linux-gnu.tar.gz"),
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            Channel::Stable,
        )
        .is_ok(),
        validate_archive_name(
            Path::new("parallax-wrong.tar.gz"),
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            Channel::Stable,
        )
        .is_err(),
        validate_archive_name(
            Path::new("parallax-x86_64-unknown-linux-gnu.tar.gz"),
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            Channel::Stable,
        )
        .is_err(),
        validate_archive_name(
            Path::new("parallax-0.1.0-x86_64-unknown-linux-gnu.tar.gz"),
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            Channel::Preview,
        )
        .is_err(),
    ];
    if !actual.into_iter().all(|valid| valid) {
        return Err(format!("release identity validation mismatch: {actual:?}"));
    }
    Ok(())
}

#[test]
fn verification_identity_rejects_ambiguous_provenance_inputs() -> Result<(), String> {
    let archive = PathBuf::from("parallax-0.1.0-x86_64-unknown-linux-gnu.tar.gz");
    let mut spec = VerifySpec {
        archive,
        target: "x86_64-unknown-linux-gnu".to_string(),
        version: "0.1.0".to_string(),
        source_epoch: 1_700_000_000,
        source_commit: "a".repeat(40),
        source_ref: "refs/tags/v0.1.0".to_string(),
        repository: "tailrocks/parallax".to_string(),
        signer_identity:
            "https://github.com/tailrocks/parallax/.github/workflows/release.yml@refs/tags/v0.1.0"
                .to_string(),
        signer_workflow: "tailrocks/parallax/.github/workflows/release.yml".to_string(),
    };
    let valid = validate_verification_identity(&spec).is_ok();

    spec.source_commit = "A".repeat(40);
    let uppercase_commit = validate_verification_identity(&spec).is_err();
    spec.source_commit = "a".repeat(39);
    let short_commit = validate_verification_identity(&spec).is_err();
    spec.source_commit = "a".repeat(40);
    spec.source_ref = "main".to_string();
    let short_ref = validate_verification_identity(&spec).is_err();
    spec.source_ref = "refs/tags/v0.1.0".to_string();
    spec.signer_identity = "https://example.test/workflow".to_string();
    let foreign_signer = validate_verification_identity(&spec).is_err();
    spec.signer_identity =
        "https://github.com/tailrocks/parallax/.github/workflows/release.yml@refs/tags/v0.1.0"
            .to_string();
    spec.signer_workflow = "tailrocks/parallax/release.yml".to_string();
    let non_workflow_path = validate_verification_identity(&spec).is_err();
    spec.signer_workflow = "other/repository/.github/workflows/release.yml".to_string();
    let foreign_workflow = validate_verification_identity(&spec).is_err();
    spec.signer_workflow = "tailrocks/parallax/.github/workflows/release.yml".to_string();
    spec.signer_identity =
        "https://github.com/tailrocks/parallax/.github/workflows/release.yml@refs/heads/main"
            .to_string();
    let mismatched_ref = validate_verification_identity(&spec).is_err();

    let actual = (
        valid,
        uppercase_commit,
        short_commit,
        short_ref,
        foreign_signer,
        non_workflow_path,
        foreign_workflow,
        mismatched_ref,
    );
    if actual != (true, true, true, true, true, true, true, true) {
        return Err(format!(
            "verification identity validation mismatch: {actual:?}"
        ));
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| error.to_string())
}

#[test]
fn release_workflows_stay_absent_while_release_is_fail_closed() -> Result<(), String> {
    let root = workspace_root()?;
    let project = include_str!("../../../../.github/ci/project.toml");
    let rehearsal = include_str!("../../../../scripts/release.sh");
    let actual = (
        project.contains("enabled = false"),
        !root.join(".github/workflows/preview.yml").exists(),
        !root.join(".github/workflows/release.yml").exists(),
        rehearsal.contains("cargo xtask release-rehearse")
            && rehearsal.contains("--channel rehearsal")
            && rehearsal.contains("*-apple-darwin")
            && rehearsal.contains("cargo build --release")
            && rehearsal.contains("cargo zigbuild")
            && !rehearsal.contains("tar -czf")
            && !rehearsal.contains("-czf")
            && !rehearsal.contains("gh release create")
            && !rehearsal.contains("git push"),
        include_str!("../../../../mise.toml")
            .contains(&format!("syft = \"{}\"", verify::SYFT_VERSION)),
        root.join("crates/parallax-server/build.rs").exists()
            && include_str!("../../../../crates/parallax-server/build.rs").contains("_shell.html")
            && !include_str!("../../../../crates/parallax-server/build.rs").contains("bun"),
    );
    if actual != (true, true, true, true, true, true) {
        return Err(format!("fail-closed release contract mismatch: {actual:?}"));
    }
    Ok(())
}

#[test]
fn velnor_generator_pin_is_the_published_048_runtime() -> Result<(), String> {
    const PIN: &str = "048a7bdaed8240cf652127c94434e60528633dec";
    let source = include_str!("../../../../.github-gen/velnor-workflow.toml");
    let policy = include_str!("../../../../.github/workflows/ci-policy.yml");
    let project = include_str!("../../../../.github/ci/project.toml");
    let actual = [
        source.contains(&format!("revision = \"{PIN}\"")),
        source.contains("runners = \"github\""),
        source.contains("scripts/fixtures/nextest-evidence/**"),
        source.contains("crates/parallax-sentry-proxy/Dockerfile"),
        policy.contains(PIN),
        !policy.contains("b9c3156cdb88e63c11b9e595a3e694b02238c09a"),
        !project.contains("id = \"rust-nextest-evidence-fixture\""),
        !project.contains("id = \"docker-crates-parallax-sentry-proxy\""),
        project.contains("id = \"docker-bench-otlp-fanout-maple\""),
        source.contains("[[units.products]]")
            && source.contains("name = \"embedded-ui\"")
            && source.contains("task = \"build-ui-for-rust\"")
            && source.contains("producer = \"bun-ui\"")
            && source.contains("product = \"embedded-ui\""),
        include_str!("../../../../mise.toml").contains("[tasks.build-ui-for-rust]"),
        project.contains("id = \"rust-parallax-cli\"")
            && project.contains(
                "github_pr_commands = [\"mise run build-ui-for-rust\", \"cd -- 'crates/parallax-cli'",
            ),
        project.contains("id = \"rust-parallax-server\"")
            && project.contains(
                "github_pr_commands = [\"mise run build-ui-for-rust\", \"cd -- 'crates/parallax-server'",
            ),
        project.contains("\"ui/**/*.css\"") && project.contains("\"ui/public/**\""),
    ];
    if actual != [true; 14] {
        return Err(format!(
            "published Velnor pin contract mismatch: {actual:?}"
        ));
    }
    Ok(())
}
