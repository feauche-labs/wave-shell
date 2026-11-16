//! Interactive PTY shell with credit-based backpressure.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use russh::client::Msg;
use russh::{ChannelMsg, ChannelReadHalf, ChannelWriteHalf};
use tokio::sync::{mpsc, Notify};

use crate::session::CREDIT_WINDOW;
use crate::SshError;

/// Counter of bytes delivered to the consumer but not yet acknowledged.
#[derive(Debug, Default)]
pub(crate) struct Credit {
    unacked: AtomicU64,
    notify: Notify,
}

impl Credit {
    pub(crate) fn add(&self, n: usize) {
        self.unacked.fetch_add(n as u64, Ordering::SeqCst);
    }

    pub(crate) fn ack(&self, n: u32) {
        let _ = self
            .unacked
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                Some(v.saturating_sub(u64::from(n)))
            });
        self.notify.notify_one();
    }

    pub(crate) fn unacked(&self) -> u64 {
        self.unacked.load(Ordering::SeqCst)
    }

    fn exhausted(&self) -> bool {
        self.unacked() > u64::from(CREDIT_WINDOW)
    }

    /// Resolves once unacked bytes are within the window.
    pub(crate) async fn wait_available(&self) {
        loop {
            let notified = self.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if !self.exhausted() {
                return;
            }
            notified.await;
        }
    }
}

/// Item read from a shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellOutput {
    /// stdout/stderr bytes (merged, as a terminal sees them).
    Data(Vec<u8>),
    /// Remote process exit status.
    ExitStatus(u32),
}

/// Wave: output held for a screen that lags (credit used up) — read from the SSH channel all the same,
/// so the connection's single loop is never stuck on this channel (exec, SFTP and forwards of the same
/// connection keep working). Past this much the channel is left unread again: the old backpressure.
pub const BACKLOG: usize = 8 << 20;

/// Wave: what the writer task sends, in order.
enum Input {
    Data(Vec<u8>),
    Resize(u16, u16),
}

/// Interactive shell with credit-based backpressure ([`CREDIT_WINDOW`]).
pub struct Shell {
    rx: ChannelReadHalf,
    tx: Arc<ChannelWriteHalf<Msg>>,
    credit: Arc<Credit>,
    /// Wave: output read from the channel but not handed out yet (see [`BACKLOG`]).
    backlog: VecDeque<ShellOutput>,
    backlog_bytes: usize,
    /// Wave: the channel has closed; the backlog is all that is left.
    eof: bool,
    /// Wave: input goes through its own task — waiting to send (russh's queue of 10, the server's window)
    /// must never stop the caller from reading output and taking acks, or the two wait on each other.
    input: mpsc::UnboundedSender<Input>,
    writer: tokio::task::JoinHandle<()>,
}

impl std::fmt::Debug for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shell")
            .field("unacked", &self.credit.unacked())
            .field("backlog", &self.backlog_bytes)
            .finish_non_exhaustive()
    }
}

impl Shell {
    pub(crate) fn new(rx: ChannelReadHalf, tx: ChannelWriteHalf<Msg>) -> Self {
        let tx = Arc::new(tx);
        let (input, mut queue) = mpsc::unbounded_channel::<Input>();
        let out = Arc::clone(&tx);
        let writer = tokio::spawn(async move {
            while let Some(item) = queue.recv().await {
                let sent = match item {
                    Input::Data(d) => out.data_bytes(d).await,
                    Input::Resize(c, r) => {
                        out.window_change(u32::from(c), u32::from(r), 0, 0).await
                    }
                };
                if sent.is_err() {
                    // Dropping the queue makes the next `write` fail: the caller sees the channel is gone.
                    return;
                }
            }
        });
        Self {
            rx,
            tx,
            credit: Arc::new(Credit::default()),
            backlog: VecDeque::new(),
            backlog_bytes: 0,
            eof: false,
            input,
            writer,
        }
    }

    /// Next output item; `None` = channel closed (EOF / disconnect). Holds output back while the
    /// credit window is exhausted; the channel itself is still read up to [`BACKLOG`].
    pub async fn read(&mut self) -> Option<ShellOutput> {
        loop {
            if let Some(front) = self.backlog.front() {
                let data = matches!(front, ShellOutput::Data(_));
                if !data || !self.credit.exhausted() {
                    let item = self.backlog.pop_front()?;
                    if let ShellOutput::Data(d) = &item {
                        self.backlog_bytes -= d.len();
                        self.credit.add(d.len());
                    }
                    return Some(item);
                }
            } else if self.eof {
                return None;
            }
            let room = !self.eof && self.backlog_bytes < BACKLOG;
            let held = !self.backlog.is_empty();
            let credit = Arc::clone(&self.credit);
            // Both arms are cancel-safe: `read` sits in the caller's `select!`.
            let got = tokio::select! {
                () = credit.wait_available(), if held => None,
                m = self.rx.wait(), if room => Some(m),
            };
            match got {
                Some(Some(ChannelMsg::Data { data } | ChannelMsg::ExtendedData { data, .. })) => {
                    self.backlog_bytes += data.len();
                    self.backlog.push_back(ShellOutput::Data(data.to_vec()));
                }
                Some(Some(ChannelMsg::ExitStatus { exit_status })) => {
                    self.backlog.push_back(ShellOutput::ExitStatus(exit_status));
                }
                Some(Some(ChannelMsg::Close) | None) => self.eof = true,
                Some(Some(_)) | None => {}
            }
        }
    }

    /// Queues input bytes; they go out in order. `Err` once the channel can no longer be written.
    pub fn write(&self, data: &[u8]) -> Result<(), SshError> {
        self.input
            .send(Input::Data(data.to_vec()))
            .map_err(|_| SshError::Network("disconnected".into()))
    }

    /// Window change, in order with the input.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), SshError> {
        self.input
            .send(Input::Resize(cols, rows))
            .map_err(|_| SshError::Network("disconnected".into()))
    }

    /// Returns credit for `bytes` consumed by the UI (`shell_ack`).
    pub fn ack(&self, bytes: u32) {
        self.credit.ack(bytes);
    }

    /// Sends EOF + close. Wave: first lets go of the output (so a full channel queue no longer holds
    /// the connection's loop) and drops input still waiting to be sent.
    pub async fn close(self) {
        let Self { rx, tx, writer, .. } = self;
        drop(rx);
        writer.abort();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            let _ = tx.eof().await;
            let _ = tx.close().await;
        })
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn available_under_window() {
        let c = Credit::default();
        c.add(CREDIT_WINDOW as usize);
        tokio::time::timeout(Duration::from_millis(50), c.wait_available())
            .await
            .expect("at the limit is still available");
    }

    #[tokio::test]
    async fn blocks_over_window_until_ack() {
        let c = Arc::new(Credit::default());
        c.add(CREDIT_WINDOW as usize + 10);
        let c2 = Arc::clone(&c);
        let waiter = tokio::spawn(async move { c2.wait_available().await });
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(!waiter.is_finished());
        c.ack(11);
        tokio::time::timeout(Duration::from_millis(500), waiter)
            .await
            .expect("unblocked")
            .expect("join");
    }

    #[test]
    fn ack_saturates() {
        let c = Credit::default();
        c.add(5);
        c.ack(100);
        assert_eq!(c.unacked(), 0);
    }
}
