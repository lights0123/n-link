# n-link Desktop — Tauri 2 Backend Port (Plan A)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Get a Tauri 2 shell running on Ubuntu 26.04 with the full n-link Rust backend ported and verified against a physically connected TI-Nspire CX II.

**Architecture:** Scaffold a clean Tauri 2 + Vue 3 + Vite baseline that replaces the contents of `desktop/`, then port the Rust backend into it as focused modules (`device.rs`, `commands.rs`, `error.rs`, `cli.rs`) rather than one 478-line `main.rs`. The IPC contract — 13 commands, 3 events, camelCase payloads — is held byte-identical so the frontend port (Plan B) is a separate, independently verifiable change.

**Tech Stack:** Rust 1.97.1, tauri 2.11.5, tauri-plugin-dialog 2.7.2, tauri-plugin-shell 2.3.5, libnspire 0.2.3, rusb 0.6.4, clap 4, Vite 8.1.5, Vue 3.5.40, Node 22.22.1, npm workspaces.

**Source of truth:** `docs/superpowers/specs/2026-07-19-desktop-tauri2-vue3-port-design.md`

## Global Constraints

- Branch: `port/tauri2-vue3`. Do not commit to `main`.
- Linux only. Bundle targets are exactly `["deb", "appimage"]`. Never add `msi` or `dmg`.
- `web/` is **out of scope and knowingly broken**. Do not modify any file under `web/`.
- `vendor/libnspire-sys` is wired via `[patch.crates-io]`. Its raised CX II NNSE handshake retry limit (10 → 100) is required; do not drop the patch or bump `libnspire-sys` past it.
- The IPC contract is frozen. Command names, event names, and payload field names (camelCase via serde) must not change.
- Do not commit `target/` or `node_modules/`.
- No test framework is introduced. The spec approved **manual verification with `n-link-cli` as oracle**; verification steps below replace the usual TDD cycle. This is a deliberate, recorded decision, not an omission.

**Oracle binary:** `/home/jjureta/Projects/n-link/nlink-cli/target/release/n-link-cli`

---

### Task 1: Install system dependencies and confirm the toolchain

**Files:** none (environment only)

**Interfaces:**
- Consumes: nothing
- Produces: a system on which `pkg-config --modversion webkit2gtk-4.1` succeeds; prerequisite for every later task.

- [ ] **Step 1: Confirm the CX II is connected**

Run: `lsusb | grep 0451`
Expected: a line containing `0451:e022 Texas Instruments, Inc. Nspire CX II`

If absent, plug in the calculator and power it on before continuing. Every verification gate depends on it.

- [ ] **Step 2: Install the Tauri 2 system dependencies**

Requires sudo. Run:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libjavascriptcoregtk-4.1-dev \
  libsoup-3.0-dev \
  librsvg2-dev \
  libxdo-dev \
  libayatana-appindicator3-dev \
  build-essential curl wget file
```

- [ ] **Step 3: Verify every required library now resolves**

Run:

```bash
for p in webkit2gtk-4.1 javascriptcoregtk-4.1 libsoup-3.0 gtk+-3.0 libusb-1.0; do
  printf "%-24s " "$p"; pkg-config --modversion "$p" || echo MISSING
done
```

Expected: five version numbers, no `MISSING`. `webkit2gtk-4.1` should report `2.52.x`.

**If any line reports MISSING, stop.** The port cannot proceed and the spec's core premise has failed.

- [ ] **Step 4: Verify the oracle binary works**

Run: `/home/jjureta/Projects/n-link/nlink-cli/target/release/n-link-cli ls /`
Expected: a directory listing from the calculator.

If the binary is missing, build it: `cd /home/jjureta/Projects/n-link/nlink-cli && cargo build --release`

If it errors with a USB permission problem, apply the udev rule documented in `nlink-cli/README.md` before continuing.

- [ ] **Step 5: Record the oracle baseline**

Run and save the output — later gates compare against it:

```bash
/home/jjureta/Projects/n-link/nlink-cli/target/release/n-link-cli ls / > /tmp/oracle-root-listing.txt
cat /tmp/oracle-root-listing.txt
```

No commit for this task — it changes no files.

---

### Task 2: Scaffold the Tauri 2 baseline (GO/NO-GO GATE)

**Files:**
- Delete: `desktop/src/`, `desktop/src-tauri/`, `desktop/babel.config.js`, `desktop/vue.config.js`, `desktop/.browserslistrc`, `desktop/postcss.config.js`, `desktop/tailwind.config.js`, `desktop/tsconfig.json`, `desktop/.eslintrc.js`
- Create: `desktop/` scaffold (package.json, vite.config.ts, index.html, src/, src-tauri/)
- Preserve: `desktop/app-icon.png`, `desktop/app-icon.svg`, `desktop/LICENSE`, `desktop/README.md`

**Interfaces:**
- Consumes: system deps from Task 1
- Produces: a runnable Tauri 2 project at `desktop/`, with `src-tauri/Cargo.toml` (package name `n-link`) and `src-tauri/tauri.conf.json` (v2 schema). Task 3 onward add modules under `desktop/src-tauri/src/`.

- [ ] **Step 1: Preserve the files worth keeping**

```bash
cd /home/jjureta/Projects/n-link/desktop
mkdir -p /tmp/nlink-keep
cp app-icon.png app-icon.svg LICENSE README.md /tmp/nlink-keep/
cp src-tauri/src/cli.rs /tmp/nlink-keep/cli-old.rs
cp src-tauri/src/main.rs /tmp/nlink-keep/main-old.rs
cp src-tauri/src/cmd.rs /tmp/nlink-keep/cmd-old.rs
cp src-tauri/tauri.conf.json /tmp/nlink-keep/tauri.conf.old.json
ls /tmp/nlink-keep
```

Expected: 8 files listed. These are reference copies; git history also retains them.

- [ ] **Step 2: Remove the Vue CLI / Tauri 1 tree**

```bash
cd /home/jjureta/Projects/n-link/desktop
git rm -r --quiet src src-tauri
git rm --quiet babel.config.js vue.config.js .browserslistrc postcss.config.js tailwind.config.js tsconfig.json .eslintrc.js package.json
git status --short
```

Expected: deletions staged. `app-icon.*`, `LICENSE`, `README.md` remain untouched.

- [ ] **Step 3: Scaffold a fresh Tauri 2 + Vue + TS project**

Run from the repo root so the scaffold lands in `desktop/`:

```bash
cd /home/jjureta/Projects/n-link
npm create tauri-app@latest -- \
  --directory . \
  --name desktop \
  --manager npm \
  --template vue-ts \
  --identifier com.lights0123.n-link \
  --yes
