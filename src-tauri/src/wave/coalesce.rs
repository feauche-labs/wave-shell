//! Wave: when terminal output goes to the screen.
//!
//! The author's pumps (SSH `session_task::pump`, local `local_shell::pump`) hold every chunk up to
//! `FLUSH_AFTER` (8 ms) hoping more will follow — right for `cat` or logs, but it also delays the echo of
//! every keystroke by those 8 ms. Rule here: a chunk that arrives after a quiet spell (nothing flushed for
//! `quiet`) goes out at once; inside a stream (a flush moments ago) the old coalescing stays.

use std::time::Duration;

use tokio::time::Instant;

/// `true`: send the chunk now; `false`: coalesce until the deadline as before.
pub(crate) fn flush_at_once(last_flush: Option<Instant>, now: Instant, quiet: Duration) -> bool {
    last_flush.is_none_or(|t| now.saturating_duration_since(t) >= quiet)
}

#[cfg(test)]
mod tests {
    use super::flush_at_once;
    use std::time::Duration;
    use tokio::time::Instant;

    const Q: Duration = Duration::from_millis(8);

    #[test]
    fn first_output_and_echo_after_a_pause_go_out_at_once() {
        let now = Instant::now();
        assert!(flush_at_once(None, now, Q));
        // A keystroke 120 ms after the previous screen update.
        assert!(flush_at_once(
            Some(now),
            now + Duration::from_millis(120),
            Q
        ));
        assert!(flush_at_once(Some(now), now + Q, Q));
    }

    #[test]
    fn a_stream_is_still_coalesced() {
        let now = Instant::now();
        assert!(!flush_at_once(Some(now), now + Duration::from_millis(1), Q));
        assert!(!flush_at_once(Some(now), now + Duration::from_millis(7), Q));
    }
}
