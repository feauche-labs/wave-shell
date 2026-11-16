//! Wave: addresses of a server and where its traffic goes — for the Network card of Status.
//!
//! One read-only script over the session's SSH connection (exec channel, like the author's `stats_sample`):
//! interface addresses (`ip -o addr`), the route to the internet (`ip route get`), listening TCP ports and two
//! snapshots of established TCP sockets one second apart. From the snapshots: per local address — inbound and
//! outbound connections and receive / send rates. UDP is not in `ss -ti` counters, so per-address rates cover
//! TCP only (Xray, web); the per-interface totals of the author's stats cover everything.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use serde::Serialize;
use tauri::State;

use crate::api::dto::SessionId;
use crate::api::{ApiError, ApiResult};
use crate::commands::session::session_of;
use crate::state::AppState;

/// An address on a network interface (global scope only).
#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct WaveAddr {
    pub iface: String,
    pub addr: String,
    pub prefix: u8,
    pub v6: bool,
}

/// Where traffic to the internet leaves: interface, source address, gateway.
#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct WaveRoute {
    pub dev: String,
    pub src: Option<String>,
    pub via: Option<String>,
}

/// TCP traffic of one local address.
#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct WaveLocal {
    pub addr: String,
    /// Established connections to a listening port of this address (clients coming in).
    pub inbound: u32,
    /// Established connections started from this address (the server going out).
    pub outbound: u32,
    /// Bytes per second received / sent over those connections (TCP only).
    pub rx_bps: f64,
    pub tx_bps: f64,
}

/// Addresses, routes and per-address TCP traffic of a server.
#[derive(Clone, Debug, Default, PartialEq, Serialize, specta::Type)]
pub struct WaveNetInfo {
    pub addrs: Vec<WaveAddr>,
    pub route_v4: Option<WaveRoute>,
    pub route_v6: Option<WaveRoute>,
    pub locals: Vec<WaveLocal>,
}

const SCRIPT: &str = r"export LC_ALL=C
echo '@@W:ADDR@@'; ip -o addr show 2>/dev/null | head -200
echo '@@W:R4@@'; ip -o route get 1.1.1.1 2>/dev/null
echo '@@W:R6@@'; ip -o -6 route get 2606:4700:4700::1111 2>/dev/null
echo '@@W:LISTEN@@'; ss -Hltn 2>/dev/null | head -400
echo '@@W:T1@@'; cut -d' ' -f1 /proc/uptime 2>/dev/null
echo '@@W:S1@@'; ss -Htni state established 2>/dev/null | head -6000
sleep 1
echo '@@W:T2@@'; cut -d' ' -f1 /proc/uptime 2>/dev/null
echo '@@W:S2@@'; ss -Htni state established 2>/dev/null | head -6000
echo '@@W:END@@'
";

/// Reads addresses and per-address traffic of the session's server. Takes ~1 s. Passive for auto-lock.
#[tauri::command]
#[specta::specta]
pub async fn wave_net_info(
    state: State<'_, AppState>,
    session_id: SessionId,
) -> ApiResult<WaveNetInfo> {
    let conn = {
        let h = session_of(&state, session_id)?;
        let live = h
            .live
            .lock()
            .map_err(|_| ApiError::Internal("state poisoned".into()))?;
        live.conn
            .clone()
            .ok_or(ApiError::Network("session not connected".into()))?
    };
    let cmd = format!("sh -c '{}' sh", SCRIPT.replace('\'', "'\\''"));
    let out = tokio::time::timeout(Duration::from_secs(15), conn.exec(&cmd, 4 << 20))
        .await
        .map_err(|_| ApiError::Network("timeout".into()))??;
    Ok(parse(&String::from_utf8_lossy(&out.stdout)))
}

/// Text of each `@@W:NAME@@` section.
fn sections(text: &str) -> HashMap<&str, Vec<&str>> {
    let mut out: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut cur = "";
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("@@W:").and_then(|s| s.strip_suffix("@@")) {
            cur = name;
            out.entry(cur).or_default();
        } else if !cur.is_empty() {
            out.entry(cur).or_default().push(line);
        }
    }
    out
}

/// `1.2.3.4:443`, `[2a01::1]:443`, `[::ffff:1.2.3.4]:443`, `[fe80::1%eth0]:22` → (address, port).
fn host_port(s: &str) -> Option<(String, u16)> {
    let (host, port) = s.rsplit_once(':')?;
    let port = port.parse().ok()?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let host = host.split('%').next().unwrap_or(host);
    let host = host
        .strip_prefix("::ffff:")
        .filter(|h| h.contains('.'))
        .unwrap_or(host);
    Some((host.to_owned(), port))
}

