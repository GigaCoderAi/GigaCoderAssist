# GigaCoder Config Assistant Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri desktop app that logs into GigaCoder, lets a user choose an API key, and writes Claude Code and Codex configuration on macOS and Windows.

**Architecture:** Vue owns the setup wizard UI. Rust owns all privileged-ish work: calling GigaCoder APIs, resolving platform paths, backing up existing files, merging JSON/TOML, and exposing Tauri commands. Tests focus on file merge behavior using temporary directories so the real home directory is never touched.

**Tech Stack:** Tauri 2, Vue 3, TypeScript, Vite, Rust, reqwest, serde, serde_json, toml_edit, dirs, tempfile, thiserror.

---

### File Structure

- Create `package.json`, `index.html`, `vite.config.ts`, `tsconfig.json`: frontend build setup.
- Create `src/main.ts`, `src/App.vue`, `src/style.css`, `src/types.ts`: wizard UI and Tauri command calls.
- Create `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`: Tauri app setup.
- Create `src-tauri/src/main.rs`: Tauri command registration.
- Create `src-tauri/src/error.rs`: shared app error type.
- Create `src-tauri/src/gigacoder/{mod.rs,client.rs,types.rs}`: GigaCoder auth and key APIs.
- Create `src-tauri/src/config/{mod.rs,paths.rs,backup.rs,claude.rs,codex.rs}`: config writers and tests.

### Task 1: Project Skeleton

- [ ] **Step 1: Create frontend and Tauri manifests**

Create the package manifests and Tauri config so `npm install`, `npm run build`, and `cargo test` have a valid project to load.

- [ ] **Step 2: Create minimal Vue app**

Create a one-screen shell that can later call Rust commands.

- [ ] **Step 3: Create Rust crate**

Create a Tauri crate with module placeholders and a compilable `main.rs`.

### Task 2: Config Writers With Tests

- [ ] **Step 1: Write failing Rust tests**

Add tests proving:

- Claude writer creates `.claude/settings.json` and writes the expected `env`.
- Claude writer preserves unrelated existing `env` keys.
- Codex writer creates `.codex/config.toml` and `.codex/auth.json`.
- Codex writer preserves unrelated TOML keys.
- Existing files are backed up before writes.

- [ ] **Step 2: Run tests and verify failure**

Run `cargo test` from `src-tauri`. Expected: tests fail because writer functions are missing.

- [ ] **Step 3: Implement config writers**

Implement directory creation, timestamped backup, JSON merge, TOML merge, and verification-friendly return values.

- [ ] **Step 4: Run tests and verify pass**

Run `cargo test` from `src-tauri`. Expected: config writer tests pass.

### Task 3: GigaCoder API Client

- [ ] **Step 1: Add API types**

Define typed request/response structures for `/api/v1/auth/login`, `/api/v1/keys`, and `/api/v1/external/auth/login`.

- [ ] **Step 2: Implement client methods**

Implement `login`, `list_keys`, `external_login`, and `fetch_available_keys`, where `fetch_available_keys` falls back to external login when no usable key exists.

- [ ] **Step 3: Add unit tests for key filtering**

Test pure filtering behavior without network: active keys with `raw_key` are selectable, inactive/expired/missing raw key are not selectable.

### Task 4: Tauri Commands

- [ ] **Step 1: Expose commands**

Expose `login_and_fetch_keys`, `apply_configuration`, and `preview_configuration`.

- [ ] **Step 2: Normalize output**

Return masked key display values to the UI and never return passwords in any response.

### Task 5: Wizard UI

- [ ] **Step 1: Implement login screen**

Collect email/password and call `login_and_fetch_keys`.

- [ ] **Step 2: Implement key and target selection**

Let the user choose one key and one or both targets: Claude Code, Codex.

- [ ] **Step 3: Implement preview and apply**

Show target files, then call `apply_configuration` and render written/backup paths.

### Task 6: Verification

- [ ] **Step 1: Run Rust tests**

Run `cargo test` from `src-tauri`.

- [ ] **Step 2: Run frontend build**

Run `npm run build`.

- [ ] **Step 3: Run Tauri build check**

Run `cargo check` from `src-tauri`; if dependencies are unavailable, report the exact blocker.

