//! Wave: two files of one queue must never write into the same destination at once. Destination has `a.txt`;
//! `a.txt` (policy Rename → `a (1).txt`) and `a (1).txt` are copied together — both used to pick `a (1).txt`
//! and write the same `*.tlpart`, leaving a mix of both reported as Done. Same for `Readme.md` / `README.md`
//! on a case-insensitive disk. Local → local.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    missing_docs
)]

use std::collections::BTreeMap;
use std::time::Duration;

use ibc_shell_sftp::{
    ConflictPolicy, Endpoint, TransferConfig, TransferEngine, TransferEvent, TransferJob,
};

fn bytes(n: usize, seed: u8) -> Vec<u8> {
    (0..n)
        .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn renamed_file_does_not_land_on_a_file_of_the_same_queue() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let one = bytes(24 << 20, 1);
    let two = bytes(24 << 20, 2);
    std::fs::write(src.path().join("a.txt"), &one).unwrap();
    std::fs::write(src.path().join("a (1).txt"), &two).unwrap();
    std::fs::write(dst.path().join("a.txt"), b"old").unwrap();

    let engine = TransferEngine::new(TransferConfig::default());
    let mut rx = engine.subscribe();
    engine
        .enqueue(TransferJob {
            src: Endpoint::Local,
            dst: Endpoint::Local,
            items: ["a.txt", "a (1).txt"]
                .iter()
                .map(|n| src.path().join(n).to_string_lossy().into_owned())
                .collect(),
            dst_dir: dst.path().to_string_lossy().into_owned(),
            conflict: ConflictPolicy::Rename,
        })
        .unwrap();
    let mut ends = 0;
    while ends < 2 {
        let ev = tokio::time::timeout(Duration::from_secs(60), rx.recv())
            .await
            .expect("timeout")
            .expect("closed");
        match ev {
            TransferEvent::Done { .. } => ends += 1,
            TransferEvent::Failed { message, .. } => panic!("failed: {message}"),
            _ => {}
        }
    }
    let files: BTreeMap<String, Vec<u8>> = std::fs::read_dir(dst.path())
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            (
                e.file_name().to_string_lossy().into_owned(),
                std::fs::read(e.path()).unwrap(),
            )
        })
        .collect();
    let names: Vec<&String> = files.keys().collect();
    assert_eq!(
        files.len(),
        3,
        "expected the old file and both copies: {names:?}"
    );
    assert_eq!(files["a.txt"], b"old");
    let copies: Vec<&Vec<u8>> = files
        .iter()
        .filter(|(k, _)| *k != "a.txt")
        .map(|(_, v)| v)
        .collect();
    assert!(
        copies.contains(&&one),
        "the copy of a.txt is damaged or missing: {names:?}"
    );
    assert!(
        copies.contains(&&two),
        "the copy of «a (1).txt» is damaged or missing: {names:?}"
    );
}
