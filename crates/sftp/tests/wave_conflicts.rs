//! Wave: «apply to all» in a conflict prompt covers every transfer of one enqueue (several selected files are
//! several transfers). Before, the answer reached only the transfer it was given in; the others waited forever.
//! Local → local, no server needed.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::time::Duration;

use ibc_shell_sftp::{
    ConflictChoice, ConflictPolicy, Endpoint, TransferConfig, TransferEngine, TransferEvent,
    TransferJob,
};

/// Three files that already exist at the destination, queued at once (three transfers), policy «Ask».
fn setup() -> (tempfile::TempDir, tempfile::TempDir, TransferJob) {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    for n in NAMES {
        std::fs::write(src.path().join(n), b"new").unwrap();
        std::fs::write(dst.path().join(n), b"old!").unwrap();
    }
    let job = TransferJob {
        src: Endpoint::Local,
        dst: Endpoint::Local,
        items: NAMES
            .iter()
            .map(|n| src.path().join(n).to_string_lossy().into_owned())
            .collect(),
        dst_dir: dst.path().to_string_lossy().into_owned(),
        conflict: ConflictPolicy::Ask,
    };
    (src, dst, job)
}

const NAMES: [&str; 3] = ["a.txt", "b.txt", "c.txt"];

#[tokio::test(flavor = "multi_thread")]
async fn apply_to_all_answers_every_file_of_the_selection() {
    let (_src, dst, job) = setup();
    let engine = TransferEngine::new(TransferConfig::default());
    let mut rx = engine.subscribe();
    engine.enqueue(job).unwrap();

    let mut answered = false;
    let (mut asked, mut closed) = (0, 0);
    let mut done = 0;
    while done < NAMES.len() {
        let ev = tokio::time::timeout(Duration::from_secs(10), rx.recv())
            .await
            .expect("a transfer hangs: «apply to all» did not reach it")
            .expect("closed");
        match ev {
            TransferEvent::Conflict { pending_id, .. } if !answered => {
                asked += 1;
                answered = true;
                engine
                    .conflict_reply(pending_id, ConflictChoice::Overwrite, true)
                    .unwrap();
            }
            TransferEvent::Conflict { .. } => asked += 1,
            TransferEvent::ConflictResolved { .. } => closed += 1,
            TransferEvent::Done { .. } => done += 1,
            TransferEvent::Failed { message, .. } => panic!("failed: {message}"),
            _ => {}
        }
    }
    // Every prompt that was shown is closed again, so none is left hanging in the UI.
    assert_eq!(asked, closed, "prompts shown {asked}, closed {closed}");
    for n in NAMES {
        assert_eq!(std::fs::read(dst.path().join(n)).unwrap(), b"new", "{n}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn without_apply_to_all_each_file_asks() {
    let (_src, dst, job) = setup();
    let engine = TransferEngine::new(TransferConfig::default());
    let mut rx = engine.subscribe();
    engine.enqueue(job).unwrap();
    let (mut asked, mut done) = (0, 0);
    while done < NAMES.len() {
        let ev = tokio::time::timeout(Duration::from_secs(10), rx.recv())
            .await
            .expect("hang")
            .expect("closed");
        match ev {
            TransferEvent::Conflict { pending_id, .. } => {
                asked += 1;
                engine
                    .conflict_reply(pending_id, ConflictChoice::Skip, false)
                    .unwrap();
            }
            TransferEvent::Done { .. } => done += 1,
            TransferEvent::Failed { message, .. } => panic!("failed: {message}"),
            _ => {}
        }
    }
    assert_eq!(asked, NAMES.len());
    for n in NAMES {
        assert_eq!(
            std::fs::read(dst.path().join(n)).unwrap(),
            b"old!",
            "{n} must be skipped"
        );
    }
}
