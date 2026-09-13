use super::*;
use tempfile::tempdir;

#[test]
fn context_lifecycle_is_atomic_and_masks_token() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("contexts.toml");
    context_add_at(
        &path,
        "prod",
        "https://parallax.internal/",
        Some("super-secret-token"),
        None,
    )
    .expect("add");
    let loaded = load_contexts_at(&path).expect("load");
    assert_eq!(loaded.contexts[0].url, "https://parallax.internal");
    assert_eq!(
        loaded.contexts[0].token.as_deref(),
        Some("super-secret-token")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&path).expect("meta").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
    context_use_at(&path, "prod").expect("use");
    let resolved = resolve_context_at(None, Some(&path), None).expect("resolve");
    assert_eq!(resolved.token.as_deref(), Some("super-secret-token"));
    context_remove_at(&path, "prod").expect("remove");
    assert!(load_contexts_at(&path).expect("load").contexts.is_empty());
}

#[test]
fn env_token_overrides_context_token() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("contexts.toml");
    context_add_at(
        &path,
        "prod",
        "https://example.test",
        Some("file-token-value"),
        None,
    )
    .expect("add");
    let resolved = resolve_context_at(
        Some("prod"),
        Some(&path),
        Some("env-token-value-16".to_string()),
    )
    .expect("resolve");
    assert_eq!(resolved.token.as_deref(), Some("env-token-value-16"));
}

#[test]
fn local_context_stays_open_without_file() {
    let resolved = resolve_context_at(None, None, None).expect("local");
    assert_eq!(resolved.name, "local");
    assert!(resolved.token.is_none());
}

#[test]
fn reserved_local_name_cannot_be_added() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("contexts.toml");
    let err = context_add_at(&path, "local", "http://example", None, None).unwrap_err();
    assert!(err.to_string().contains("reserved"));
}