```

If the CLI refuses to write into the non-empty `desktop/`, scaffold to a temp dir and move the generated files in:

```bash
cd /tmp && rm -rf nlink-scaffold
npm create tauri-app@latest nlink-scaffold -- --manager npm --template vue-ts --identifier com.lights0123.n-link --yes
cp -r /tmp/nlink-scaffold/. /home/jjureta/Projects/n-link/desktop/
rm -rf /home/jjureta/Projects/n-link/desktop/.git
```

- [ ] **Step 4: Restore the preserved assets**

```bash
cd /home/jjureta/Projects/n-link/desktop
cp /tmp/nlink-keep/app-icon.png /tmp/nlink-keep/app-icon.svg /tmp/nlink-keep/LICENSE /tmp/nlink-keep/README.md .
ls app-icon.png app-icon.svg LICENSE README.md
```

Expected: all four present.

- [ ] **Step 5: Convert the workspace from yarn to npm**

Replace `/home/jjureta/Projects/n-link/package.json` with:

```json
{
  "private": true,
  "workspaces": [
    "desktop",
    "n-link-core",
    "web"
  ]
}
```

Then remove the stale lockfile — this port invalidates it:

```bash
cd /home/jjureta/Projects/n-link
git rm --quiet yarn.lock
```

- [ ] **Step 6: Install dependencies**

```bash
cd /home/jjureta/Projects/n-link
npm install --workspace desktop
```

Expected: completes without error.

**If it fails resolving `web/`'s dependencies** (npm reads every workspace member's manifest even when installing just one, and `web/` is on Nuxt 2 with a 2020-era tree), temporarily drop `web` from the workspaces array:

```json
{
  "private": true,
  "workspaces": ["desktop", "n-link-core"]
}
```

This is consistent with the spec — `web/` is knowingly out of scope and left broken. Record it in the commit message so Plan C knows to restore the entry.

- [ ] **Step 7: GO/NO-GO — launch the scaffold**

```bash
cd /home/jjureta/Projects/n-link/desktop
npm run tauri dev
```

Expected: a native window opens showing the default Tauri + Vue welcome page.

**This is the decisive gate.** If the window opens, webkit2gtk-4.1 works and the entire approach is validated. If it fails to build or open, stop and report the exact error — everything downstream depends on this and no further work is worthwhile until it is resolved.

Close the window with Ctrl+C in the terminal.

- [ ] **Step 8: Commit the scaffold**

```bash
cd /home/jjureta/Projects/n-link
cat >> .gitignore <<'EOF'

# build artifacts
node_modules/
target/
dist/
EOF
git add -A
git commit -m "feat(desktop): replace Tauri 1 / Vue CLI tree with Tauri 2 + Vue 3 + Vite scaffold

Tauri 1.0.0-beta.8 links webkit2gtk-4.0, which is unavailable on
Ubuntu 26.04. Tauri 2 links webkit2gtk-4.1. Verified: scaffold window
opens.

Workspace moved from yarn to npm; yarn.lock invalidated by this port."
```

---

### Task 3: Port the error type and device registry

**Files:**
- Create: `desktop/src-tauri/src/error.rs`
- Create: `desktop/src-tauri/src/device.rs`
- Modify: `desktop/src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: the scaffold from Task 2.
- Produces:
  - `error::SerializedError` — `#[derive(Serialize)] pub struct SerializedError(String)`, with a blanket `From<T: Display>`.
  - `device::DevId { bus_number: u8, address: u8 }` — `Copy + Serialize + Deserialize`, serialized `camelCase` as `busNumber`/`address`.
  - `device::Device { name: String, device: Arc<rusb::Device<GlobalContext>>, state: DeviceState, needs_drivers: bool }`
  - `device::DeviceState::{Open(Arc<Mutex<libnspire::Handle<GlobalContext>>>, libnspire::info::Info), Closed}`
  - `device::DEVICES: RwLock<HashMap<(u8, u8), Device>>` (std HashMap, **not** hashbrown)
  - `device::add_device(Arc<rusb::Device<GlobalContext>>) -> rusb::Result<((u8, u8), Device)>`
  - `device::get_open_dev(&DevId) -> Result<Arc<Mutex<libnspire::Handle<GlobalContext>>>, anyhow::Error>`
  - `device::{AddDevice, ProgressUpdate, FileInfo}` payload structs

- [ ] **Step 1: Set the backend dependencies**

Replace the `[dependencies]` section of `desktop/src-tauri/Cargo.toml` so the file reads:

