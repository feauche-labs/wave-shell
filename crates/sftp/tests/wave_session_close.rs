//! Wave: closing a Files tab closes its SFTP session; the transfers using it have to end as «Cancelled» (as
//! `sftp_close` promises) — including one waiting on a conflict prompt, which used to wait forever — and a
//! retry has to be refused instead of failing on the dead session again and again. Local `sftp-server`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ibc_shell_sftp::{
    ConflictPolicy, Endpoint, Sftp, SftpError, TransferConfig, TransferEngine, TransferEvent,
    TransferJob,
};

async fn server() -> Option<Arc<Sftp>> {
    let bin = [
        "/usr/libexec/sftp-server",
        "/usr/lib/openssh/sftp-server",
        "/usr/libexec/openssh/sftp-server",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|p| p.exists())?;
    let mut child = tokio::process::Command::new(bin)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .ok()?;
    let io = tokio::io::join(child.stdout.take()?, child.stdin.take()?);
    tokio::spawn(async move {
        let _ = child.wait().await;
    });
    Some(Arc::new(Sftp::from_stream(io).await.expect("sftp init")))
}

async fn next(rx: &mut tokio::sync::mpsc::UnboundedReceiver<TransferEvent>) -> TransferEvent {
    tokio::time::timeout(Duration::from_secs(10), rx.recv())
        .await
        .expect("timeout")
        .expect("closed")
}

#[tokio::test(flavor = "multi_thread")]
async fn closing_the_session_cancels_its_transfers_and_refuses_retry() {
    let Some(sftp) = server().await else {
        eprintln!("sftp-server not found, skipping");
        return;
    };
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    std::fs::write(src.path().join("a.txt"), b"new").unwrap();
    std::fs::write(dst.path().join("a.txt"), b"old").unwrap(); // → a conflict prompt that nobody answers

    let engine = TransferEngine::new(TransferConfig::default());
    let mut rx = engine.subscribe();
    let ids = engine
        .enqueue(TransferJob {
            src: Endpoint::Local,
            dst: Endpoint::Remote(Arc::clone(&sftp)),
            items: vec![src.path().join("a.txt").to_string_lossy().into_owned()],
            dst_dir: dst.path().to_string_lossy().into_owned(),
            conflict: ConflictPolicy::Ask,
        })
        .unwrap();
    while !matches!(next(&mut rx).await, TransferEvent::Conflict { .. }) {}

    // The tab is closed: the bridge cancels the session's transfers, then closes the session.
    engine.cancel_using(&sftp);
    sftp.close().await;
    loop {
        match next(&mut rx).await {
            TransferEvent::Cancelled { .. } => break,
            TransferEvent::Failed { message, .. } => {
                panic!("failed instead of cancelled: {message}")
            }
            _ => {}
        }
    }
    assert!(
        matches!(engine.retry(ids[0]), Err(SftpError::Cancelled)),
        "retry must be refused"
    );
    assert_eq!(std::fs::read(dst.path().join("a.txt")).unwrap(), b"old");
}
