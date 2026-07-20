# N-Link

Free, cross-platform, CX-II compatible computer linking program for the TI-Nspire.

This package is a **Tauri 2 + Vue 3 + Vite** application. It was ported from
Tauri 1.0.0-beta.8 / Vue CLI, which linked `webkit2gtk-4.0` and could not build
or run on current distributions (Ubuntu 26.04 ships only `webkit2gtk-4.1`).

Currently Linux-only: bundle targets are `deb` and `appimage`.

## Prerequisites

- [Node.js](https://nodejs.org/) 22+ (npm ships with it — this project uses
  **npm workspaces**, not yarn)
- [Rust](https://rustup.rs/) 1.97+
- System libraries (Ubuntu/Debian):

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libjavascriptcoregtk-4.1-dev \
  libsoup-3.0-dev \
  librsvg2-dev \
  libxdo-dev \
  libayatana-appindicator3-dev \
  libusb-1.0-0-dev \
  build-essential curl wget file
```

Verify they resolve before building:

```bash
for p in webkit2gtk-4.1 javascriptcoregtk-4.1 libsoup-3.0 gtk+-3.0 libusb-1.0; do
  printf "%-24s " "$p"; pkg-config --modversion "$p" || echo MISSING
done
```

## Setup

Run from the **repository root** (this is a workspace member):

```bash
npm install --workspace desktop --legacy-peer-deps
```

`--legacy-peer-deps` is currently required: the sibling `n-link-core` package is
still on Vue 2 and its peer ranges conflict. It can be dropped once `n-link-core`
is ported to Vue 3.

## Development

```bash
cd desktop
npm run tauri dev
```

## Production build

```bash
cd desktop
npm run tauri build
```

Produces `src-tauri/target/release/bundle/deb/*.deb` and
`.../appimage/*.AppImage`.

## Headless CLI

The same binary works without a GUI:

```bash
./src-tauri/target/release/n-link --help
./src-tauri/target/release/n-link ls /
```

## Notes and gotchas

- **`cargo` commands need `dist/` to exist.** Tauri's `generate_context!` macro
  validates the `frontendDist` path at compile time, so a bare
  `cargo check` / `cargo build` in `src-tauri/` fails on a fresh clone until the
  frontend has been built once: `npm run build --workspace desktop`.
  `npm run tauri build` does this for you.
- **Do not run the binary from a snap-confined shell** (for example, the
  integrated terminal of VS Code installed as a snap). Snap injects an
  `LD_LIBRARY_PATH` whose `libpthread` shadows the system one, producing
  `undefined symbol: __libc_pthread_init`. Use a normal terminal.
- **AppImage needs FUSE.** Without it, run the AppImage with
  `--appimage-extract` and launch `squashfs-root/AppRun` instead.
- **`vendor/libnspire-sys`** at the repo root is a locally patched crate wired in
  via `[patch.crates-io]`. It raises the CX II NNSE handshake retry limit
  (10 → 30). The build will not reproduce upstream behaviour without it.
- **Known upstream limitation:** libnspire's CX II support has been frozen since
  2020. Against current CX II firmware, data transfers may return `Busy` or hang.
  This is independent of the Tauri version and is not a defect in this port.
