# Design: Port n-link desktop to Tauri 2 + Vue 3

Date: 2026-07-19
Status: Approved
Branch: `port/tauri2-vue3`

## Problem

The n-link desktop app cannot build or run on Ubuntu 26.04. `desktop/src-tauri`
pins `tauri = "1.0.0-beta.8"`, which links against `webkit2gtk-4.0`. That library
is not available on this system — only `webkit2gtk-4.1` is. The prebuilt 0.1.6
`.deb` fails for the same reason, plus a missing `libssl.so.1.1` (only
`libssl.so.3` is present).

This is the blocker that caused `nlink-cli` to be written as a GUI-free
workaround. This project removes the blocker properly so the GUI works.

Tauri 2.x links against `webkit2gtk-4.1`, which is what makes the port the fix.

### Verified environment

| Component | State |
|---|---|
| `webkit2gtk-4.0` | absent, not installable |
| `libwebkit2gtk-4.1-dev` | available, 2.52.3-0ubuntu0.26.04.2 (not yet installed) |
| `libsoup-3.0-dev` | available, 3.6.6-1 (not yet installed) |
| `libssl` | 3 only; no 1.1 |
| `libusb-1.0` | 1.0.29, installed |
| rustc / cargo | 1.97.1 |
| node | v22.22.1 |
| yarn | **not installed** |
| TI-Nspire CX II | connected, USB `0451:e022` |

## Scope

In scope:

1. `n-link-core` — port 8 components from Vue 2 to Vue 3.
2. `desktop` — port to Tauri 2, Vue 3, and Vite; bundle deb + appimage.

Out of scope (deferred to a separate project):

- `web` (Nuxt 2). It shares `n-link-core`, so porting core **knowingly breaks
  it**. `web/` is left untouched at its current commit rather than
  half-migrated. It is a browser/WebUSB product and does not bear on the
  Linux GUI problem.
- Windows and macOS targets. `msi` and `dmg` are dropped from the bundle
  config; they cannot be verified from this machine.

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Platform targets | Linux only; deb + appimage | Fully testable here |
| Frontend stack | Vue 3 + Vite | Vue CLI 4.5 / webpack 4 fails on Node 22 (OpenSSL 3); `vue-cli-plugin-tauri` is Tauri-v1-era and abandoned |
| Shared core | Port `n-link-core` to Vue 3 | `web` deferred, accepted as broken |
| Component style | `<script setup>` + Composition API | `vue-class-component` has no Vue 3 future |
| Migration method | Scaffold clean, port code across | v1→v2 config is a near-total rewrite; a generated baseline means only our code is ever in question |
| Verification | Manual, `n-link-cli` as oracle | Repo has zero tests; risk concentrates in USB behavior needing real hardware |

## Approach

Scaffold a fresh Tauri 2 + Vue 3 + Vite + TypeScript app, then move logic into
it. Backend goes first — this is forced, not preference: the Vue 3 frontend
cannot be exercised until a Tauri 2 shell runs, because Tauri 1 cannot launch
on this machine at all.

"Scaffold clean" means the generated baseline **replaces the contents of
`desktop/`** in place, on the `port/tauri2-vue3` branch — it is not a new
top-level package. The workspace keeps its three members (`desktop`,
`n-link-core`, `web`). The old `desktop/src` and `desktop/src-tauri` are
removed in the same commit that adds the scaffold, so history shows one
coherent replacement rather than a half-migrated intermediate state.

Trade-off accepted: per-file `git blame` continuity in `desktop/` is lost.
Given that `tauri.conf.json`, the build tooling, and every component's syntax
are all being replaced anyway, that continuity is largely notional.

## Architecture

```
desktop/
  src-tauri/
    Cargo.toml                   tauri 2.x, patched libnspire-sys
    tauri.conf.json              v2 schema
    capabilities/default.json    replaces v1 allowlist
    src/
      main.rs                    thin: CLI passthrough + Builder
      device.rs                  Device/DeviceState/DEVICES registry
      commands.rs                the 13 #[tauri::command] wrappers
      error.rs                   SerializedError
      cli.rs                     headless mode
  src/                           Vue 3 + Vite
```

`main.rs` is currently 478 lines holding the device registry, hotplug monitor,
all 13 commands, and `main()`. Splitting it into focused modules is a targeted
improvement to code being rewritten end-to-end anyway — not unrelated
refactoring.

The patched `vendor/libnspire-sys` (CX II NNSE handshake retry limit raised
10 → 100) is wired via `[patch.crates-io]`, exactly as `nlink-cli` already does.

## IPC contract — held invariant

Deliberately unchanged, so the frontend and backend ports can be verified
independently.

