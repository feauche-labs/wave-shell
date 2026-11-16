//! Wave: servers that refuse SETSTAT (chroot-only / managed SFTP hosting). Setting the modification time after
//! an upload is best effort: all bytes arrived, so the file must be saved, not left as `*.tlpart` with a failed
//! transfer that fails the same way on every retry. Runs a local OpenSSH `sftp-server -P setstat`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ibc_shell_sftp::{
    ConflictPolicy, Endpoint, Sftp, TransferConfig, TransferEngine, TransferEvent, TransferJob,
};

async fn server_denying_setstat() -> Option<Arc<Sftp>> {
    let bin = [
        "/usr/libexec/sftp-server",
        "/usr/lib/openssh/sftp-server",
        "/usr/libexec/openssh/sftp-server",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|p| p.exists())?;
    let mut child = tokio::process::Command::new(bin)
        .args(["-P", "setstat,fsetstat,lsetstat"])
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

#[tokio::test(flavor = "multi_thread")]
async fn upload_completes_when_the_server_refuses_to_set_times() {
    let Some(sftp) = server_denying_setstat().await else {
        eprintln!("sftp-server not found, skipping");
        return;
    };
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let file = src.path().join("report.txt");
    std::fs::write(&file, b"all bytes arrive").unwrap();

    let engine = TransferEngine::new(TransferConfig::default());
    let mut rx = engine.subscribe();
    engine
        .enqueue(TransferJob {
            src: Endpoint::Local,
            dst: Endpoint::Remote(sftp),
            items: vec![file.to_string_lossy().into_owned()],
            dst_dir: dst.path().to_string_lossy().into_owned(),
            conflict: ConflictPolicy::Overwrite,
        })
        .unwrap();
    loop {
        match tokio::time::timeout(Duration::from_secs(20), rx.recv())
            .await
            .expect("timeout")
            .expect("closed")
        {
            TransferEvent::Done { .. } => break,
            TransferEvent::Failed { message, .. } => panic!("upload failed: {message}"),
            _ => {}
        }
    }
    assert_eq!(
        std::fs::read(dst.path().join("report.txt")).unwrap(),
        b"all bytes arrive"
    );
    let leftovers: Vec<_> = std::fs::read_dir(dst.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(
        leftovers.len(),
        1,
        "a partial file is left behind: {leftovers:?}"
    );
}
