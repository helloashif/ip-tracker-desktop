# Contributing

Thanks for helping out.

## Setup

1. Install the Tauri prerequisites for your OS: https://tauri.app/start/prerequisites/
2. `npm install`
3. `npm run tauri dev`

## Before you open a PR

- `npm run build` passes (type-check + Vite build)
- `cargo fmt` and `cargo clippy -- -D warnings` pass in `src-tauri`
- Keep PRs focused on one change

## Ideas that would be welcome

- Built-in dynamic DNS providers (DuckDNS, Cloudflare, No-IP) as a guided form on top of the hook system
- Auto-update via tauri-plugin-updater (needs a signing key in CI)
- Translations
- Signed/notarized macOS builds
- Homebrew, winget, Flathub and AUR packaging
