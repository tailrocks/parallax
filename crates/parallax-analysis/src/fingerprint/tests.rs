use super::*;

#[test]
fn volatile_tokens_group_together() {
    let a = fingerprint(
        "redis::ConnectionTimeout",
        "timed out connecting to redis://cache-7:6379 after 2000ms (attempt 4)",
        Some("checkout::payment::authorize at src/payment.rs:184"),
    );
    let b = fingerprint(
        "redis::ConnectionTimeout",
        "timed out connecting to redis://cache-9:6379 after 1500ms (attempt 2)",
        Some("checkout::payment::authorize at src/payment.rs:184"),
    );
    assert_eq!(a, b);
    assert_eq!(a.len(), 16);
}

#[test]
fn different_types_do_not_group() {
    let a = fingerprint("TypeA", "same message", None);
    let b = fingerprint("TypeB", "same message", None);
    assert_ne!(a, b);
}

#[test]
fn frame_line_numbers_do_not_split() {
    let a = fingerprint(
        "redis::ConnectionTimeout",
        "connect timed out",
        Some("checkout::payment::authorize at /srv/app/src/payment.rs:184:9"),
    );
    let b = fingerprint(
        "redis::ConnectionTimeout",
        "connect timed out",
        Some("checkout::payment::authorize at /tmp/build/src/payment.rs:200"),
    );
    let c = fingerprint(
        "redis::ConnectionTimeout",
        "connect timed out",
        Some("checkout::payment::capture at /tmp/build/src/payment.rs:200"),
    );
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn broadened_volatile_tokens_group_together() {
    let a = fingerprint(
        "jackin::AttachFailed",
        "capsule attach failed for jk-qfrehkbv-holla-thearchitect uid 501:0 id a1b2c3d4",
        None,
    );
    let b = fingerprint(
        "jackin::AttachFailed",
        "capsule attach failed for jk-z9y8x7-holla-thearchitect uid 501:20 id de4dbeef",
        None,
    );
    assert_eq!(a, b);
}

#[test]
fn prose_hex_words_do_not_normalize() {
    assert_eq!(
        normalize_message("deadbe prose token"),
        "deadbe prose token"
    );
    assert_eq!(normalize_message("a1b2c3 token"), "<hex> token");
}

#[test]
fn distinct_messages_without_volatile_tokens_do_not_group() {
    let a = fingerprint("redis::ConnectionTimeout", "connection timed out", None);
    let b = fingerprint("redis::ConnectionTimeout", "authentication failed", None);
    assert_ne!(a, b);
}

#[test]
fn operation_partitions_same_error_message() {
    let a = fingerprint_with_operation(
        "jackin::AttachFailed",
        "capsule failed for jk-demo-one",
        None,
        Some("capsule.attach"),
    );
    let b = fingerprint_with_operation(
        "jackin::AttachFailed",
        "capsule failed for jk-demo-two",
        None,
        Some("capsule.attach"),
    );
    let c = fingerprint_with_operation(
        "jackin::AttachFailed",
        "capsule failed for jk-demo-two",
        None,
        Some("capsule.detach"),
    );
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn ansi_colored_message_groups_and_reads_like_plain_text() {
    // D-011 family (plan 160, corpus l-bodies/e-burst): colored CLI output
    // must fingerprint identically to its plain form.
    let plain = fingerprint("log_error", "error with ANSI escapes", None);
    let colored = fingerprint(
        "log_error",
        "\u{1b}[31merror\u{1b}[0m with \u{1b}[1mANSI\u{1b}[0m escapes",
        None,
    );
    assert_eq!(plain, colored);
    assert_eq!(strip_ansi("\u{1b}[31mred\u{1b}[0m text"), "red text");
}