```toml
[package]
name = "n-link"
version = "0.1.6"
description = "Free, cross-platform, CX-II compatible computer linking program for the TI-Nspire"
authors = ["Ben Schattinger <developer@lights0123.com>"]
license = "GPL-3.0"
repository = "https://github.com/lights0123/n-link"
default-run = "n-link"
edition = "2021"
build = "build.rs"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2.11.5", features = [] }
tauri-plugin-dialog = "2.7.2"
tauri-plugin-shell = "2.3.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
lazy_static = "1.4"
libnspire = "0.2.3"
rusb = "0.6.4"
clap = { version = "4", features = ["derive"] }
indicatif = "0.15"

[patch.crates-io]
libnspire-sys = { path = "../../vendor/libnspire-sys" }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]

[[bin]]
name = "n-link"
path = "src/main.rs"
```

Two deliberate changes from the original, both required:

1. **`hashbrown` is dropped.** The old `cmd.rs:62` used `drain_filter`, a hashbrown 0.11 API that was renamed to `extract_if` with changed semantics. Task 4 replaces that call with an explicit collect-then-remove on a std `HashMap`, which is stable and needs no third-party map.
2. **`libusb1-sys` vendored is dropped.** `nlink-cli` established that the vendored build is broken on this toolchain; the system `libusb-1.0` (1.0.29, already installed) is used instead.

Note the patch path is `../../vendor/libnspire-sys` — two levels up from `desktop/src-tauri/`, unlike `nlink-cli`'s one level.

- [ ] **Step 2: Write `error.rs`**

Create `desktop/src-tauri/src/error.rs`:

```rust
use serde::Serialize;

/// Wraps any displayable error into a serde-serializable form so it can
/// cross the Tauri IPC boundary.
#[derive(Serialize)]
pub struct SerializedError(String);

impl<T: std::fmt::Display> From<T> for SerializedError {
  fn from(f: T) -> Self {
    SerializedError(f.to_string())
  }
}
```

- [ ] **Step 3: Write `device.rs`**

Create `desktop/src-tauri/src/device.rs`:

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use libnspire::{PID, PID_CX2, VID};
use rusb::GlobalContext;
use serde::{Deserialize, Serialize};

pub enum DeviceState {
  Open(
    Arc<Mutex<libnspire::Handle<GlobalContext>>>,
    libnspire::info::Info,
  ),
  Closed,
}

pub struct Device {
  pub name: String,
  pub device: Arc<rusb::Device<GlobalContext>>,
  pub state: DeviceState,
  pub needs_drivers: bool,
}

