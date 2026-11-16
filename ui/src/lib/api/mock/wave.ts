/** Wave: demo data for Wave commands (browser demo only). */
import type { Commands } from '../types';
import { delay, ok } from './state';

const jitter = (v: number) => Math.round(v * (0.85 + Math.random() * 0.3));

/** Two addresses on eth0: clients come in on .10, the server goes out to the internet from .11. */
export const waveNetInfo: Commands['waveNetInfo'] = async () => {
  await delay(300);
  return ok({
    addrs: [
      { iface: 'eth0', addr: '203.0.113.10', prefix: 24, v6: false },
      { iface: 'eth0', addr: '203.0.113.11', prefix: 24, v6: false },
      { iface: 'eth0', addr: '2a01:4f8::10', prefix: 64, v6: true },
      { iface: 'docker0', addr: '172.17.0.1', prefix: 16, v6: false }
    ],
    route_v4: { dev: 'eth0', src: '203.0.113.11', via: '203.0.113.1' },
    route_v6: { dev: 'eth0', src: '2a01:4f8::10', via: 'fe80::1' },
    locals: [
      {
        addr: '203.0.113.10',
        inbound: 14,
        outbound: 0,
        rx_bps: jitter(4_600_000),
        tx_bps: jitter(380_000)
      },
      {
        addr: '203.0.113.11',
        inbound: 0,
        outbound: 7,
        rx_bps: jitter(370_000),
        tx_bps: jitter(4_000_000)
      },
      {
        addr: '2a01:4f8::10',
        inbound: 2,
        outbound: 0,
        rx_bps: jitter(210_000),
        tx_bps: jitter(18_000)
      }
    ]
  });
};
