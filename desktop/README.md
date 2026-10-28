# N-Link

Free, cross-platform, CX-II compatible computer linking program for the TI-Nspire

## Native Apple Silicon build

From the repository root, run:

```sh
rustup toolchain install 1.77.2 --profile minimal
./scripts/build-apple-silicon.sh
```

Requires Apple's Command Line Tools, Node.js/npm, Python 3, and Rust installed
through rustup. Node.js 22 is used in CI. The script selects Rust 1.77.2 for
this build to support the project's 2021 dependencies, and uses the existing Yarn and Cargo
lockfiles, skips the old Tauri CLI's Intel-only installer, builds the frontend,
and compiles the backend for `aarch64-apple-darwin`. `libusb` is linked statically;
the resulting app needs only macOS system libraries.

Outputs are `releases/N-Link.app`, `releases/N-Link_0.1.6_macOS_arm64.zip`, and
the ZIP's SHA-256 checksum. The app is ad-hoc signed for local use; it is not
Developer ID signed or notarized. Install it by copying `N-Link.app` to
`~/Applications` and opening it.

The deployment target is macOS 11.0; older macOS versions still need runtime
testing. `.github/workflows/apple-silicon.yml` builds on a native ARM64 runner,
checks the architecture and code signature, and uploads the release ZIP and
checksum as an Actions artifact. USB behavior needs testing on a physical Mac
with a connected calculator.

## Project setup
You'll need to install [Node.js](https://nodejs.org/en/download/package-manager),
[Yarn](https://classic.yarnpkg.com/en/docs/install/) (which may be included with your installation of Node.js), and
[Rust](https://rustup.rs/). You'll also need a
[C compiler](https://github.com/alexcrichton/cc-rs#compile-time-requirements) available. On Linux, you'll need
`webkit2gtk` and `squashfs-tools`, which, on Ubuntu, can be installed with
`sudo apt install webkit2gtk-4.0 squashfs-tools`.
```
yarn
```

### Compiles and hot-reloads for development
```
yarn tauri:serve
```

### Compiles and minifies for production
```
yarn tauri:build
```

### Lints and fixes files
```
yarn lint
```

### Customize configuration
See [Configuration Reference](https://cli.vuejs.org/config/).
