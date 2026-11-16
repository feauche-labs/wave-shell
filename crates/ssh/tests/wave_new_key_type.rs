#![allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unused_async,
    clippy::unused_async_trait_impl,
    missing_docs
)]
//! Wave: a server that had only an ECDSA key (saved as trusted) gains an Ed25519 key, the ECDSA key
//! unchanged. The app asked for Ed25519 first regardless of what it had saved, got a key it did not
//! know and showed the hard «host key changed» block. OpenSSH asks for the saved key types first.
//! In-process russh server with both keys.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use ibc_shell_ssh::{
    connect, AuthMethod, ConnectStep, Hop, HostKeyDecision, HostKeyInfo, KbdInteractiveRequest,
    KnownHosts, KnownKey, Prompter, SshError,
};
use russh::keys::{decode_secret_key, HashAlg};
use russh::server::{Auth, Config, Handler, Server};
use secrecy::SecretString;
use tokio::net::TcpListener;

// The throwaway fixture from mock_server.rs.
const ED25519: &str = "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACAjWF9XMGonNvOZS5jQtwN2+OyShDB1KB0m2qfUAEySdgAAAIhasfTVWrH0
1QAAAAtzc2gtZWQyNTUxOQAAACAjWF9XMGonNvOZS5jQtwN2+OyShDB1KB0m2qfUAEySdg
AAAEBpExXls+OkMCOV+0lvM7tv4lCconrTA8VEu/lb4/uEkyNYX1cwaic285lLmNC3A3b4
7JKEMHUoHSbap9QATJJ2AAAABHRlc3QB
-----END OPENSSH PRIVATE KEY-----
";

#[derive(Clone)]
struct Mock;

impl Server for Mock {
    type Handler = Self;
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        Self
    }
}

impl Handler for Mock {
    type Error = russh::Error;
    async fn auth_password(&mut self, _: &str, _: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }
}

#[derive(Default)]
struct Hosts(Mutex<Vec<(String, KnownKey)>>);

impl KnownHosts for Hosts {
    fn lookup(&self, p: &str) -> Result<Vec<KnownKey>, String> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .iter()
            .filter(|(h, _)| h == p)
            .map(|(_, k)| k.clone())
            .collect())
    }
    fn add(&self, p: &str, k: &KnownKey) -> Result<(), String> {
        self.0.lock().unwrap().push((p.to_owned(), k.clone()));
        Ok(())
    }
}

/// Answers «reject» to an unknown key: the test must never get that far.
struct Ui;

impl Prompter for Ui {
    fn progress(&self, _: &str, _: ConnectStep) {}
    async fn confirm_unknown_host(&self, _: HostKeyInfo) -> HostKeyDecision {
        HostKeyDecision::Reject
    }
    async fn keyboard_interactive(&self, _: KbdInteractiveRequest) -> Option<Vec<SecretString>> {
        None
    }
    async fn passphrase(&self, _: &str) -> Option<SecretString> {
        None
    }
}

#[tokio::test]
async fn a_new_key_type_on_a_known_server_is_not_a_changed_key() {
    let ecdsa_pem = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/keys/id_ecdsa_p256"
    ))
    .expect("ecdsa fixture");
    let ecdsa = decode_secret_key(&ecdsa_pem, None).expect("ecdsa key");
    let ed = decode_secret_key(ED25519, None).expect("ed25519 key");
    let config = Config {
        keys: vec![ed, ecdsa.clone()],
        auth_rejection_time: Duration::from_millis(1),
        auth_rejection_time_initial: Some(Duration::from_millis(0)),
        ..Default::default()
    };
    let sock = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
    let port = sock.local_addr().expect("addr").port();
    tokio::spawn(async move {
        let _ = Mock.run_on_socket(Arc::new(config), &sock).await;
    });

    // Trusted earlier, when the server only had its ECDSA key.
    let public = ecdsa.public_key();
    let hosts = Hosts::default();
    hosts
        .add(
            &format!("[127.0.0.1]:{port}"),
            &KnownKey {
                id: "1".into(),
                key_type: public.algorithm().as_str().to_owned(),
                key_blob: public.to_bytes().expect("blob"),
                fingerprint: public.fingerprint(HashAlg::Sha256).to_string(),
            },
        )
        .unwrap();

    let hop = Hop {
        label: "mock".into(),
        host: "127.0.0.1".into(),
        port,
        username: "tester".into(),
        auth: vec![AuthMethod::Password(SecretString::from("pw".to_owned()))],
        keepalive_s: 0,
        legacy_algorithms: false,
    };
    match connect(vec![hop], &hosts, &Ui).await {
        Ok(_) => {}
        Err(SshError::HostKeyChanged(m)) => {
            panic!("false «host key changed»: {m:?}")
        }
        Err(e) => panic!("connect: {e:?}"),
    }
    assert_eq!(hosts.0.lock().unwrap().len(), 1, "nothing new to save");
}
