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

## The `Busy` error on CX II (fixed)
Long operations against current CX II firmware — listing a folder with many entries,
downloading a file of any size — used to fail with `Busy`. Two bugs in the vendored
libnspire, both fixed in `vendor/libnspire-sys/libnspire/src/cx2.cpp`:

1. **`readPacket` checksummed the wrong number of bytes.** It used the byte count the
   bulk transfer returned instead of the length the packet declares. The calculator can
   hand back more bytes than the packet occupies (a 64-byte packet arriving as 65), and
   the stray byte failed the checksum. The packet was then dropped and never acked, so
   the calculator retransmitted it forever until the receive loop gave up. The same
   variable was also wrong after a multi-chunk read, where it held only the last chunk's
   size — this is what made large stream transfers unreliable.
2. **No retransmission.** A lost packet or lost ack was fatal; the packet was sent once
   and never repeated. It now retransmits with the "not the first try" bit (`reqAck` bit
   3) that TI's own stack uses, keeping the same sequence number.

Note that the error was never really `Busy`: the `libnspire` **crate** (0.2.3, not this
vendored C library) matches its positive `NSPIRE_ERR_*` constants against libnspire's
negative return codes, so every error falls through to the libusb table and is
mislabelled. `-NSPIRE_ERR_INVALPKT` is `-6`, and so is `LIBUSB_ERROR_BUSY`. A reported
`Busy` is really `Invalid packet received`; `Timeout` is really `NACK received`, and so
on. That mismapping is upstream in the crate and is **not** fixed here.
