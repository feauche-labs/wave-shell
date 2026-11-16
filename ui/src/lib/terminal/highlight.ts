/**
 * Output highlighting (Termius-style): colours IPs, URLs, errors, warnings, success words and
 * sizes in plain server output. Only touches text printed with default colours on the normal
 * screen, so programs that colour their own output, and full-screen apps (vim, top), are left
 * exactly as sent. Escape sequences split across chunks are carried over.
 */

const RULES: [RegExp, number][] = [
  [/https?:\/\/[^\s"'<>]+[^\s"'<>.,;:)\]]/, 34],
  [/\b(?:\d{1,3}\.){3}\d{1,3}(?:\/\d{1,2})?(?::\d{1,5})?\b/, 36],
  [/\b(?:error|errors|failed|failure|fatal|denied|refused|panic|critical|exception)\b/i, 31],
  [/\b(?:warn|warning|warnings|deprecated)\b/i, 33],
  [/\b(?:ok|success|successful|succeeded|active|enabled|passed|done)\b/i, 32],
  [/\b\d+(?:[.,]\d+)?\s?(?:%|[KMGT]i?B|[KMGT]B?\/s)(?![\w])/, 35]
];
/** `user@host:~/dir$ ` (Debian/Ubuntu) or `[user@host dir]# ` (RHEL), at a line start. */
const PROMPT =
  // eslint-disable-next-line no-control-regex -- the path stops at the next escape
  /(?<=^|[\r\n])(?:(?<who>[a-z_][\w.-]*@[\w.-]+):(?<path>[~/][^\s\x1b]*)?(?<sign>[#$%] )|\[(?<who2>[a-z_][\w.-]*@[\w.-]+) (?<path2>[^\]\s]+)\](?<sign2>[#$%] ))/g;
const ALL = new RegExp(RULES.map(([r]) => `(${r.source})`).join('|'), 'gi');
// Wave: parameter bytes are the whole 0x30–0x3F range (digits : ; < = > ?), as ECMA-48 defines them.
// eslint-disable-next-line no-control-regex -- matching terminal escape sequences is the point
const CSI = /^\x1b\[([0-?]*)([ -/]*)([@-~])/;

/** Wave: the colour state an SGR sequence leaves behind — foreground, background, reverse. */
type Sgr = { fg: boolean; bg: boolean; rev: boolean };

/**
 * Wave: applies one SGR parameter list to the state. Only what changes the look of colours is
 * tracked (bold, underline and the like do not); `38;5;n` / `38;2;r;g;b` swallow their arguments, so
 * the index of a 256-colour is not read as «bold». Sub-parameters (`38:2::1:2:3`) arrive as one item.
 */
function applySgr(st: Sgr, params: string): Sgr {
  const p = params.split(';');
  const next = { ...st };
  for (let k = 0; k < p.length; k++) {
    const [head = ''] = p[k]!.split(':');
    const n = head === '' ? 0 : Number(head);
    if (n === 0) {
      next.fg = false;
      next.bg = false;
      next.rev = false;
    } else if (n === 7) next.rev = true;
    else if (n === 27) next.rev = false;
    else if (n === 39) next.fg = false;
    else if (n === 49) next.bg = false;
    else if ((n >= 30 && n <= 37) || (n >= 90 && n <= 97)) next.fg = true;
    else if ((n >= 40 && n <= 47) || (n >= 100 && n <= 107)) next.bg = true;
    else if (n === 38 || n === 48) {
      if (n === 38) next.fg = true;
      else next.bg = true;
      // Arguments in the same item (colons) or the following ones (semicolons).
      if (!p[k]!.includes(':')) k += p[k + 1] === '5' ? 2 : p[k + 1] === '2' ? 4 : 0;
    }
  }
  return next;
}

export class Highlighter {
  /** Wave: colours the program set (we only colour text it left in default colours). */
  #sgr: Sgr = { fg: false, bg: false, rev: false };
  /** Default colours in effect. */
  get #plain(): boolean {
    return !this.#sgr.fg && !this.#sgr.bg && !this.#sgr.rev;
  }
  /** Alternate screen (full-screen app) active. */
  #alt = false;
  /** Incomplete escape sequence from the previous chunk. */
  #carry = '';

  feed(chunk: string): string {
    const s = this.#carry + chunk;
    this.#carry = '';
    let out = '';
    let i = 0;
    while (i < s.length) {
      const esc = s.indexOf('\x1b', i);
      const end = esc === -1 ? s.length : esc;
      if (end > i) out += this.#text(s.slice(i, end));
      if (esc === -1) break;
      const n = this.#escape(s, esc);
      if (n === 0) {
        this.#carry = s.slice(esc);
        break;
      }
      out += s.slice(esc, esc + n);
      i = esc + n;
    }
    return out;
  }

  /** Length of the escape sequence at `at`, 0 when it is incomplete. */
  #escape(s: string, at: number): number {
    if (at + 1 >= s.length) return 0;
    const kind = s[at + 1];
    if (kind === '[') {
      const m = CSI.exec(s.slice(at, at + 64));
      if (!m) return s.length - at < 64 ? 0 : 2;
      const [all, params = '', , final] = m;
      // Wave: a leading < = > ? is a private mode (key reporting etc.), not a colour change.
      if (final === 'm' && !/^[<=>?]/.test(params)) {
        this.#sgr = applySgr(this.#sgr, params);
      } else if ((final === 'h' || final === 'l') && /^\?(1049|1047|47)$/.test(params)) {
        this.#alt = final === 'h';
      }
      return all.length;
    }
    if (kind === ']' || kind === 'P' || kind === '_' || kind === '^') {
      for (let j = at + 2; j < s.length; j++) {
        if (s[j] === '\x07') return j - at + 1;
        if (s[j] === '\x1b' && s[j + 1] === '\\') return j - at + 2;
        if (s[j] === '\x1b' && j + 1 >= s.length) return 0;
      }
      return 0;
    }
    return 2;
  }

  #text(t: string): string {
    if (!this.#plain || this.#alt) return t;
    // A plain shell prompt gets the colours of a default Ubuntu one: green user@host, blue path.
    let out = '';
    let last = 0;
    for (const mt of t.matchAll(PROMPT)) {
      out += this.#words(t.slice(last, mt.index));
      const g = mt.groups ?? {};
      out += g.who
        ? `\x1b[1;32m${g.who}\x1b[22;39m:\x1b[1;34m${g.path ?? ''}\x1b[22;39m${g.sign}`
        : `[\x1b[1;32m${g.who2}\x1b[22;39m \x1b[1;34m${g.path2}\x1b[22;39m]${g.sign2}`;
      last = mt.index + mt[0].length;
    }
    return out + this.#words(t.slice(last));
  }

  #words(t: string): string {
    if (!t) return t;
    return t.replace(ALL, (match, ...groups: unknown[]) => {
      const idx = groups.findIndex((g, k) => k < RULES.length && g !== undefined);
      const color = RULES[idx]?.[1];
      return color ? `\x1b[${color}m${match}\x1b[39m` : match;
    });
  }
}
