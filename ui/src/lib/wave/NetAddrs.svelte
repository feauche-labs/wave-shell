<script lang="ts">
  /**
   * Wave: interfaces of the Network card (Status page) with their addresses — which address clients come in on,
   * which one the server goes out to the internet from, and TCP traffic per address. Interface rates come from
   * the author's stats sample; addresses and per-address traffic from `wave_net_info` (refreshed every 10 s).
   */
  import { api, type NetIface, type WaveNetInfo } from '$lib/api';
  import * as m from '$lib/paraglide/messages.js';
  import { bytes, rate } from '$lib/stats/format';

  let { sid, ifaces }: { sid: string | null; ifaces: NetIface[] } = $props();

  let info = $state.raw<WaveNetInfo | null>(null);

  $effect(() => {
    if (!sid) return;
    const id = sid;
    let stop = false;
    const load = async () => {
      const r = await api.waveNetInfo(id);
      if (!stop && r.status === 'ok') info = r.data;
    };
    void load();
    const timer = setInterval(() => void load(), 10_000);
    return () => {
      stop = true;
      clearInterval(timer);
    };
  });

  const routes = $derived([info?.route_v4, info?.route_v6].filter((r) => !!r));
  const outDevs = $derived(new Set(routes.map((r) => r!.dev)));
  const srcs = $derived(new Set(routes.flatMap((r) => (r!.src ? [r!.src] : []))));
  const local = (addr: string) => info?.locals.find((l) => l.addr === addr);

  /** Interfaces: the author's (busiest first), then the ones that only have addresses. */
  const rows = $derived.by(() => {
    const names = [...ifaces.map((i) => i.name)];
    for (const a of info?.addrs ?? []) if (!names.includes(a.iface)) names.push(a.iface);
    return names.map((name) => {
      const addrs = (info?.addrs ?? []).filter((a) => a.iface === name);
      return {
        name,
        rates: ifaces.find((i) => i.name === name),
        addrs,
        incoming: addrs.some((a) => (local(a.addr)?.inbound ?? 0) > 0),
        outgoing: outDevs.has(name)
      };
    });
  });
  const r = (v: number | null | undefined) => rate(v ?? 0).join(' ');
</script>

<div class="wave-net tnum">
  {#each rows as row (row.name)}
    <div class="if">
      <div
        class="head"
        title={row.rates
          ? m.stat_since_boot({ rx: bytes(row.rates.rx_total), tx: bytes(row.rates.tx_total) })
          : undefined}
      >
        <span class="ifn mono">{row.name}</span>
        {#if row.incoming}<span class="badge in">{m.wave_net_in()}</span>{/if}
        {#if row.outgoing}<span class="badge out">{m.wave_net_out()}</span>{/if}
        <span class="sp"></span>
        {#if row.rates}<span class="rate"
            >↓ <b>{r(row.rates.rx_bps)}</b>&nbsp;&nbsp;↑ <b>{r(row.rates.tx_bps)}</b></span
          >{/if}
      </div>
      {#each row.addrs as a (a.addr)}
        {@const l = local(a.addr)}
        <div class="ip">
          <code>{a.addr}/{a.prefix}</code>
          {#if srcs.has(a.addr)}<span class="badge out">{m.wave_net_src()}</span>{/if}
          <span class="sp"></span>
          {#if l}
            <span class="rate">
              {#if l.inbound}{m.wave_net_conns_in({ count: String(l.inbound) })} ·{/if}
              {#if l.outbound}{m.wave_net_conns_out({ count: String(l.outbound) })} ·{/if}
              ↓ {r(l.rx_bps)} · ↑ {r(l.tx_bps)}
            </span>
          {/if}
        </div>
      {/each}
    </div>
  {/each}
  {#if info}
    <p class="note">{m.wave_net_note()}</p>
  {:else if sid}
    <p class="note">{m.wave_net_loading()}</p>
  {/if}
</div>

<style>
  .wave-net {
    display: grid;
    gap: 10px;
    margin-top: 12px;
    padding-top: 12px;
    border-top: 0.5px solid var(--sep);
    font-size: 13px;
  }
  .if {
    display: grid;
    gap: 3px;
  }
  .head,
  .ip {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .ifn {
    font-weight: 600;
    color: var(--text);
  }
  .ip {
    padding-left: 16px;
    color: var(--muted);
  }
  code {
    color: var(--text);
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 12.5px;
  }
  .sp {
    flex: 1;
  }
  .rate {
    color: var(--muted);
    white-space: nowrap;
  }
  .rate b {
    color: var(--text);
    font-weight: 500;
  }
  .badge {
    padding: 1px 7px;
    border-radius: 99px;
    background: var(--fill-2);
    color: var(--muted);
    font-size: 11px;
    white-space: nowrap;
  }
  .badge.in {
    background: color-mix(in oklab, var(--ok, #30d158) 20%, transparent);
    color: var(--ok, #30d158);
  }
  .badge.out {
    background: color-mix(in oklab, var(--accent) 22%, transparent);
    color: var(--accent);
  }
  .note {
    margin: 2px 0 0;
    color: var(--muted);
    font-size: 12px;
  }
</style>
