use super::*;

/// Existing input set (plan 176). Each `hash` is the first 16 hex chars of
/// SHA-256 as returned by a live `fingerprint_explained` dump — remapping
/// every digest would still pass a==b / a!=b tests, but not these bytes.
struct GoldenFingerprint {
    error_type: &'static str,
    message: &'static str,
    frame: Option<&'static str>,
    op: Option<&'static str>,
    hash: &'static str,
}

const GOLDEN_FINGERPRINT_CORPUS: &[GoldenFingerprint] = &[
    GoldenFingerprint {
        error_type: "redis::ConnectionTimeout",
        message: "timed out connecting to redis://cache-7:6379 after 2000ms (attempt 4)",
        frame: Some("checkout::payment::authorize at src/payment.rs:184"),
        op: None,
        hash: "25d7f5a6a571b76c",
    },
    GoldenFingerprint {
        error_type: "redis::ConnectionTimeout",
        message: "timed out connecting to redis://cache-9:6379 after 1500ms (attempt 2)",
        frame: Some("checkout::payment::authorize at src/payment.rs:184"),
        op: None,
        hash: "25d7f5a6a571b76c",
    },
    GoldenFingerprint {
        error_type: "TypeA",
        message: "same message",
        frame: None,
        op: None,
        hash: "0c1cf2050469677e",
    },
    GoldenFingerprint {
        error_type: "TypeB",
        message: "same message",
        frame: None,
        op: None,
        hash: "615f6414416fbeb6",
    },
    GoldenFingerprint {
        error_type: "redis::ConnectionTimeout",
        message: "connect timed out",
        frame: Some("checkout::payment::authorize at /srv/app/src/payment.rs:184:9"),
        op: None,
        hash: "4279e8c99f1955ba",
    },
    GoldenFingerprint {
        error_type: "redis::ConnectionTimeout",
        message: "connect timed out",
        frame: Some("checkout::payment::authorize at /tmp/build/src/payment.rs:200"),
        op: None,
        hash: "4279e8c99f1955ba",
    },
    GoldenFingerprint {
        error_type: "redis::ConnectionTimeout",
        message: "connect timed out",
        frame: Some("checkout::payment::capture at /tmp/build/src/payment.rs:200"),
        op: None,
        hash: "4695917e67d80372",
    },
    GoldenFingerprint {
        error_type: "jackin::AttachFailed",
        message: "capsule attach failed for jk-qfrehkbv-holla-thearchitect uid 501:0 id a1b2c3d4",
        frame: None,
        op: None,
        hash: "761ab1e679fa4e15",
    },
    GoldenFingerprint {
        error_type: "jackin::AttachFailed",
        message: "capsule attach failed for jk-z9y8x7-holla-thearchitect uid 501:20 id de4dbeef",
        frame: None,
        op: None,
        hash: "761ab1e679fa4e15",
    },
    GoldenFingerprint {
        error_type: "redis::ConnectionTimeout",
        message: "connection timed out",
        frame: None,
        op: None,
        hash: "94f4c5cfdc5a4a73",
    },
    GoldenFingerprint {
        error_type: "redis::ConnectionTimeout",
        message: "authentication failed",
        frame: None,
        op: None,
        hash: "45ba99a9df11e725",
    },
    GoldenFingerprint {
        error_type: "jackin::AttachFailed",
        message: "capsule failed for jk-demo-one",
        frame: None,
        op: Some("capsule.attach"),
        hash: "542b5ecb7d247f92",
    },
    GoldenFingerprint {
        error_type: "jackin::AttachFailed",
        message: "capsule failed for jk-demo-two",
        frame: None,
        op: Some("capsule.attach"),
        hash: "542b5ecb7d247f92",
    },
    GoldenFingerprint {
        error_type: "jackin::AttachFailed",
        message: "capsule failed for jk-demo-two",
        frame: None,
        op: Some("capsule.detach"),
        hash: "cf5c26076cc51e51",
    },
    GoldenFingerprint {
        error_type: "log_error",
        message: "error with ANSI escapes",
        frame: None,
        op: None,
        hash: "74da7918f43ca203",
    },
    GoldenFingerprint {
        error_type: "log_error",
        message: "\u{1b}[31merror\u{1b}[0m with \u{1b}[1mANSI\u{1b}[0m escapes",
        frame: None,
        op: None,
        hash: "74da7918f43ca203",
    },
];

/// Cited by `docs/guide/grouping.md`. Pins exact 16-hex bytes and proves
/// `fingerprint_explained.hash == fingerprint() == fingerprint_with_operation(..., None)`
/// on no-op rows (with-op rows compare explained + `fingerprint_with_operation`).
#[test]
fn explained_hash_matches_fingerprint_bytes() {
    for row in GOLDEN_FINGERPRINT_CORPUS {
        let explained = fingerprint_explained(row.error_type, row.message, row.frame, row.op);
        let via_op = fingerprint_with_operation(row.error_type, row.message, row.frame, row.op);
        if row.op.is_none() {
            let via_fp = fingerprint(row.error_type, row.message, row.frame);
            let via_none = fingerprint_with_operation(row.error_type, row.message, row.frame, None);
            assert_eq!(
                (explained.hash.as_str(), via_fp.as_str(), via_none.as_str()),
                (row.hash, row.hash, row.hash),
                "{} / {}",
                row.error_type,
                row.message
            );
        } else {
            assert_eq!(
                (explained.hash.as_str(), via_op.as_str()),
                (row.hash, row.hash),
                "{} / {} / {:?}",
                row.error_type,
                row.message,
                row.op
            );
        }
    }
}

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
    assert_eq!(
        a,
        fingerprint_explained(
            "redis::ConnectionTimeout",
            "timed out connecting to redis://cache-7:6379 after 2000ms (attempt 4)",
            Some("checkout::payment::authorize at src/payment.rs:184"),
            None,
        )
        .hash
    );
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
