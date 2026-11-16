import { describe, expect, it } from 'vitest';
import { Highlighter } from './highlight';

const C = (code: number, s: string) => `\x1b[${code}m${s}\x1b[39m`;

describe('Highlighter', () => {
  it('colours plain output', () => {
    const h = new Highlighter();
    expect(h.feed('IPv4 address for ens3: 138.124.38.121\r\n')).toBe(
      `IPv4 address for ens3: ${C(36, '138.124.38.121')}\r\n`
    );
    expect(h.feed('Memory usage: 17%')).toBe(`Memory usage: ${C(35, '17%')}`);
    expect(h.feed('Permission denied')).toBe(`Permission ${C(31, 'denied')}`);
    expect(h.feed('see https://ubuntu.com/pro.')).toBe(`see ${C(34, 'https://ubuntu.com/pro')}.`);
  });

  it('leaves coloured and full-screen output alone', () => {
    const h = new Highlighter();
    expect(h.feed('\x1b[32mok 1.2.3.4\x1b[0m done')).toBe(
      `\x1b[32mok 1.2.3.4\x1b[0m ${C(32, 'done')}`
    );
    expect(h.feed('\x1b[?1049herror\x1b[?1049l')).toBe('\x1b[?1049herror\x1b[?1049l');
  });

  it('keeps escape sequences split across chunks intact', () => {
    const h = new Highlighter();
    expect(h.feed('a\x1b[3')).toBe('a');
    expect(h.feed('2mx\x1b[0m')).toBe('\x1b[32mx\x1b[0m');
    expect(h.feed('\x1b]0;title')).toBe('');
    expect(h.feed('\x07ok')).toBe(`\x1b]0;title\x07${C(32, 'ok')}`);
  });

  it('colours a plain shell prompt like a default Ubuntu one', () => {
    const out = new Highlighter().feed('\x1b]0;t\x07root@vps:~/app# ls\r\n[deploy@web1 www]$ ');
    expect(out).toContain('\x1b[1;32mroot@vps\x1b[22;39m:\x1b[1;34m~/app\x1b[22;39m# ls');
    expect(out).toContain('[\x1b[1;32mdeploy@web1\x1b[22;39m \x1b[1;34mwww\x1b[22;39m]$ ');
    // An address in the middle of a line is not a prompt.
    expect(new Highlighter().feed('mail to a@b.c:/x$ now')).not.toContain('1;32m');
  });

  // Wave: fish 4, vim and neovim send key-mode codes with < = > markers, and SGR with colons
  // (undercurl). They were taken for incomplete sequences: the prompt and typed echo were held back.
  it('passes private-marker and colon sequences at once', () => {
    const h = new Highlighter();
    expect(h.feed('\x1b[>4;1m\x1b[=5uroot@vps:~# ')).toBe(
      '\x1b[>4;1m\x1b[=5u\x1b[1;32mroot@vps\x1b[22;39m:\x1b[1;34m~\x1b[22;39m# '
    );
    expect(h.feed('l')).toBe('l');
    expect(h.feed('\x1b[<u')).toBe('\x1b[<u');
    expect(h.feed('\x1b[4:3mwavy\x1b[4:0m')).toBe('\x1b[4:3mwavy\x1b[4:0m');
  });

  it('keeps colouring after key-mode codes (they are not colours)', () => {
    const h = new Highlighter();
    expect(h.feed('\x1b[>4;2mPermission denied')).toBe('\x1b[>4;2mPermission ' + C(31, 'denied'));
  });
  // Wave: bash 5.1+ (Ubuntu 22.04+) shows a paste reversed — `ESC[7m`…`ESC[27m` — and never sends a
  // full reset after it. Only a full reset counted as «plain again», so colouring stopped until some
  // program (ls) happened to send one.
  it('colours again once a pasted (reversed) line is turned back to normal', () => {
    const h = new Highlighter();
    h.feed('root@nl-2:~# \x1b[?2004h\x1b[7mcat > /etc/x.nft\x1b[27m\r\n\x1b[?2004l\r');
    expect(h.feed('root@nl-2:~# ')).toBe('\x1b[1;32mroot@nl-2\x1b[22;39m:\x1b[1;34m~\x1b[22;39m# ');
  });

  it('tracks colour per attribute, not only a full reset', () => {
    const h = new Highlighter();
    // Own colour → left alone; colour off (39) with bold still on → plain colours again.
    expect(h.feed('\x1b[1;31mfailed\x1b[39m ok')).toBe(`\x1b[1;31mfailed\x1b[39m ${C(32, 'ok')}`);
    // A background, a 256-colour and a truecolour foreground each count as «coloured».
    expect(h.feed('\x1b[44m ok \x1b[49m')).toBe('\x1b[44m ok \x1b[49m');
    expect(h.feed('\x1b[38;5;196mok\x1b[39m')).toBe('\x1b[38;5;196mok\x1b[39m');
    expect(h.feed('\x1b[38:2::1:2:3mok\x1b[m')).toBe('\x1b[38:2::1:2:3mok\x1b[m');
    // `38;5;1` — the trailing 1 is the colour index, not bold; 39 after it is still plain.
    expect(h.feed('\x1b[38;5;1;39mok')).toBe(`\x1b[38;5;1;39m${C(32, 'ok')}`);
  });
});
