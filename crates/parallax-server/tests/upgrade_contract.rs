//! Always-run upgrade/loss contract (plan 174). Cross-release binary
//! download lives in `upgrade_preview.rs` and is `#[ignore]`d.

use parallax_spool::{Signal, Spool};

#[tokio::test]
async fn spool_frame_count_survives_reopen() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let first = Spool::open(tmp.path()).expect("open");
    first
        .append_raw(Signal::Traces, &bytes::Bytes::from_static(b"frame-a"))
        .await
        .expect("append a");
    first
        .append_raw(Signal::Traces, &bytes::Bytes::from_static(b"frame-b"))
        .await
        .expect("append b");
    let before = first.line_count(Signal::Traces).expect("count");
    drop(first);
    let second = Spool::open(tmp.path()).expect("reopen");
    assert_eq!(second.line_count(Signal::Traces).expect("recount"), before);
    assert_eq!(before, 2);
}
