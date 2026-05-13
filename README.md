# GigaCoderAssist

GigaCoderAssist is a desktop configuration assistant for setting up GigaCoder credentials in Claude Code and OpenAI Codex-compatible clients. It is built with Vue 3, Vite, TypeScript, Rust, and Tauri 2.

The app provides a guided flow: log in with a GigaCoder account, fetch an available API key and model list, choose whether to configure Claude Code, Codex, or both, preview the files that will be written, and then apply the configuration with backups.

## Code Overview

The frontend lives in `src/`:

- `src/App.vue` implements the full wizard UI, language switcher, login form, API key selection, target selection, preview step, and result page.
- `src/types.ts` defines the TypeScript types returned by the Tauri commands.
- `src/style.css` contains the application styling.
- `src/main.ts` mounts the Vue app.

The native backend lives in `src-tauri/src/`:

- `main.rs` exposes Tauri commands used by the frontend: login, preview configuration, and apply configuration.
- `gigacoder/client.rs` handles GigaCoder API calls, API key filtering, OpenAI-compatible model detection, proxy support, and model sorting.
- `gigacoder/types.rs` defines API response payloads.
- `config/claude.rs` writes Claude Code settings.
- `config/codex.rs` writes Codex config, auth, and `custom_models.json`.
- `config/model_catalog_template.json` is the Codex model catalog template used to preserve required fields such as `base_instructions` and `model_messages`.
- `config/backup.rs` creates backups before overwriting existing config files.
- `config/paths.rs` centralizes target paths under the user's home directory.

Release automation lives in `.github/workflows/release.yml`. Pushing a `v*` tag builds and publishes Windows, macOS Intel, and macOS Apple Silicon release assets.

## What It Configures

When Claude Code is selected, the app writes:

```text
~/.claude/settings.json
```

When Codex is selected, the app writes:

```text
~/.codex/config.toml
~/.codex/auth.json
~/.codex/custom_models.json
```

Codex models are generated from the OpenAI-compatible model list returned by the GigaCoder login/API flow. Image models such as `gpt-image*` are excluded from the Codex model catalog, and text models are sorted by natural version order so newer GPT models appear first in Codex's `/model` list.

Existing files are backed up before they are overwritten.

## Requirements

- Node.js 22 or newer
- npm
- Rust stable toolchain
- Platform-specific Tauri build dependencies
- A GigaCoder account and API access from `www.gigacoder.org`

## Development

Install dependencies:

```bash
npm ci
```

Run the frontend development server:

```bash
npm run dev
```

Run the Tauri desktop app in development mode:

```bash
npm run tauri dev
```

Run frontend type checks and production frontend build:

```bash
npm run build
```

Run Rust tests:

```bash
cd src-tauri
cargo test
```

## Build Locally

Build the desktop app and platform installer:

```bash
npm run tauri build
```

On macOS, the release binary is written to:

```text
src-tauri/target/release/gigacoder-config-assistant
```

The DMG installer is written to:

```text
src-tauri/target/release/bundle/dmg/
```

## GitHub Release

Releases are triggered by pushing a version tag:

```bash
git tag v0.1.1
git push origin v0.1.1
```

The release workflow syncs package metadata from the pushed tag before building, so a `v0.1.1` tag produces `0.1.1` app packages.

The GitHub Actions workflow builds:

- Windows: NSIS installer
- macOS Intel: DMG for `x86_64-apple-darwin`
- macOS Apple Silicon: DMG for `aarch64-apple-darwin`

The workflow publishes the build artifacts to the GitHub Release for that tag.

macOS release builds use temporary ad-hoc signing with `signingIdentity: "-"`. This allows CI to produce Intel and Apple Silicon DMG packages without Apple Developer credentials. Because the app is not notarized, users may still need to allow it manually in macOS Security settings after download.

## Using The App

1. Open GigaCoderAssist.
2. Log in with your GigaCoder account.
3. Select an active API key.
4. Choose the tools you want to configure: Claude Code, Codex, or both.
5. Review the target files in the preview step.
6. Apply the configuration.
7. Restart Claude Code or Codex so the new settings are loaded.

For Codex, run `/model` after restarting to verify that the GigaCoder OpenAI-compatible models are available.

## About GigaCoder

GigaCoder, available at `https://www.gigacoder.org`, provides developer-focused AI access for coding workflows. It exposes OpenAI-compatible endpoints and model catalogs that can be used by tools such as Codex and Claude Code through local configuration.

GigaCoderAssist is designed to make that setup process safer and faster: it retrieves the correct account key, writes the required local config files, preserves backups, and prepares a Codex-compatible model catalog automatically.

If your team uses multiple coding assistants or wants a simpler way to manage AI coding model access, GigaCoder provides a centralized service layer while GigaCoderAssist handles the local developer setup.
