# nlink-cli — standalone TI-Nspire CX II link CLI (Linux)

A GUI-free command-line build extracted from [n-link](https://github.com/lights0123/n-link),
for transferring files to/from a TI-Nspire CX II (CAS) over USB on Linux.

Built because:
- **TiLP2** (distro `libticables` 1.3.6) does not support the CX II (USB PID `e022`) — its
  probe reports "no devices found".
- The **prebuilt n-link 0.1.6 `.deb`** won't run on Ubuntu 26.04 (needs the retired
  `libssl.so.1.1` + WebKit2GTK 4.0). n-link is a Tauri/WebKit app.

This CLI reuses only n-link's self-contained `cli.rs` (upload/download/ls/mkdir/rmdir/
copy/move/upload-os), dropping Tauri/WebKit entirely.

## Build
```bash
# one-time system deps
sudo apt-get install -y libusb-1.0-0-dev pkg-config
# Rust toolchain via rustup, then:
cargo build --release
# binary: target/release/n-link-cli
```
`Cargo.toml` uses a local `[patch.crates-io]` override of `libnspire-sys`
(`../vendor/libnspire-sys`) whose CX II NNSE handshake retry limit is raised from 10.

## Run
```bash
./target/release/n-link-cli ls /
./target/release/n-link-cli download "/MyFile.tns" ./
./target/release/n-link-cli upload ./MyFile.tns "/"
./target/release/n-link-cli --help
```

## System setup required (Linux)
1. **udev access** for the CX II (`/etc/udev/rules.d/70-ti-nspire.rules`):
   ```
   SUBSYSTEM=="usb", ATTRS{idVendor}=="0451", ENV{ID_MM_DEVICE_IGNORE}="1"
   ```
   (plus group/`uaccess` for PID `e022`) — then `udevadm control --reload-rules && udevadm trigger`.
2. **ModemManager** grabs the interface on plug; the `ID_MM_DEVICE_IGNORE` rule above
   tells it to leave TI devices alone. Restart it after adding the rule:
   `sudo systemctl start ModemManager`.

## Known limitation
libnspire's CX II ("NavNet SE") protocol support has been **frozen upstream since 2020**
(`lights0123/libnspire`, last commit 2020-09-30; crates.io `libnspire-sys` 0.3.4 is the newest).
Against **current (2026) CX II firmware** the USB link and handshake work — real filenames
are transferred — but **multi-packet stream transfers are unreliable** and long operations
(e.g. listing a folder with many entries) can fail mid-transfer with `Busy`. Short transfers
are more likely to complete. This is a library/firmware-era mismatch, not a local
configuration problem.
