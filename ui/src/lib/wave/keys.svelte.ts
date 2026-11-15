/**
 * Wave: the Keys section (like Termius Keychain). Keys are the author's identities of kind key / key_password /
 * agent (encrypted in the vault); saved host passwords stay out of this list. A host picks one of these keys.
 */
import { api, type IdentityDto } from '$lib/api';

/** What the side panel shows: a key id, a new key (paste / file), a generated key, or nothing. */
export type KeyPanel = { kind: 'edit'; id: string } | { kind: 'new' } | { kind: 'generate' } | null;

class WaveKeys {
  list = $state.raw<IdentityDto[]>([]);
  panel = $state<KeyPanel>(null);

  async load(): Promise<void> {
    const r = await api.identityList();
    if (r.status === 'ok')
      this.list = r.data
        .filter((i) => i.kind !== 'password')
        .sort((a, b) => a.label.localeCompare(b.label));
  }

  get count(): number {
    return this.list.length;
  }

  byId(id: string): IdentityDto | undefined {
    return this.list.find((k) => k.id === id);
  }

  /** Short type for a tile: «ED25519», «RSA», or «Агент». */
  type(k: IdentityDto): string {
    if (k.kind === 'agent') return 'SSH-agent';
    const a = (k.key_algo ?? '').toLowerCase();
    if (a.includes('ed25519')) return 'ED25519';
    if (a.includes('rsa')) return 'RSA';
    if (a.includes('ecdsa')) return 'ECDSA';
    return k.key_algo ?? '—';
  }
}

export const waveKeys = new WaveKeys();

/** The vault's answer for an encrypted key without (or with a wrong) passphrase. */
export function passphraseProblem(detail: string | undefined): 'need' | 'wrong' | null {
  if (!detail) return null;
  if (/wrong passphrase/.test(detail)) return 'wrong';
  if (/passphrase required/.test(detail)) return 'need';
  return null;
}