fn is_loopback(addr: &str) -> bool {
    addr.starts_with("127.") || addr == "::1"
}

fn parse_route(lines: Option<&Vec<&str>>) -> Option<WaveRoute> {
    let toks: Vec<&str> = lines?.first()?.split_whitespace().collect();
    let after = |k: &str| {
        toks.iter()
            .position(|t| *t == k)
            .and_then(|i| toks.get(i + 1))
            .map(|s| (*s).to_owned())
    };
    Some(WaveRoute {
        dev: after("dev")?,
        src: after("src"),
        via: after("via"),
    })
}

/// One socket of `ss -tni`: (local, peer) → (bytes received, bytes sent).
fn sockets(lines: Option<&Vec<&str>>) -> HashMap<(String, String), (f64, f64)> {
    let mut out = HashMap::new();
    let mut head: Option<(String, String)> = None;
    for line in lines.into_iter().flatten() {
        let toks: Vec<&str> = line.split_whitespace().collect();
        if line.starts_with(|c: char| c.is_ascii_digit()) && toks.len() >= 4 {
            head = Some((toks[2].to_owned(), toks[3].to_owned()));
            continue;
        }
        let Some(key) = head.take() else { continue };
        let num = |k: &str| {
            toks.iter()
                .find_map(|t| t.strip_prefix(k))
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0)
        };
        let acked = num("bytes_acked:");
        let sent = if acked > 0.0 {
            acked
        } else {
            num("bytes_sent:")
        };
        out.insert(key, (num("bytes_received:"), sent));
    }
    out
}

