#![allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unused_async,
    clippy::unused_async_trait_impl,
    clippy::too_many_arguments,
    missing_docs
)]
//! Wave: a terminal whose screen lags behind a flood of output (it has not confirmed what it drew) must
//! neither stall the rest of the connection (exec — «Состояние», SFTP, forwards) nor hang for good
//! once the user types. russh hands every channel its data from one loop per connection and waits
//! when a channel's queue is full; input goes through a queue of 10. In-process russh server.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ibc_shell_ssh::{
    connect, AuthMethod, ConnectStep, Connection, Hop, HostKeyDecision, HostKeyInfo,
    KbdInteractiveRequest, KnownHosts, KnownKey, Prompter, PtyRequest, Shell, ShellOutput,
};
use russh::keys::decode_secret_key;
use russh::server::{Auth, Config, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId};
use secrecy::SecretString;
use tokio::net::TcpListener;
use tokio::time::timeout;

// The throwaway fixture from mock_server.rs.
const HOST_KEY: &str = "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACAjWF9XMGonNvOZS5jQtwN2+OyShDB1KB0m2qfUAEySdgAAAIhasfTVWrH0
1QAAAAtzc2gtZWQyNTUxOQAAACAjWF9XMGonNvOZS5jQtwN2+OyShDB1KB0m2qfUAEySdg
AAAEBpExXls+OkMCOV+0lvM7tv4lCconrTA8VEu/lb4/uEkyNYX1cwaic285lLmNC3A3b4
7JKEMHUoHSbap9QATJJ2AAAABHRlc3QB
-----END OPENSSH PRIVATE KEY-----
";
const CHUNK: usize = 32 * 1024;

/// A server whose shell prints `flood` bytes at once and counts the input it gets.
#[derive(Clone)]
struct Flood {
    flood: usize,
    typed: Arc<AtomicUsize>,
}

impl Server for Flood {
    type Handler = Self;
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        self.clone()
    }
}

impl Handler for Flood {
    type Error = russh::Error;

    async fn auth_password(&mut self, _: &str, _: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        _: Channel<Msg>,
        reply: russh::server::ChannelOpenHandle,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        reply.accept().await;
        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _: &str,
        _: u32,
        _: u32,
        _: u32,
        _: u32,
        _: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_success(channel)?;
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_success(channel)?;
        let (h, n) = (session.handle(), self.flood);
        tokio::spawn(async move {
            let mut sent = 0;
            while sent < n {
                if h.data(channel, vec![b'y'; CHUNK]).await.is_err() {
                    return;
                }
                sent += CHUNK;
            }
        });
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        _: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_success(channel)?;
        session.data(channel, b"pong".to_vec())?;
        session.exit_status_request(channel, 0)?;
        session.close(channel)?;
        Ok(())
    }

    async fn data(
        &mut self,
        _: ChannelId,
        data: &[u8],
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        self.typed.fetch_add(data.len(), Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Default)]
struct Hosts(Mutex<Vec<(String, KnownKey)>>);

impl KnownHosts for Hosts {
    fn lookup(&self, _: &str) -> Result<Vec<KnownKey>, String> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|(_, k)| k.clone())
            .collect())
    }
    fn add(&self, p: &str, k: &KnownKey) -> Result<(), String> {
        self.0.lock().unwrap().push((p.to_owned(), k.clone()));
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

async fn open(flood: usize) -> (Connection, Shell, Arc<AtomicUsize>) {
    let typed = Arc::new(AtomicUsize::new(0));
    let config = Config {
        keys: vec![decode_secret_key(HOST_KEY, None).expect("host key")],
        auth_rejection_time: Duration::from_millis(1),
        auth_rejection_time_initial: Some(Duration::from_millis(0)),
        ..Default::default()
    };
    let sock = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
    let port = sock.local_addr().expect("addr").port();
    let mut srv = Flood {
        flood,
        typed: typed.clone(),
    };
    tokio::spawn(async move {
        let _ = srv.run_on_socket(Arc::new(config), &sock).await;
    });
    let hop = Hop {
        label: "flood".into(),
        host: "127.0.0.1".into(),
        port,
        username: "tester".into(),
        auth: vec![AuthMethod::Password(SecretString::from("pw".to_owned()))],
        keepalive_s: 0,
        legacy_algorithms: false,
    };
    let conn = connect(vec![hop], &Hosts::default(), &Ui)
        .await
        .expect("connect");
    let sh = conn
        .open_shell(PtyRequest {
            term: "xterm-256color".into(),
            cols: 80,
            rows: 24,
        })
        .await
        .expect("shell");
    (conn, sh, typed)
}

/// Reads like a screen that draws but never confirms, until the shell holds output back.
async fn read_until_held(sh: &mut Shell) -> usize {
    let mut got = 0;
    while let Ok(Some(ShellOutput::Data(d))) = timeout(Duration::from_millis(500), sh.read()).await
    {
        got += d.len();
    }
    got
}

/// Confirms and drains everything that is left.
async fn drain(sh: &mut Shell, total: usize, mut got: usize) {
    sh.ack(u32::MAX);
    while got < total {
        match timeout(Duration::from_secs(10), sh.read())
            .await
            .expect("output stalled")
        {
            Some(ShellOutput::Data(d)) => {
                got += d.len();
                sh.ack(u32::try_from(d.len()).unwrap());
            }
            Some(ShellOutput::ExitStatus(_)) => {}
            None => break,
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_lagging_screen_does_not_stall_the_rest_of_the_connection() {
    let flood = 6 << 20;
    let (conn, mut sh, _) = open(flood).await;
    let got = read_until_held(&mut sh).await;
    assert!(
        got < flood,
        "the shell must hold output back for a lagging screen"
    );
    // «Состояние» asks the same connection meanwhile.
    let out = timeout(Duration::from_secs(5), conn.exec("ping", 64))
        .await
        .expect("the connection stalled behind the lagging terminal")
        .expect("exec");
    assert_eq!(out.stdout, b"pong");
    drain(&mut sh, flood, got).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn typing_while_the_screen_lags_never_hangs() {
    let flood = 16 << 20;
    let (_conn, mut sh, typed) = open(flood).await;
    let got = read_until_held(&mut sh).await;
    // The user presses Ctrl+C again and again while the screen is behind.
    // (Writing used to wait behind the screen: the 11th keystroke hung, and in the app the shell's
    // loop then stopped taking the screen's acks — for good.)
    for _ in 0..30 {
        sh.write(b"\x03").expect("write");
    }
    // The screen catches up; every keystroke reaches the server.
    drain(&mut sh, flood, got).await;
    timeout(Duration::from_secs(5), async {
        while typed.load(Ordering::SeqCst) < 30 {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("keystrokes lost");
}
