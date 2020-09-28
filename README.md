# N-Link

Free, cross-platform, CX-II compatible computer linking program for the TI-Nspire

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