fn parse(text: &str) -> WaveNetInfo {
    let s = sections(text);
    let addrs = s
        .get("ADDR")
        .into_iter()
        .flatten()
        .filter_map(|line| {
            let t: Vec<&str> = line.split_whitespace().collect();
            let family = *t.get(2)?;
            if family != "inet" && family != "inet6" {
                return None;
            }
            let iface = t.get(1)?.split('@').next()?.to_owned();
            let (addr, prefix) = t.get(3)?.split_once('/')?;
            let scope = t
                .iter()
                .position(|x| *x == "scope")
                .and_then(|i| t.get(i + 1));
            if iface == "lo" || scope.is_some_and(|sc| *sc != "global") {
                return None;
            }
            Some(WaveAddr {
                iface,
                addr: addr.to_owned(),
                prefix: prefix.parse().ok()?,
                v6: family == "inet6",
            })
        })
        .take(32)
        .collect();

    let listening: HashSet<u16> = s
        .get("LISTEN")
        .into_iter()
        .flatten()
        .filter_map(|line| {
            line.split_whitespace()
                .find_map(|t| host_port(t).map(|(_, p)| p))
        })
        .collect();

    let time = |k: &str| {
        s.get(k)
            .and_then(|l| l.first())
            .and_then(|v| v.trim().parse::<f64>().ok())
    };
    let dt = match (time("T1"), time("T2")) {
        (Some(a), Some(b)) if b > a => b - a,
        _ => 1.0,
    };
    let before = sockets(s.get("S1"));
    let after = sockets(s.get("S2"));
    let mut per: HashMap<String, WaveLocal> = HashMap::new();
    for ((local, peer), (rx, tx)) in &after {
        let Some((addr, port)) = host_port(local) else {
            continue;
        };
        if is_loopback(&addr) || host_port(peer).is_some_and(|(p, _)| is_loopback(&p)) {
            continue;
        }
        let e = per.entry(addr.clone()).or_insert(WaveLocal {
            addr,
            inbound: 0,
            outbound: 0,
            rx_bps: 0.0,
            tx_bps: 0.0,
        });
        if listening.contains(&port) {
            e.inbound += 1;
        } else {
            e.outbound += 1;
        }
        if let Some((rx0, tx0)) = before.get(&(local.clone(), peer.clone())) {
            e.rx_bps += (rx - rx0).max(0.0) / dt;
            e.tx_bps += (tx - tx0).max(0.0) / dt;
        }
    }
    let mut locals: Vec<WaveLocal> = per.into_values().collect();
    locals.sort_by(|a, b| {
        (b.inbound + b.outbound)
            .cmp(&(a.inbound + a.outbound))
            .then(a.addr.cmp(&b.addr))
    });
    locals.truncate(32);

    WaveNetInfo {
        addrs,
        route_v4: parse_route(s.get("R4")),
        route_v6: parse_route(s.get("R6")),
        locals,
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    const OUT: &str = "@@W:ADDR@@
1: lo    inet 127.0.0.1/8 scope host lo\\       valid_lft forever preferred_lft forever
2: eth0    inet 203.0.113.10/24 brd 203.0.113.255 scope global eth0\\       valid_lft forever
2: eth0    inet6 2a01:4f8::10/64 scope global \\       valid_lft forever
2: eth0    inet6 fe80::1/64 scope link \\       valid_lft forever
3: eth1    inet 198.51.100.77/24 brd 198.51.100.255 scope global eth1\\       valid_lft forever
5: veth9@if4    inet 172.18.0.1/16 scope global veth9\\       valid_lft forever
@@W:R4@@
1.1.1.1 via 198.51.100.1 dev eth1 src 198.51.100.77 uid 0 \\    cache
@@W:R6@@
@@W:LISTEN@@
LISTEN 0      4096   0.0.0.0:443  0.0.0.0:*
LISTEN 0      128    [::]:22      [::]:*
@@W:T1@@
100.00
@@W:S1@@
0      0      203.0.113.10:443   91.204.17.6:51234
\t cubic bytes_sent:1000 bytes_acked:1000 bytes_received:5000 segs_out:10
0      0      [::ffff:203.0.113.10]:22   85.140.3.77:40000
\t cubic bytes_acked:100 bytes_received:200
0      0      198.51.100.77:41000   8.8.8.8:443
\t cubic bytes_acked:2000 bytes_received:300
0      0      127.0.0.1:6379   127.0.0.1:50000
\t cubic bytes_acked:999 bytes_received:999
@@W:T2@@
102.00
@@W:S2@@
0      0      203.0.113.10:443   91.204.17.6:51234
\t cubic bytes_sent:1500 bytes_acked:1400 bytes_received:9000 segs_out:12
0      0      [::ffff:203.0.113.10]:22   85.140.3.77:40000
\t cubic bytes_acked:300 bytes_received:400
0      0      198.51.100.77:41000   8.8.8.8:443
\t cubic bytes_acked:8000 bytes_received:700
0      0      203.0.113.10:443   176.59.40.12:600
\t cubic bytes_acked:50 bytes_received:60
0      0      127.0.0.1:6379   127.0.0.1:50000
\t cubic bytes_acked:5999 bytes_received:5999
@@W:END@@
";

    #[test]
    fn addresses_skip_loopback_and_link_local() {
        let n = parse(OUT);
        let a: Vec<(&str, &str, u8, bool)> = n
            .addrs
            .iter()
            .map(|a| (a.iface.as_str(), a.addr.as_str(), a.prefix, a.v6))
            .collect();
        assert_eq!(
            a,
            vec![
                ("eth0", "203.0.113.10", 24, false),
                ("eth0", "2a01:4f8::10", 64, true),
                ("eth1", "198.51.100.77", 24, false),
                ("veth9", "172.18.0.1", 16, false)
            ]
        );
    }

    #[test]
    fn route_to_the_internet() {
        let n = parse(OUT);
        let r = n.route_v4.expect("v4 route");
        assert_eq!(
            (r.dev.as_str(), r.src.as_deref(), r.via.as_deref()),
            ("eth1", Some("198.51.100.77"), Some("198.51.100.1"))
        );
        assert!(n.route_v6.is_none());
    }

    #[test]
    fn traffic_per_local_address() {
        let n = parse(OUT);
        let by = |a: &str| n.locals.iter().find(|l| l.addr == a).cloned().expect(a);
        let inb = by("203.0.113.10");
        // 443 + 22 are listening: three clients came in (the mapped IPv6 one counts as IPv4).
        assert_eq!((inb.inbound, inb.outbound), (3, 0));
        // Over dt = 2 s: 443 got 4000 B and sent 400 B, 22 got 200 B and sent 200 B; the new socket has no baseline.
        assert!(
            (inb.rx_bps - 2100.0).abs() < 1e-6 && (inb.tx_bps - 300.0).abs() < 1e-6,
            "{inb:?}"
        );
        let out = by("198.51.100.77");
        assert_eq!((out.inbound, out.outbound), (0, 1));
        assert!(
            (out.tx_bps - 3000.0).abs() < 1e-6 && (out.rx_bps - 200.0).abs() < 1e-6,
            "{out:?}"
        );
        assert!(n.locals.iter().all(|l| l.addr != "127.0.0.1"));
    }
}
