# IP Tracker

A small desktop app that watches your public IP address, logs every change, and shows you reports. Runs in the system tray on Linux, Windows and macOS.

- Public IPv4 and IPv6, plus local interface addresses, at a glance
- Logs every change and outage with timestamp, provider and location
- Label addresses ("Home", "Office VPN") so reports read like English
- System notifications when the address changes or the connection drops
- Reports: online time, changes per day, time on each address, providers seen
- Webhook and shell-command hooks on every event (dynamic DNS, chat alerts, scripts)
- Pause/resume from the tray, auto-start on login, single-instance, remembers window position
- Export history to CSV or JSON; automatic retention
- Light, dark and system themes
- Everything is stored locally in SQLite. No accounts, no server.

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| ⌘/Ctrl + 1–4 | Switch between Overview, History, Reports, Settings |
| ⌘/Ctrl + R | Check now |

## Hooks

**Webhook** — set a URL in Settings and every event is POSTed as JSON:

```json
{
  "event": "change",
  "timestamp": "2026-09-28T10:15:00Z",
  "public_ipv4": "203.0.113.42",
  "previous_ipv4": "203.0.113.41",
  "public_ipv6": null,
  "previous_ipv6": null,
  "isp": "Example ISP",
  "country": "Bangladesh",
  "city": "Dhaka",
  "label": "Home",
  "local_ips": ["192.168.1.10"]
}
```

`event` is one of `change`, `offline`, `online` (`start` is not sent). "Send test" in Settings posts a sample payload with `"test": true`.

**Shell command** — runs after each event with these environment variables: `IP_EVENT`, `IP_TIMESTAMP`, `IP_NEW`, `IP_OLD`, `IP_NEW_V6`, `IP_OLD_V6`, `IP_ISP`, `IP_LABEL`. Example dynamic-DNS update:

```sh
curl -s "https://www.duckdns.org/update?domains=myhost&token=$DUCKDNS_TOKEN&ip=$IP_NEW"
```

## Install

Download the latest build for your OS from the [Releases](../../releases) page:

| OS | File |
|---|---|
| Windows | `IP Tracker_x.y.z_x64-setup.exe` or `.msi` |
| macOS | `IP Tracker_x.y.z_aarch64.dmg` (Apple Silicon) / `_x64.dmg` (Intel) |
| Linux | `.AppImage` or `.deb` |

macOS builds are not notarized yet. If Gatekeeper blocks it: right-click → Open, or run `xattr -d com.apple.quarantine "/Applications/IP Tracker.app"`.

## Privacy

The app makes exactly two kinds of network requests:

1. **Public IP lookup** — a plain-text request to `api4.ipify.org` / `api6.ipify.org`, falling back to `icanhazip.com`, `ifconfig.me`, `ipinfo.io`.
2. **ISP / location lookup** — your new public IP is sent to `ip-api.com` when it changes. You can turn this off in Settings.

Nothing else leaves your machine unless you configure a webhook or command yourself.

## Build from source

Prerequisites: [Node 22+](https://nodejs.org), [Rust stable](https://rustup.rs), and the [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your OS.

```bash
git clone https://github.com/helloashif/ip-tracker-desktop
cd ip-tracker-desktop
npm install
npm run tauri dev      # run with hot reload
npm run tauri build    # produce installers in src-tauri/target/release/bundle
```

To regenerate the icon set from `app-icon.png`:

```bash
npm run tauri icon app-icon.png
```

## Project layout

```
src/                 Vue 3 + TypeScript UI
  components/        Dashboard, History, Reports, Settings + ui/ primitives
  lib/               Typed Tauri API, theme, toasts, confirm dialog, formatting
src-tauri/
  src/lib.rs         App setup, tray, polling loop, commands
  src/ip.rs          Public IPv4/IPv6 + geo lookup, local interface listing
  src/db.rs          SQLite schema + migrations, queries, stats, labels, export
  src/hooks.rs       Webhook and shell-command hooks
  src/settings.rs    settings.json load/save
.github/workflows/   CI and cross-platform release builds
```

Data lives in the OS app-data directory (`~/.local/share/dev.ashif.iptracker` on Linux, `%APPDATA%\dev.ashif.iptracker` on Windows, `~/Library/Application Support/dev.ashif.iptracker` on macOS).

## Releasing

Releases are automatic. Bump the version and push to `main`:

```bash
npm version patch      # or minor / major — updates package.json (tauri.conf.json reads it)
git push --follow-tags
```

The Release workflow sees a version with no matching `v*` tag, builds installers for Linux, Windows and macOS (Intel and Apple Silicon), creates the tag and publishes the GitHub Release with everything attached. Pushes that don't change the version are skipped. You can also trigger it manually from the Actions tab.

## Contributing

Issues and pull requests are welcome. Before opening a PR run `npm run build`, and in `src-tauri` run `cargo fmt` and `cargo clippy -- -D warnings`.

## License

MIT
