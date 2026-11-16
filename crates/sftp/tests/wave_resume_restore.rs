//! Wave: «Resume» onto an existing partial file renames it to `*.tlpart` before copying. When the copy then
//! fails or is cancelled, the user's file has to get its name back — it used to vanish from the folder, only
//! `name.tlpart` remained. Local → local; the failure: the partial file is read-only (an earlier download kept
//! 0444) — it can be renamed, but not written to.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use ibc_shell_sftp::{
    ConflictPolicy, Endpoint, TransferConfig, TransferEngine, TransferEvent, TransferJob,
};

#[tokio::test(flavor = "multi_thread")]
async fn resume_that_fails_gives_the_file_its_name_back() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let full: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
    let source = src.path().join("disk.img");
    std::fs::write(&source, &full).unwrap();
    // The user's partial copy: the first half of the same file, read-only.
    let partial = dst.path().join("disk.img");
    std::fs::write(&partial, &full[..100_000]).unwrap();
    std::fs::set_permissions(&partial, std::fs::Permissions::from_mode(0o444)).unwrap();

    let engine = TransferEngine::new(TransferConfig::default());
    let mut rx = engine.subscribe();
    engine
        .enqueue(TransferJob {
            src: Endpoint::Local,
            dst: Endpoint::Local,
            items: vec![source.to_string_lossy().into_owned()],
            dst_dir: dst.path().to_string_lossy().into_owned(),
            conflict: ConflictPolicy::ResumeIfSmaller,
        })
        .unwrap();
    loop {
        let ev = tokio::time::timeout(Duration::from_secs(10), rx.recv())
            .await
            .expect("timeout")
            .expect("closed");
        match ev {
            TransferEvent::Failed { .. } => break,
            TransferEvent::Done { .. } => {
                panic!("the partial file is read-only, resume cannot succeed")
            }
            _ => {}
        }
    }

    let names: Vec<String> = std::fs::read_dir(dst.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        vec!["disk.img".to_owned()],
        "the user's file lost its name"
    );
    assert_eq!(std::fs::read(&partial).unwrap(), &full[..100_000]);
}