**13 commands:** `enumerate`, `open_device`, `close_device`, `update_device`,
`list_dir`, `download_file`, `upload_file`, `upload_os`, `delete_file`,
`delete_dir`, `create_nspire_dir`, `move_file`, `copy`

**3 events:** `addDevice`, `removeDevice`, `progress`

**Payloads:** `DevId{busNumber,address}`, `AddDevice`, `ProgressUpdate`,
`FileInfo` — all `camelCase` via serde, unchanged.

## Data flow

`rusb` hotplug thread → `DeviceMon` → emits `addDevice`/`removeDevice` → Vue
store in `devices.ts`. Commands mutate the global `DEVICES` map behind a
`RwLock`. Long transfers emit `progress`.

One frontend change: the ~10 `$set`/`$delete` calls in `devices.ts` become
plain assignment and `delete`. Vue 3 proxy reactivity makes them unnecessary —
a deletion, not a rewrite.

## Error handling

Unchanged in shape. `SerializedError` wraps `libnspire::Error` and
`rusb::Error` into a serde-serializable form for crossing IPC. Kept as-is,
relocated to `error.rs`.

## Migration hazards

Specific traps identified in the existing code:

| Hazard | Detail |
|---|---|
| `on_page_load` | Signature changed in v2 (`&Webview` + payload). Hotplug registration at `main.rs:440` must move; `setup()` is the better home |
| `Window` → `WebviewWindow` | `emit` additionally requires `use tauri::Emitter` in scope |
| `drain_filter` | `cmd.rs:62`, hashbrown 0.11 → renamed `extract_if`, **with changed semantics** |
| `allowlist` → capabilities | allowlist block deleted; `dialog:allow-open` + `shell:allow-open` become plugin permissions |
| Plugins | `dialog` and `shell` are separate crates + JS packages in v2. `notification-all` is in Cargo features but unused by the frontend — dropped |
| CSP | Commit `0472908` fixed a CSP blank-screen in release mode. v2 moves CSP to `app.security.csp`; this fix must be carried forward or release builds go blank again |
| `clap` 3.0.0-beta.2 | `cli.rs` uses it — but `nlink-cli/src/cli.rs` is **already** the clap-4 port of this same file. Reuse it instead of redoing the migration |
| element-ui | Only `el-popover`, 5 usages across 3 files. Swap to element-plus |
| Vue class components | 9 `@Component`, 18 `@Prop`, 6 `@Watch` across 9 files → `<script setup>` |
| yarn missing | Root `package.json` declares yarn workspaces but yarn is absent. Migrating to **npm workspaces** (npm ships with Node 22; the root `yarn.lock` is invalidated by this port's dependency changes regardless) |

## Build sequence

1. Install system deps (`libwebkit2gtk-4.1-dev`, `libsoup-3.0-dev`,
   `librsvg2-dev`, `libxdo-dev`, `libayatana-appindicator3-dev`). Requires sudo.
2. Scaffold Tauri 2 + Vue 3 + Vite shell. **Go/no-go gate.**
3. Port Rust backend; wire patched `libnspire-sys`.
4. Port `n-link-core` components to Vue 3 `<script setup>`.
5. Port `desktop/src` (`devices.ts`, `Home.vue`, router); drop `$set`/`$delete`.
6. Bundle deb + appimage.

## Verification

Each gate runs against the connected CX II.

1. **Go/no-go** — an empty Tauri 2 window opens on webkit2gtk-4.1. Cheap, and
   everything downstream is wasted if it fails, so it runs first.
2. **Backend** — `enumerate` sees the device; `list_dir /` matches
   `n-link-cli ls /`.
3. **Frontend** — components render; device appears in the UI.
4. **End-to-end** — download a `.tns`; `sha256sum` matches the same file
   fetched via `n-link-cli download`.
5. **Bundle** — deb and appimage build; installed deb launches.

### Known ceiling, not a regression

Per `nlink-cli/README.md`: libnspire's CX II support has been frozen upstream
since 2020, and against 2026 firmware, multi-packet transfers are unreliable —
long operations can fail mid-transfer with `Busy`.

This ceiling is inherited by the GUI. Step 4 therefore uses a **short**
transfer, and any failure is compared against `n-link-cli` behavior before
being treated as a port regression.

## Risks

- **Go/no-go fails.** If webkit2gtk-4.1 cannot produce a window, the whole
  approach is void. Mitigated by making it step 2, at near-zero cost.
- **Vue 3 reactivity drift.** Removing `$set`/`$delete` is where silent
  behavioral regressions are most likely. Caught by oracle cross-check at
  gate 2 and 4, not by tests.
- **`web` stays broken.** Accepted and explicit; tracked as follow-up work.
