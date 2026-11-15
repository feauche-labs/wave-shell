# Wave Shell

[Русский](README.md) · **English**

A lightweight SSH and SFTP client for people who run many servers: nested folders, country flags, a terminal that
stays out of the way, and secrets that never leave your computer. macOS, Windows and Linux.

## Download

Click your system and the download starts. Always the latest version.

<p align="center">
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-macOS-Apple-Silicon.dmg"><img alt="Download macOS · Apple Silicon" src="https://img.shields.io/badge/Download-macOS%20%C2%B7%20Apple%20Silicon-000000?style=for-the-badge&logo=apple&logoColor=white"></a>
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-macOS-Intel.dmg"><img alt="Download macOS · Intel" src="https://img.shields.io/badge/Download-macOS%20%C2%B7%20Intel-555555?style=for-the-badge&logo=apple&logoColor=white"></a>
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-Windows-x64-setup.exe"><img alt="Download Windows 10/11" src="https://img.shields.io/badge/Download-Windows%2010/11-0078D4?style=for-the-badge&logo=windows&logoColor=white"></a>
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-Linux-x64.AppImage"><img alt="Download Linux · AppImage" src="https://img.shields.io/badge/Download-Linux%20%C2%B7%20AppImage-E95420?style=for-the-badge&logo=linux&logoColor=white"></a>
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-Linux-x64.deb"><img alt="Download Linux · .deb" src="https://img.shields.io/badge/Download-Linux%20%C2%B7%20.deb-A81D33?style=for-the-badge&logo=debian&logoColor=white"></a>
</p>

Other versions and checksums (`SHA256SUMS.txt`) are on the
[Releases](https://github.com/feauche-labs/wave-shell/releases) page.

Builds are not signed with a paid certificate, so the system warns on first launch:

- **macOS.** Drag the app to Applications. On first launch: right-click → Open → Open. If macOS says the app is
  damaged: `xattr -dr com.apple.quarantine "/Applications/Wave Shell.app"`.
- **Windows.** In the SmartScreen window: More info → Run anyway.
- **Linux.** `chmod +x` the AppImage and run it, or `sudo apt install ./file.deb`.

## Features

- **Servers where they live.** Nested folders, a country flag per host, search, import from `~/.ssh/config`.
- **Terminal.** Tabs (drag to reorder or move into a separate window), 15 themes, a local terminal, auto-reconnect,
  startup command and one-click tmux, a notification when a long command finishes.
- **Files (SFTP).** Transfer queue with resume and pause, drag and drop from Finder, text and log viewer, open in the
  system app, server-to-server transfers.
- **Server status without an agent.** Per-core CPU, memory, disk, network and processes over the open SSH connection.
- **Docker.** Containers, Compose, images, volumes, networks, events and logs across all connected servers.
- **Domains.** SSL certificate expiry (Let's Encrypt, nginx, Apache, Caddy) across all servers.
- **Port forwarding.** Local, remote and SOCKS5, auto-start on connect.
- **Keys.** Generate an Ed25519 key and install it on a server in one click, like `ssh-copy-id`.
- **Snippets.** Commands in folders, run on one server or several at once.
- **Security.** Encrypted vault (Argon2id + XChaCha20-Poly1305) with the key in the system keychain. A changed host
  key blocks the connection. No accounts, no cloud. Encrypted backup.
- **Keyboard first.** Command palette <kbd>⌘K</kbd> / <kbd>Ctrl K</kbd>, all shortcuts on <kbd>?</kbd>.
- Russian and English UI, light and dark themes, a macOS menu bar icon.

## Development

You need Rust (stable), Node with pnpm, and [just](https://github.com/casey/just).

```sh
pnpm install
pnpm tauri dev                  # run the app
cargo test --workspace          # Rust tests
pnpm -C ui check && pnpm -C ui test
just e2e                        # integration tests against OpenSSH in Docker
```

## Releasing

Releases are cut by [release-please](https://github.com/googleapis/release-please); versions and tags are never set by
hand.

1. Commits to `main` use a type prefix: `feat: …` for new things, `fix: …` for fixes; also `perf`, `update`,
   `delete`, `upstream` (changes pulled from iBC Shell), `docs`, `refactor`, `build`, `ci`. Commits without a prefix
   are left out of the changelog and do not bump the version.
2. After every push release-please keeps a "chore: подготовить релиз X.Y.Z" PR open with the new version in all files
   and a `CHANGELOG.md` entry. `feat` bumps the minor number, everything else the patch number.
3. Merging the PR builds five installers (macOS Apple Silicon and Intel, Windows, Linux AppImage and .deb), attaches
   them to the release with checksums and publishes it.

## Auto-update

Wave Shell updates itself: when a new version is out, "Version … available" appears at the bottom of the sidebar.
Clicking it shows what is new and an "Update and restart" button — the app downloads the update, installs it and
restarts. Works on macOS, Windows and Linux (AppImage and .deb; the .deb install asks for the administrator
password).

Updates are signed with a separate key (not a paid Apple or Microsoft certificate): the app installs only packages
signed with it, so a swapped file is never installed. The public key is in `src-tauri/tauri.conf.json`
(`plugins.updater.pubkey`); the private key and its password are repository secrets
(**Settings → Secrets and variables → Actions**): `TAURI_SIGNING_PRIVATE_KEY` and
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Every release signs the packages and publishes `latest.json`, which the app
reads.

Do not lose the private key: without it installed copies can no longer update themselves, only by reinstalling.

## Origin and license

Wave Shell is based on [iBC Shell](https://github.com/dramaformama/ibc-shell) by Georg (MIT) and regularly pulls in
its updates. License: MIT, see [LICENSE](LICENSE).
