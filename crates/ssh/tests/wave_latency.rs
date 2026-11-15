#![allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unused_async,
    clippy::unused_async_trait_impl,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    missing_docs
)]
//! Wave: keystroke → echo latency over a real SSH shell (`tests/docker`, password server), with the app's
//! old output rule (every chunk waits 8 ms) and the new one (output after a quiet spell goes at once).
//! Run: `TL_E2E=1 TL_SSH_PORT=2201 TL_SSH_USER=tester TL_SSH_PASSWORD=test-pw-123
//! cargo test -p ibc-shell-ssh --test wave_latency -- --ignored --nocapture`.

use std::sync::Mutex;
use std::time::Duration;

use ibc_shell_ssh::{
    connect, AuthMethod, ConnectStep, Hop, HostKeyDecision, HostKeyInfo, KbdInteractiveRequest,
    KnownHosts, KnownKey, Prompter, PtyRequest, Shell, ShellOutput,
};
use secrecy::SecretString;
use tokio::time::{sleep, sleep_until, timeout, Instant};

#[derive(Default)]
struct Hosts(Mutex<Vec<(String, KnownKey)>>);

impl KnownHosts for Hosts {
    fn lookup(&self, p: &str) -> Result<Vec<KnownKey>, String> {
        let g = self.0.lock().map_err(|e| e.to_string())?;
        Ok(g.iter()
            .filter(|(h, _)| h == p)
            .map(|(_, k)| k.clone())
            .collect())
    }
    fn add(&self, p: &str, k: &KnownKey) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .push((p.to_owned(), k.clone()));
        Ok(())
    }
}

struct Ui;

impl Prompter for Ui {
    fn progress(&self, _: &str, _: ConnectStep) {}
    async fn confirm_unknown_host(&self, _: HostKeyInfo) -> HostKeyDecision {
        HostKeyDecision::AcceptAndRemember
    }
    async fn keyboard_interactive(&self, _: KbdInteractiveRequest) -> Option<Vec<SecretString>> {
        None
    }
    async fn passphrase(&self, _: &str) -> Option<SecretString> {
        None
    }
}

/// Reads until nothing arrives for `quiet`.
async fn drain(sh: &mut Shell, quiet: Duration) {
    while let Ok(Some(item)) = timeout(quiet, sh.read()).await {
        if let ShellOutput::Data(d) = item {
            sh.ack(u32::try_from(d.len()).unwrap_or(u32::MAX));
        }
    }
}

fn stats(name: &str, mut ms: Vec<f64>) {
    ms.sort_by(f64::total_cmp);
    let p = |q: f64| ms[((ms.len() - 1) as f64 * q).round() as usize];
    println!(
        "{name:<34} медиана {:>6.2} мс   90% {:>6.2} мс   макс {:>6.2} мс",
        p(0.5),
        p(0.9),
        ms[ms.len() - 1]
    );
}

#[tokio::test]
#[ignore = "needs docker (TL_E2E=1)"]
async fn keystroke_echo_latency() {
    if std::env::var("TL_E2E").as_deref() != Ok("1") {
        return;
    }
    let env = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("env {k}"));
    let hop = Hop {
        label: "latency".into(),
        host: std::env::var("TL_SSH_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
        port: env("TL_SSH_PORT").parse().unwrap(),
        username: env("TL_SSH_USER"),
        auth: vec![AuthMethod::Password(SecretString::from(env(
            "TL_SSH_PASSWORD",
        )))],
        keepalive_s: 5,
        legacy_algorithms: false,
    };
    let c = connect(vec![hop], &Hosts::default(), &Ui)
        .await
        .expect("connect");
    let mut sh = c
        .open_shell(PtyRequest {
            term: "xterm-256color".into(),
            cols: 120,
            rows: 40,
        })
        .await
        .expect("shell");
    drain(&mut sh, Duration::from_millis(800)).await;

    const FLUSH_AFTER: Duration = Duration::from_millis(8);
    let (mut raw, mut old, mut new) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..60 {
        sleep(Duration::from_millis(60)).await; // a person typing
        let key = [b'a' + (i % 26) as u8];
        let sent = Instant::now();
        sh.write(&key).await.expect("write");
        let first = loop {
            match timeout(Duration::from_secs(2), sh.read()).await {
                Ok(Some(ShellOutput::Data(d))) => {
                    sh.ack(u32::try_from(d.len()).unwrap_or(u32::MAX));
                    break Instant::now();
                }
                Ok(Some(_)) => {}
                _ => panic!("no echo"),
            }
        };
        let ms = |t: Instant| t.duration_since(sent).as_secs_f64() * 1000.0;
        raw.push(ms(first));
        // New rule: a pause of 60 ms before the key → the echo goes to the screen at once.
        new.push(ms(Instant::now()));
        // Old rule: the chunk waited for the 8 ms timer.
        sleep_until(first + FLUSH_AFTER).await;
        old.push(ms(Instant::now()));
    }
    sh.write(b"\x15exit\n").await.ok();
    println!();
    stats("эхо от сервера (сеть + sshd)", raw);
    stats("до экрана, старое правило (8 мс)", old);
    stats("до экрана, новое правило", new);
}
