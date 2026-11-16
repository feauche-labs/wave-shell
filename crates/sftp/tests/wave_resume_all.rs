//! Wave: «Resume» answered «for all» only fits files whose destination is a smaller start of the source. A file
//! whose destination is as big or bigger is some other file of the same name — it used to be overwritten
//! silently; now it gets its own question. Local → local.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    missing_docs
)]

use std::time::Duration;

use ibc_shell_sftp::{
    ConflictChoice, ConflictPolicy, Endpoint, TransferConfig, TransferEngine, TransferEvent,
    TransferJob,
};

#[tokio::test(flavor = "multi_thread")]
async fn resume_for_all_does_not_overwrite_a_bigger_file() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let a: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();
    std::fs::write(src.path().join("a.txt"), &a).unwrap();
    std::fs::write(dst.path().join("a.txt"), &a[..40_000]).unwrap(); // can be resumed
    std::fs::write(src.path().join("b.txt"), b"short source").unwrap();
    let theirs = vec![7u8; 50_000];
    std::fs::write(dst.path().join("b.txt"), &theirs).unwrap(); // bigger: a different file

    let engine = TransferEngine::new(TransferConfig::default());
    let mut rx = engine.subscribe();
    engine
        .enqueue(TransferJob {
            src: Endpoint::Local,
            dst: Endpoint::Local,
            items: ["a.txt", "b.txt"]
                .iter()
                .map(|n| src.path().join(n).to_string_lossy().into_owned())
                .collect(),
            dst_dir: dst.path().to_string_lossy().into_owned(),
            conflict: ConflictPolicy::Ask,
        })
        .unwrap();

    let (mut answered_a, mut asked_b_after) = (false, false);
    let mut done = 0;
    while done < 2 {
        let ev = tokio::time::timeout(Duration::from_secs(10), rx.recv())
            .await
            .expect("timeout")
            .expect("closed");
        match ev {
            TransferEvent::Conflict {
                pending_id, info, ..
            } if info.path.ends_with("a.txt") => {
                engine
                    .conflict_reply(pending_id, ConflictChoice::Resume, true)
                    .unwrap();
                answered_a = true;
            }
            // A prompt for b.txt shown before «for all» is answered by it; one shown after is the new question.
            // (An older prompt may still be in the queue after «for all» closed it: answering it is NotFound.)
            TransferEvent::Conflict { pending_id, .. } if answered_a => {
                if engine
                    .conflict_reply(pending_id, ConflictChoice::Skip, false)
                    .is_ok()
                {
                    asked_b_after = true;
                }
            }
            TransferEvent::Done { .. } => done += 1,
            TransferEvent::Failed { message, .. } => panic!("failed: {message}"),
            _ => {}
        }
    }
    assert_eq!(
        std::fs::read(dst.path().join("a.txt")).unwrap(),
        a,
        "a.txt is resumed"
    );
    assert_eq!(
        std::fs::read(dst.path().join("b.txt")).unwrap(),
        theirs,
        "b.txt was overwritten"
    );
    assert!(asked_b_after, "b.txt got no question of its own");
}