lazy_static::lazy_static! {
  pub static ref DEVICES: RwLock<HashMap<(u8, u8), Device>> = RwLock::new(HashMap::new());
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct DevId {
  pub bus_number: u8,
  pub address: u8,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDevice {
  #[serde(flatten)]
  pub dev: DevId,
  pub name: String,
  pub is_cx_ii: bool,
  pub needs_drivers: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressUpdate {
  #[serde(flatten)]
  pub dev: DevId,
  pub remaining: usize,
  pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
  pub path: String,
  pub is_dir: bool,
  pub date: u64,
  pub size: u64,
}

pub fn add_device(dev: Arc<rusb::Device<GlobalContext>>) -> rusb::Result<((u8, u8), Device)> {
  let descriptor = dev.device_descriptor()?;
  if !(descriptor.vendor_id() == VID && matches!(descriptor.product_id(), PID | PID_CX2)) {
    return Err(rusb::Error::Other);
  }

  let (name, needs_drivers) = match dev.open() {
    Ok(handle) => (
      handle.read_product_string(
        handle.read_languages(Duration::from_millis(100))?[0],
        &descriptor,
        Duration::from_millis(100),
      )?,
      false,
    ),
    Err(rusb::Error::NotSupported) | Err(rusb::Error::Access) => (
      if descriptor.product_id() == PID_CX2 {
        "TI-Nspire CX II"
      } else {
        "TI-Nspire"
      }
      .to_string(),
      true,
    ),
    Err(other) => return Err(other),
  };

  Ok((
    (dev.bus_number(), dev.address()),
    Device {
      name,
      device: dev,
      state: DeviceState::Closed,
      needs_drivers,
    },
  ))
}

pub fn get_open_dev(
  dev: &DevId,
) -> Result<Arc<Mutex<libnspire::Handle<GlobalContext>>>, anyhow::Error> {
  if let Some(dev) = DEVICES.read().unwrap().get(&(dev.bus_number, dev.address)) {
    match &dev.state {
      DeviceState::Open(handle, _) => Ok(handle.clone()),
      DeviceState::Closed => anyhow::bail!("Device closed"),
    }
  } else {
    anyhow::bail!("Failed to find device");
  }
}
```

- [ ] **Step 4: Wire the modules and check it compiles**

Replace `desktop/src-tauri/src/main.rs` with a temporary stub so the crate builds while later tasks fill it in:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod device;
mod error;

fn main() {
  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
```

Run: `cd /home/jjureta/Projects/n-link/desktop/src-tauri && cargo check`
Expected: compiles. Warnings about unused items in `device.rs`/`error.rs` are expected at this stage.

**If `libnspire-sys` fails to build**, confirm the patch path resolves: `ls ../../vendor/libnspire-sys/Cargo.toml` must exist.

- [ ] **Step 5: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/src-tauri/Cargo.toml desktop/src-tauri/Cargo.lock desktop/src-tauri/src/
git commit -m "feat(desktop): port device registry and error type to Tauri 2

Splits the former 478-line main.rs into focused modules. Drops hashbrown
in favour of std HashMap (the drain_filter API it was used for was
renamed with changed semantics) and drops vendored libusb1-sys in favour
of the system library, matching nlink-cli."
```

---

### Task 4: Port the 13 commands

**Files:**
- Create: `desktop/src-tauri/src/commands.rs`

**Interfaces:**
- Consumes: everything `device.rs` and `error.rs` produce in Task 3.
- Produces: 13 `#[tauri::command]` functions registered by Task 5 — `enumerate`, `open_device`, `close_device`, `update_device`, `list_dir`, `download_file`, `upload_file`, `upload_os`, `delete_file`, `delete_dir`, `create_nspire_dir`, `move_file`, `copy`. Also `commands::{err_wrap, progress_sender}` helpers, used only within this module.

Three Tauri 2 API changes apply throughout this file:

1. `Window<R>` → `WebviewWindow<R>`
2. `window.emit(...)` requires `use tauri::Emitter` in scope
3. `drain_filter` is gone — replaced with collect-then-remove

- [ ] **Step 1: Write `commands.rs`**

Create `desktop/src-tauri/src/commands.rs`:

```rust
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;

use libnspire::dir::EntryType;
use libnspire::PID_CX2;
use serde::Serialize;
use tauri::{Emitter, Runtime, WebviewWindow};

use crate::device::{
  add_device, get_open_dev, AddDevice, DevId, DeviceState, FileInfo, ProgressUpdate, DEVICES,
};
use crate::error::SerializedError;

/// If the device vanished mid-operation, drop it from the registry and tell
/// the frontend, then pass the original result through unchanged.
fn err_wrap<T, R: Runtime>(
  res: Result<T, libnspire::Error>,
  dev: DevId,
  window: &WebviewWindow<R>,
) -> Result<T, libnspire::Error> {
  if let Err(libnspire::Error::NoDevice) = res {
    DEVICES
      .write()
      .unwrap()
      .remove(&(dev.bus_number, dev.address));
    if let Err(msg) = window.emit("removeDevice", dev) {
      eprintln!("{}", msg);
    };
  }
  res
}

/// Emits a `progress` event on every 6th callback, and always on completion,
/// to avoid flooding the IPC channel during large transfers.
fn progress_sender<R: Runtime>(
  window: &WebviewWindow<R>,
  dev: DevId,
  total: usize,
) -> impl FnMut(usize) + '_ {
  let mut i = 0;
  move |remaining| {
    if i > 5 {
      i = 0;
    }
    if i == 0 || remaining == 0 {
      if let Err(msg) = window.emit(
        "progress",
        ProgressUpdate {
          dev,
          remaining,
          total,
        },
      ) {
        eprintln!("{}", msg);
      };
    }
    i += 1;
  }
}

#[tauri::command]
pub fn enumerate<R: Runtime>(window: WebviewWindow<R>) -> Result<Vec<AddDevice>, SerializedError> {
  let devices: Vec<_> = rusb::devices()?.iter().collect();
  let mut map = DEVICES.write().unwrap();

  // Replaces hashbrown's `drain_filter`: collect the keys of devices that are
  // no longer present on the bus, then remove them. std HashMap has no stable
  // drain-with-predicate, and this is clearer than the original anyway.
  let stale: Vec<(u8, u8)> = map
    .keys()
    .filter(|k| {
      devices
        .iter()
        .all(|d| d.bus_number() != k.0 || d.address() != k.1)
    })
    .copied()
    .collect();

  for key in stale {
    map.remove(&key);
    if let Err(msg) = window.emit(
      "removeDevice",
      DevId {
        bus_number: key.0,
        address: key.1,
      },
    ) {
      eprintln!("{}", msg);
    }
  }

  let filtered: Vec<_> = devices
    .into_iter()
    .filter(|d| !map.contains_key(&(d.bus_number(), d.address())))
    .collect();

  Ok(
    filtered
      .into_iter()
      .filter_map(|dev| add_device(Arc::new(dev)).ok())
      .map(|dev| {
        let msg = AddDevice {
          dev: DevId {
            bus_number: (dev.0).0,
            address: (dev.0).1,
          },
          name: (dev.1).name.clone(),
          is_cx_ii: (dev.1)
            .device
            .device_descriptor()
            .map(|d| d.product_id() == PID_CX2)
            .unwrap_or(false),
          needs_drivers: (dev.1).needs_drivers,
        };
        map.insert(dev.0, dev.1);
        msg
      })
      .collect(),
  )
}

#[tauri::command]
pub fn open_device(bus_number: u8, address: u8) -> Result<impl Serialize, SerializedError> {
  let device = if let Some(dev) = DEVICES.read().unwrap().get(&(bus_number, address)) {
    if !matches!(dev.state, DeviceState::Closed) {
      return Err("Already open".into());
    };
    dev.device.clone()
  } else {
    return Err("Failed to find device".into());
  };
  let handle = libnspire::Handle::new(device.open()?)?;
  let info = handle.info()?;
  {
    let mut guard = DEVICES.write().unwrap();
    let device = guard
      .get_mut(&(bus_number, address))
      .ok_or_else(|| anyhow::anyhow!("Device lost"))?;
    device.state = DeviceState::Open(Arc::new(std::sync::Mutex::new(handle)), info.clone());
  }
  Ok(info)
}

#[tauri::command]
pub fn close_device(bus_number: u8, address: u8) -> Result<impl Serialize, SerializedError> {
  let mut guard = DEVICES.write().unwrap();
  let device = guard
    .get_mut(&(bus_number, address))
    .ok_or_else(|| anyhow::anyhow!("Device lost"))?;
  device.state = DeviceState::Closed;
  Ok(())
}

#[tauri::command]
pub fn update_device<R: Runtime>(
  bus_number: u8,
  address: u8,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  let info = err_wrap(handle.info(), dev, &window)?;
  Ok(info)
}

#[tauri::command]
pub fn list_dir<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  let dir = err_wrap(handle.list_dir(&path), dev, &window)?;

  Ok(
    dir
      .iter()
      .map(|file| FileInfo {
        path: file.name().to_string_lossy().to_string(),
        is_dir: file.entry_type() == EntryType::Directory,
        date: file.date(),
        size: file.size(),
      })
      .collect::<Vec<_>>(),
  )
}

#[tauri::command]
pub fn download_file<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: (String, u64),
  dest: String,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let (file, size) = path;
  let dest = PathBuf::from(dest);
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  let mut buf = vec![0; size as usize];
  err_wrap(
    handle.read_file(
      &file,
      &mut buf,
      &mut progress_sender(&window, dev, size as usize),
    ),
    dev,
    &window,
  )?;
  if let Some(name) = file.split('/').last() {
    File::create(dest.join(name))?.write_all(&buf)?;
  }
  Ok(())
}

#[tauri::command]
pub fn upload_file<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  src: String,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let file = PathBuf::from(src);
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  let mut buf = vec![];
  File::open(&file)?.read_to_end(&mut buf)?;
  let name = file
    .file_name()
    .ok_or_else(|| anyhow::anyhow!("Failed to get file name"))?
    .to_string_lossy()
    .to_string();
  err_wrap(
    handle.write_file(
      &format!("{}/{}", path, name),
      &buf,
      &mut progress_sender(&window, dev, buf.len()),
    ),
    dev,
    &window,
  )?;
  Ok(())
}

#[tauri::command]
pub fn upload_os<R: Runtime>(
  bus_number: u8,
  address: u8,
  src: String,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  let mut buf = vec![];
  File::open(&src)?.read_to_end(&mut buf)?;
  err_wrap(
    handle.send_os(&buf, &mut progress_sender(&window, dev, buf.len())),
    dev,
    &window,
  )?;
  Ok(())
}

#[tauri::command]
pub fn delete_file<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  err_wrap(handle.delete_file(&path), dev, &window)?;
  Ok(())
}

#[tauri::command]
pub fn delete_dir<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  err_wrap(handle.delete_dir(&path), dev, &window)?;
  Ok(())
}

#[tauri::command]
pub fn create_nspire_dir<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  err_wrap(handle.create_dir(&path), dev, &window)?;
  Ok(())
}

#[tauri::command]
pub fn move_file<R: Runtime>(
  bus_number: u8,
  address: u8,
  src: String,
  dest: String,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  err_wrap(handle.move_file(&src, &dest), dev, &window)?;
  Ok(())
}

#[tauri::command]
pub fn copy<R: Runtime>(
  bus_number: u8,
  address: u8,
  src: String,
  dest: String,
  window: WebviewWindow<R>,
) -> Result<impl Serialize, SerializedError> {
  let dev = DevId {
    bus_number,
    address,
  };
  let handle = get_open_dev(&dev)?;
  let handle = handle.lock().unwrap();
  err_wrap(handle.copy_file(&src, &dest), dev, &window)?;
  Ok(())
}
```

Note `enumerate` now takes `window: WebviewWindow<R>` where the original took `handle: Window<R>` — same injected value, clearer name.

- [ ] **Step 2: Add the module and compile**

Add `mod commands;` to `desktop/src-tauri/src/main.rs` alongside the existing `mod device;` / `mod error;`.

Run: `cd /home/jjureta/Projects/n-link/desktop/src-tauri && cargo check`
Expected: compiles cleanly. Unused-function warnings are fine — Task 5 registers them.

- [ ] **Step 3: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/src-tauri/src/commands.rs desktop/src-tauri/src/main.rs
git commit -m "feat(desktop): port all 13 IPC commands to Tauri 2

Window -> WebviewWindow, emit now requires the Emitter trait, and
hashbrown's drain_filter is replaced by an explicit collect-then-remove
on std HashMap. IPC contract unchanged."
```

---

### Task 5: Wire the Tauri 2 builder, hotplug monitor, and config

**Files:**
- Modify: `desktop/src-tauri/src/main.rs`
- Modify: `desktop/src-tauri/tauri.conf.json`
- Create: `desktop/src-tauri/capabilities/default.json`

**Interfaces:**
- Consumes: `commands::*` from Task 4; `device::*` from Task 3.
- Produces: a running app that registers all 13 commands and emits `addDevice`/`removeDevice` on USB hotplug. Task 6 adds `cli::run()` ahead of the GUI launch.

- [ ] **Step 1: Write the v2 config**

Replace `desktop/src-tauri/tauri.conf.json` entirely:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "n-link",
  "version": "0.1.6",
  "identifier": "com.lights0123.n-link",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "N-Link",
        "width": 800,
        "height": 600,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src blob: data: filesystem: ws: wss: http: https: tauri: 'unsafe-eval' 'unsafe-inline' 'self'; img-src 'self' asset: http://asset.localhost blob: data:"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": [],
    "externalBin": [],
    "copyright": "Copyright (c) 2021 Ben Schattinger. Licensed under GPL-3.0",
    "category": "Utility",
    "shortDescription": "Free, cross-platform, CX-II compatible computer linking program for the TI-Nspire",
    "longDescription": "Free, cross-platform, CX-II compatible computer linking program for the TI-Nspire"
  }
}
```

Three things carried forward deliberately:

- **`targets` is `["deb", "appimage"]`** — msi and dmg dropped per the spec's Linux-only scope.
- **The CSP is preserved.** Upstream commit `0472908` fixed a blank release-mode screen caused by CSP; v2 moves this key to `app.security.csp`. The malformed `img-src:` from the v1 config is corrected to a proper `img-src` directive here.
- **`devUrl` is port 1420** — the Tauri scaffold's Vite default. If `vite.config.ts` uses a different port, make these match or dev mode shows a blank window.

- [ ] **Step 2: Write the capability file**

Tauri 2 replaces the v1 `allowlist` with capabilities. Create `desktop/src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Core, dialog, and shell permissions for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:allow-open",
    "shell:allow-open"
  ]
}
```

The v1 allowlist granted exactly `shell.open` and `dialog.open`; this is the v2 equivalent. `notification-all` appeared in the old Cargo features but no frontend code uses it, so it is not carried over.

- [ ] **Step 3: Write `main.rs`**

Replace `desktop/src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::time::Duration;

use libnspire::{PID_CX2, VID};
use rusb::{GlobalContext, Hotplug, UsbContext};
use tauri::{Emitter, Manager, Runtime, WebviewWindow};

use crate::device::{add_device, AddDevice, DevId, DEVICES};

mod commands;
mod device;
mod error;

struct DeviceMon<R: Runtime> {
  window: WebviewWindow<R>,
}

impl<R: Runtime> Hotplug<GlobalContext> for DeviceMon<R> {
  fn device_arrived(&mut self, device: rusb::Device<GlobalContext>) {
    let handle = self.window.clone();
    let is_cx_ii = device
      .device_descriptor()
      .map(|d| d.product_id() == PID_CX2)
      .unwrap_or(false);
    let device = Arc::new(device);
    // The calculator often reports Busy immediately after enumeration, so
    // retry until it settles.
    std::thread::spawn(move || loop {
      match add_device(device.clone()) {
        Ok(dev) => {
          let name = (dev.1).name.clone();
          let needs_drivers = (dev.1).needs_drivers;
          DEVICES.write().unwrap().insert(dev.0, dev.1);
          if let Err(msg) = handle.emit(
            "addDevice",
            AddDevice {
              dev: DevId {
                bus_number: (dev.0).0,
                address: (dev.0).1,
              },
              name,
              is_cx_ii,
              needs_drivers,
            },
          ) {
            eprintln!("{}", msg);
          };
          return;
        }
        Err(rusb::Error::Busy) => {
          println!("busy");
        }
        Err(e) => {
          eprintln!("{}", e);
          return;
        }
      }
      std::thread::sleep(Duration::from_millis(250));
    });
  }

  fn device_left(&mut self, device: rusb::Device<GlobalContext>) {
    if let Some((dev, _)) = DEVICES
      .write()
      .unwrap()
      .remove_entry(&(device.bus_number(), device.address()))
    {
      if let Err(msg) = self.window.emit(
        "removeDevice",
        DevId {
          bus_number: dev.0,
          address: dev.1,
        },
      ) {
        eprintln!("{}", msg);
      };
    }
  }
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_shell::init())
    .setup(|app| {
      // Tauri 1 registered this in on_page_load, guarded by an AtomicBool to
      // avoid double-registering on reload. In v2, setup runs exactly once,
      // so the guard is unnecessary.
      let window = app
        .get_webview_window("main")
        .expect("main window not found");
      if rusb::has_hotplug() {
        if let Err(msg) =
          GlobalContext::default().register_callback(Some(VID), None, None, Box::new(DeviceMon { window }))
        {
          eprintln!("{}", msg);
        };
        std::thread::spawn(|| loop {
          GlobalContext::default().handle_events(None).unwrap();
        });
      } else {
        println!("no hotplug");
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::enumerate,
      commands::open_device,
      commands::close_device,
      commands::update_device,
      commands::list_dir,
      commands::download_file,
      commands::upload_file,
      commands::upload_os,
      commands::delete_file,
      commands::delete_dir,
      commands::create_nspire_dir,
      commands::move_file,
      commands::copy,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
```

The `AtomicBool` guard from the original is deliberately gone: it existed because `on_page_load` fires on every navigation, whereas `setup` runs once.

- [ ] **Step 4: Build and launch**

```bash
cd /home/jjureta/Projects/n-link/desktop
npm run tauri dev
```

Expected: window opens; terminal shows no `no hotplug` warning on a normal Linux system.

- [ ] **Step 5: Verify hotplug end to end**

With the app running, unplug the CX II. The terminal should stay quiet (events go to the frontend, which does not yet listen). Replug it; expect `busy` lines to appear briefly and then stop, indicating `add_device` retried and succeeded.

**This confirms the hotplug thread and `DEVICES` registry work** even before the frontend exists.

- [ ] **Step 6: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/src-tauri/src/main.rs desktop/src-tauri/tauri.conf.json desktop/src-tauri/capabilities/
git commit -m "feat(desktop): wire Tauri 2 builder, hotplug, and capabilities

Hotplug registration moves from on_page_load to setup (runs once, so the
AtomicBool guard is dropped). v1 allowlist replaced by a capability file
granting dialog:allow-open and shell:allow-open. CSP from 0472908 carried
forward to app.security.csp; bundle targets narrowed to deb + appimage."
```

---

### Task 6: Restore headless CLI mode

**Files:**
- Create: `desktop/src-tauri/src/cli.rs`
- Modify: `desktop/src-tauri/src/main.rs`

**Interfaces:**
- Consumes: `device::*`.
- Produces: `cli::run() -> bool` — returns `true` when CLI arguments were handled and the GUI should not launch.

- [ ] **Step 1: Copy the already-migrated CLI**

The old `desktop/src-tauri/src/cli.rs` used `clap = "3.0.0-beta.2"`. `nlink-cli/src/cli.rs` is **already the clap 4 port of this same file** — reuse it rather than redoing the migration:

```bash
cd /home/jjureta/Projects/n-link
cp nlink-cli/src/cli.rs desktop/src-tauri/src/cli.rs
```

- [ ] **Step 2: Confirm it compiles unmodified**

`cli.rs` is **fully self-contained** — verified: it contains no `crate::` or `super::` references and carries its own `get_dev()` built directly on `rusb`. No import reconciliation is needed. Its only external needs are `clap` 4 and `indicatif` 0.15, both already in `Cargo.toml` from Task 3.

Run: `cd /home/jjureta/Projects/n-link/desktop/src-tauri && cargo check 2>&1 | head -40`
Expected: compiles clean (unused-function warnings until Step 3 wires it in).

Leave `cli.rs`'s private `get_dev()` alone even though `device.rs` has similar logic. They are deliberately separate: `get_dev()` opens a handle directly for one-shot headless use, while `device.rs::add_device` populates the global `DEVICES` registry that only the GUI's hotplug monitor and IPC commands use. Merging them would couple the headless path to GUI state.

Public surface produced: `cli::run() -> bool` and `cli::cwd() -> PathBuf`. `run()` returns `true` when a subcommand was parsed and handled, `false` when no subcommand was given.

- [ ] **Step 3: Call the CLI before the GUI**

In `desktop/src-tauri/src/main.rs`, add `mod cli;` with the other module declarations, and make `main` start with the early return:

```rust
fn main() {
  if cli::run() {
    return;
  }
  tauri::Builder::default()
    // ... unchanged
```

- [ ] **Step 4: Verify both modes still work**

```bash
cd /home/jjureta/Projects/n-link/desktop/src-tauri
cargo build --release
./target/release/n-link --help
```
Expected: CLI help text, no window.

```bash
./target/release/n-link ls /
```
Expected: a listing matching `/tmp/oracle-root-listing.txt` from Task 1.

Then confirm the GUI path is unaffected: `cd .. && npm run tauri dev` opens a window.

- [ ] **Step 5: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/src-tauri/src/cli.rs desktop/src-tauri/src/main.rs
git commit -m "feat(desktop): restore headless CLI mode on clap 4

Reuses nlink-cli's already-migrated cli.rs rather than repeating the
clap 3-beta -> 4 migration."
```

---

### Task 7: Verify the backend against the calculator

**Files:**
- Create: `desktop/src/App.vue` (temporary verification harness, replaced by Plan B)

**Interfaces:**
- Consumes: the full backend from Tasks 3–6.
- Produces: evidence that all IPC commands work from the webview. No lasting interface — Plan B replaces this file.

- [ ] **Step 1: Install the frontend plugin packages**

```bash
cd /home/jjureta/Projects/n-link
npm install --workspace desktop @tauri-apps/api@2 @tauri-apps/plugin-dialog@2 @tauri-apps/plugin-shell@2
```

- [ ] **Step 2: Write a throwaway harness**

Replace `desktop/src/App.vue`:

```vue
<template>
  <main style="font-family: monospace; padding: 1rem">
    <button @click="run">Run backend check</button>
    <pre>{{ log }}</pre>
  </main>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const log = ref('');
const add = (line: string) => (log.value += line + '\n');

listen('addDevice', (e) => add('event addDevice: ' + JSON.stringify(e.payload)));
listen('removeDevice', (e) => add('event removeDevice: ' + JSON.stringify(e.payload)));
listen('progress', (e) => add('event progress: ' + JSON.stringify(e.payload)));

async function run() {
  log.value = '';
  try {
    const devices = (await invoke('enumerate')) as any[];
    add('enumerate -> ' + JSON.stringify(devices, null, 2));
    if (!devices.length) return add('NO DEVICES FOUND');

    const { busNumber, address } = devices[0];
    add(`opening ${busNumber}:${address}`);

    const info = await invoke('open_device', { busNumber, address });
    add('open_device -> ' + JSON.stringify(info, null, 2));

    const files = await invoke('list_dir', { busNumber, address, path: '/' });
    add('list_dir / -> ' + JSON.stringify(files, null, 2));

    await invoke('close_device', { busNumber, address });
    add('close_device -> ok');
  } catch (e) {
    add('ERROR: ' + String(e));
  }
}
</script>
```

Note the Tauri 2 import path: `@tauri-apps/api/core`, not the v1 `@tauri-apps/api/tauri`.

- [ ] **Step 3: GATE — enumerate and list against the real device**

```bash
cd /home/jjureta/Projects/n-link/desktop
npm run tauri dev
```

Click "Run backend check". Expected in the page:

- `enumerate` returns one entry with `busNumber`, `address`, `name`, `isCxIi: true`, `needsDrivers: false`
- `open_device` returns an info object with `name`, `free_storage`, `version`, `battery`
- `list_dir /` returns an array of `{path, isDir, date, size}`

**Oracle cross-check:** the file and folder names from `list_dir` must match `/tmp/oracle-root-listing.txt`. If they differ, the port has a defect — investigate before proceeding.

The camelCase keys (`busNumber`, `isDir`) confirm serde serialization is intact, which is what Plan B's frontend depends on.

- [ ] **Step 4: GATE — verify a real file transfer**

Pick a small `.tns` from the listing. In the harness, temporarily append to `run()`:

```ts
    await invoke('open_device', { busNumber, address });
    await invoke('download_file', {
      busNumber, address,
      path: ['/YOURFILE.tns', 1234],
      dest: '/tmp/nlink-gui-test',
    });
    add('download_file -> ok');
```

Substitute the real filename and its exact `size` from the `list_dir` output. Create the destination first: `mkdir -p /tmp/nlink-gui-test`.

Then compare against the oracle:

```bash
mkdir -p /tmp/nlink-cli-test
/home/jjureta/Projects/n-link/nlink-cli/target/release/n-link-cli download "/YOURFILE.tns" /tmp/nlink-cli-test/
sha256sum /tmp/nlink-gui-test/YOURFILE.tns /tmp/nlink-cli-test/YOURFILE.tns
```

Expected: identical hashes.

**Use a small file.** Per the spec, libnspire's frozen CX II support makes multi-packet transfers unreliable against 2026 firmware — a `Busy` failure on a large file is the known upstream ceiling, not a port regression. Confirm any failure reproduces with `n-link-cli` before treating it as a bug in this work.

- [ ] **Step 5: Commit the harness**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/src/App.vue desktop/package.json package-lock.json
git commit -m "test(desktop): add temporary backend verification harness

Verified against a connected CX II: enumerate, open_device, list_dir, and
download_file all work through Tauri 2 IPC, with list_dir output and file
checksum matching n-link-cli. Replaced by the real UI in Plan B."
```

---

### Task 8: Produce Linux bundles

**Files:**
- Modify: `desktop/src-tauri/icons/` (regenerate from `app-icon.png`)

**Interfaces:**
- Consumes: a verified backend from Task 7.
- Produces: `.deb` and `.AppImage` artifacts. Terminal deliverable of Plan A.

- [ ] **Step 1: Regenerate icons from the project artwork**

The scaffold ships placeholder icons; replace them with n-link's own:

```bash
cd /home/jjureta/Projects/n-link/desktop
npm run tauri icon app-icon.png
```

Expected: `src-tauri/icons/` repopulated.

- [ ] **Step 2: Build the release bundles**

```bash
cd /home/jjureta/Projects/n-link/desktop
npm run tauri build
```

Expected: completes and reports paths under `src-tauri/target/release/bundle/`.

- [ ] **Step 3: Confirm the artifacts exist**

```bash
find /home/jjureta/Projects/n-link/desktop/src-tauri/target/release/bundle -type f \( -name '*.deb' -o -name '*.AppImage' \)
```

Expected: one `.deb` and one `.AppImage`.

- [ ] **Step 4: GATE — verify the release build actually renders**

```bash
cd /home/jjureta/Projects/n-link/desktop
./src-tauri/target/release/n-link
```

Expected: the window opens **and shows the harness UI**.

**A blank window here means the CSP regressed** — this is exactly the failure upstream commit `0472908` fixed. If blank, revisit `app.security.csp` in `tauri.conf.json` before continuing.

- [ ] **Step 5: Verify the installable package**

```bash
sudo dpkg -i /home/jjureta/Projects/n-link/desktop/src-tauri/target/release/bundle/deb/*.deb
n-link --help
```

Expected: installs cleanly, CLI help prints. Confirms the deb has no unsatisfied dependency on a webkit2gtk-4.0-era library — the original failure this whole port exists to fix.

- [ ] **Step 6: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/src-tauri/icons/
git commit -m "build(desktop): regenerate icons and verify deb/appimage bundles

Release build renders (no CSP regression) and the deb installs cleanly on
Ubuntu 26.04 — the failure mode that motivated this port."
```

---

## Definition of Done (Plan A)

- [ ] A Tauri 2 window opens on Ubuntu 26.04 with webkit2gtk-4.1
- [ ] All 13 commands are registered and callable from the webview
- [ ] `enumerate` detects the CX II with correct camelCase payload keys
- [ ] `list_dir /` output matches `n-link-cli ls /`
- [ ] A downloaded file's sha256 matches the same file fetched by `n-link-cli`
- [ ] Hotplug add/remove events fire
- [ ] Headless CLI mode still works
- [ ] `.deb` and `.AppImage` build; the release binary renders without a blank screen
- [ ] `web/` is untouched
- [ ] All work is on `port/tauri2-vue3`

## Follow-on

Plan B (frontend) covers `n-link-core` → Vue 3, `desktop/src` → Vue 3, and replaces the Task 7 harness. It is written **after** Plan A is verified, so it can be grounded in the real scaffold output rather than assumptions. Known hazards already identified for it:

- `Vue.prototype.$devices = devices` (`devices.ts:291`) and `new Devices()` are Vue 2 patterns with no Vue 3 equivalent — the store becomes `reactive()` plus `app.config.globalProperties`.
- `@PropSync('selected')` (`DeviceSelect.vue:52`) has no Vue 3 counterpart — becomes `defineModel()`.
- `<div slot="reference">` is Vue 2 slot syntax — becomes `<template #reference>`.
- `src="~feather-icons/..."` uses webpack `~` resolution, which Vite does not support.
- Tailwind 1.8 + `@tailwindcss/custom-forms` → Tailwind 4.3.3, which is CSS-first and drops `tailwind.config.js`.
- The `declare module 'vue/types/vue'` augmentation (`n-link-core/components/devices.ts:62`) is Vue 2 — becomes `ComponentCustomProperties` on `@vue/runtime-core`.
- ~10 `$set`/`$delete` calls in `devices.ts` are deleted outright; Vue 3 proxy reactivity makes them unnecessary. This is the likeliest source of silent regressions.
