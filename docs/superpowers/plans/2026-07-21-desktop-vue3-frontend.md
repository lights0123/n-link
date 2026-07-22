# n-link Desktop — Vue 3 Frontend Port (Plan B)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Plan A's throwaway verification harness with the real n-link UI, ported from Vue 2 class components to Vue 3 `<script setup>`, running on Vite.

**Architecture:** `n-link-core`'s 8 components and `desktop/src`'s store and views are rewritten as Vue 3 Composition API. The IPC contract stays frozen, so the Rust backend from Plan A is untouched except for one deliberate fix (making blocking USB commands async). `n-link-core` remains a source-only package consumed through a Vite alias — it has no build step of its own.

**Tech Stack:** Vue 3.5.40, Vite 8.1.5, vue-router 4.6.4, element-plus 2.14.3, Tailwind 3.4.19, @tailwindcss/forms 0.5.11, sass 1.101.3, TypeScript, Tauri 2.11.5.

**Source of truth:** `docs/superpowers/specs/2026-07-19-desktop-tauri2-vue3-port-design.md`
**Predecessor:** `docs/superpowers/plans/2026-07-19-desktop-tauri2-backend.md` (complete, verified)

## Global Constraints

- Branch: `port/tauri2-vue3`. Do not commit to `main`. Remote `origin` is the fork `jjureta/n-link`; `upstream` is `lights0123/n-link`.
- **The IPC contract is frozen.** 13 commands: `enumerate`, `open_device`, `close_device`, `update_device`, `list_dir`, `download_file`, `upload_file`, `upload_os`, `delete_file`, `delete_dir`, `create_nspire_dir`, `move_file`, `copy`. 3 events: `addDevice`, `removeDevice`, `progress`. Payloads are camelCase. Do not rename anything on either side.
- Tauri 2 import paths: `@tauri-apps/api/core` for `invoke` (NOT the v1 `@tauri-apps/api/tauri`), `@tauri-apps/api/event` for `listen`, `@tauri-apps/plugin-dialog` for `open`, `@tauri-apps/plugin-shell` for `open`.
- **Tailwind 3**, not 4. Decision recorded 2026-07-21: the existing theme is a JS config with 16 `@apply`/`theme()` uses inside scoped SCSS; Tailwind 4's CSS-first rewrite would layer a second migration on top of the Vue one.
- `web/` is out of scope and knowingly broken. Do not modify any file under `web/`.
- `nlink-cli/` and `vendor/` are supporting trees. Do not modify them.
- Do not commit `target/`, `node_modules/`, or `dist/`.
- No test framework. Verification is compilation plus manual checks against the connected CX II, per the spec.

**Hardware reality (from Plan A, verified 2026-07-21):**
- `enumerate` works — reads USB descriptors only, no libnspire transfer.
- `open_device` works — returns full device info.
- `list_dir` **hangs** against current CX II firmware. This is a known upstream libnspire limitation the user owns. It is NOT a defect in this work, and it means the file browser cannot be end-to-end verified. Build it correctly; verify what can be verified.

---

### Task 1: Make blocking USB commands async

**Files:**
- Modify: `desktop/src-tauri/src/commands.rs`

**Interfaces:**
- Consumes: the Plan A backend.
- Produces: the same 13 commands, with the blocking ones no longer able to wedge the command queue. No signature change visible to the frontend.

**Why this is first:** Plan A's verification found that `close_device` — which does no I/O at all — timed out because a hung `list_dir` blocked it. All 13 commands are synchronous, and Tauri runs sync commands on the main thread, so one stuck USB operation freezes every subsequent command. Since `list_dir` reliably hangs on this hardware, the UI is untestable until this is fixed.

- [ ] **Step 1: Confirm the current state**

Run: `cd /home/jjureta/Projects/n-link/desktop/src-tauri && grep -c "async fn" src/commands.rs`
Expected: `0` — all commands are currently synchronous.

- [ ] **Step 2: Move blocking commands onto the blocking pool**

The commands that perform libnspire I/O must not run on the main thread. Wrap each body in `tauri::async_runtime::spawn_blocking`. Apply to exactly these 11: `open_device`, `update_device`, `list_dir`, `download_file`, `upload_file`, `upload_os`, `delete_file`, `delete_dir`, `create_nspire_dir`, `move_file`, `copy`.

Leave `enumerate` and `close_device` synchronous — neither performs a libnspire transfer (`enumerate` reads USB descriptors; `close_device` only takes a write lock and sets an enum).

The pattern, shown for `list_dir`:

```rust
#[tauri::command]
pub async fn list_dir<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  window: WebviewWindow<R>,
) -> Result<Vec<FileInfo>, SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
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
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}
```

Three things change per command and they are easy to get wrong:

1. `pub fn` becomes `pub async fn`.
2. **The return type must become concrete.** `impl Serialize` cannot cross an async boundary here. Replace `Result<impl Serialize, SerializedError>` with the actual type: `Vec<FileInfo>` for `list_dir`, `libnspire::info::Info` for `open_device` and `update_device`, and `()` for all the mutating commands (`download_file`, `upload_file`, `upload_os`, `delete_file`, `delete_dir`, `create_nspire_dir`, `move_file`, `copy`).
3. The closure must own its arguments — `move ||`, and `window` moves in.

- [ ] **Step 3: Verify it compiles**

Run: `cd /home/jjureta/Projects/n-link/desktop/src-tauri && cargo check`
Expected: compiles clean.

If you hit a lifetime error on `progress_sender(&window, ...)`, that is expected: it borrows `window`. Because `window` is moved into the closure, the borrow is now inside the closure and valid. If the compiler still objects, the closure is not taking ownership — check for a missing `move`.

- [ ] **Step 4: Confirm the frontend contract did not change**

Run: `grep -oE "pub async fn [a-z_]+|pub fn [a-z_]+" src/commands.rs | sed 's/pub async fn //;s/pub fn //' | sort | tr '\n' ' '`
Expected exactly these 13 names: `close_device copy create_nspire_dir delete_dir delete_file download_file enumerate list_dir move_file open_device update_device upload_file upload_os`

Command names and payloads are unchanged, so `invoke('list_dir', ...)` on the frontend behaves identically — it just no longer blocks its siblings.

- [ ] **Step 5: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/src-tauri/src/commands.rs
git commit -m "fix(desktop): run blocking USB commands on the blocking pool

Plan A verification found close_device timing out despite doing no I/O:
all commands were synchronous, Tauri runs sync commands on the main
thread, so a hung list_dir blocked every subsequent command.

The 11 libnspire-touching commands now spawn_blocking. enumerate and
close_device stay sync — neither performs a transfer. Return types become
concrete because impl Serialize cannot cross the async boundary. No
change to command names or payloads."
```

---

### Task 2: Frontend foundation — Tailwind 3, Vite aliases, shared types

**Files:**
- Modify: `desktop/package.json`, `desktop/vite.config.ts`
- Create: `desktop/tailwind.config.js`, `desktop/postcss.config.js`
- Modify: `n-link-core/package.json`, `n-link-core/tailwind.config.js`
- Create: `n-link-core/components/devices.ts` (replace the Vue 2 module augmentation)
- Delete: `n-link-core/shims-vue.d.ts`, `n-link-core/shims-tsx.d.ts`

**Interfaces:**
- Consumes: nothing from Task 1.
- Produces:
  - A Vite alias `n-link-core` → `../n-link-core`, so components import as `n-link-core/components/Foo.vue`.
  - Working Tailwind 3 with the original theme.
  - `n-link-core/components/devices.ts` exporting the shared types (`DevId`, `Version`, `Lcd`, `HardwareType`, `RunLevel`, `Battery`, `Info`, `FileInfo`, `Progress`, `PartialCmd`, `Cmd`, `Device`, `GenericDevices`) with the Vue 2 `declare module 'vue/types/vue'` block replaced by a Vue 3 `ComponentCustomProperties` augmentation.

- [ ] **Step 1: Install frontend dependencies**

```bash
cd /home/jjureta/Projects/n-link
npm install --workspace desktop --legacy-peer-deps \
  vue-router@4.6.4 element-plus@2.14.3 filesize@10 feather-icons@4.29.2
npm install --workspace desktop --legacy-peer-deps --save-dev \
  tailwindcss@3.4.19 @tailwindcss/forms@0.5.11 postcss autoprefixer sass@1.101.3
```

`--legacy-peer-deps` is required while `n-link-core` still declares Vue 2 peers; Step 6 fixes that.

- [ ] **Step 2: Configure Vite**

Replace `desktop/vite.config.ts`. Keep the existing Tauri-specific settings (fixed port 1420, `strictPort`, the HMR block, ignoring `src-tauri`) — those come from the scaffold and Tauri depends on them. Add the alias and SCSS support:

```ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath, URL } from "node:url";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      "n-link-core": fileURLToPath(new URL("../n-link-core", import.meta.url)),
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  // Vite ignores watching src-tauri; Tauri expects a fixed port.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
```

Do not change the port — `tauri.conf.json`'s `devUrl` is `http://localhost:1420` and they must agree.

- [ ] **Step 3: Add Tailwind 3 config to desktop**

Create `desktop/tailwind.config.js`:

```js
module.exports = {
  content: [
    './index.html',
    './src/**/*.{vue,js,ts}',
    '../n-link-core/components/**/*.{vue,js,ts}',
  ],
  presets: [require('../n-link-core/tailwind.config.js')],
};
```

Create `desktop/postcss.config.js`:

```js
module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

- [ ] **Step 4: Migrate the core Tailwind config from 1.x to 3.x**

Replace `n-link-core/tailwind.config.js`. The theme is preserved exactly; only the 1.x-era plumbing changes:

```js
module.exports = {
  theme: {
    fontFamily: {
      sans: [
        'Cantarell',
        'Roboto',
        'system-ui',
        '-apple-system',
        'BlinkMacSystemFont',
        '"Segoe UI"',
        '"Helvetica Neue"',
        'Arial',
        '"Noto Sans"',
        'sans-serif',
        '"Apple Color Emoji"',
        '"Segoe UI Emoji"',
        '"Segoe UI Symbol"',
        '"Noto Color Emoji"',
      ],
    },
    extend: {
      inset: {
        '1/8': '0.125em',
      },
      boxShadow: {
        even: '0 2px 12px 0 #0000001a',
        error: '0 0 0 3px #F56C6CA0',
      },
      padding: {
        '2.5': '0.625em',
        '3/2': '0.375em',
      },
      colors: {
        ui: {
          background: 'var(--color-ui-background)',
          emph: 'var(--color-ui-emph)',
          sidebar: 'var(--color-ui-sidebar)',
          text: 'var(--color-ui-text)',
          'text-inv': 'var(--color-ui-text-inv)',
          primary: 'var(--color-ui-primary)',
          border: 'var(--color-ui-border)',
          blockquote: 'var(--color-ui-blockquote)',
        },
      },
      height: {
        min: 'min-content',
      },
    },
  },
  plugins: [require('@tailwindcss/forms')],
};
```

Four deliberate changes:
- `future: {...}` removed — those flags became default behaviour in Tailwind 2.
- `variants: {}` removed — Tailwind 3 generates all variants on demand.
- `purge: {...}` removed — replaced by `content` in the consuming config (Step 3). The old block was `web`-specific and is dead here.
- `@tailwindcss/custom-forms` → `@tailwindcss/forms`. **This changes class names**: `.form-checkbox` becomes a bare `<input type="checkbox">` styled by the plugin's base styles. Task 9 handles the one usage in `Home.vue`.

- [ ] **Step 5: Verify the gray-400 colour used by `theme()` still resolves**

`FileBrowser.vue` and `Home.vue` use `theme('colors.gray.400')` in SCSS. Tailwind 3 renamed the default palette's `gray` to `gray` (unchanged) but the shades differ slightly from 1.x. This is cosmetic and acceptable; do not add a compatibility shim.

- [ ] **Step 6: Update n-link-core's package manifest for Vue 3**

Replace `n-link-core/package.json`:

```json
{
  "name": "n-link-core",
  "version": "0.0.0",
  "peerDependencies": {
    "element-plus": "^2.14.3",
    "vue": "^3.5.40"
  },
  "devDependencies": {
    "tailwindcss": "^3.4.19",
    "@tailwindcss/forms": "^0.5.11"
  }
}
```

`element-ui`, `vue-class-component`, and `vue-property-decorator` are all removed — nothing will import them after this plan.

- [ ] **Step 7: Rewrite the shared types module**

Replace `n-link-core/components/devices.ts`. Keep every type exactly as-is (the backend serializes to these shapes), and replace only the trailing Vue 2 augmentation:

```ts
export type DevId = { address: number; busNumber: number };

export type Version = { major: number; minor: number; patch: number; build: number };
export type Lcd = { width: number; height: number; bpp: number; sample_mode: number };
export type HardwareType =
    | "Cas"
    | "NonCas"
    | "CasCx"
    | "NonCasCx"
    | { Unknown: number };
// The current state of the calculator.
export type RunLevel =
    | "Recovery"
    | "Os"
    | { Unknown: number };
export type Battery =
    | "Powered"
    | "Low"
    | "Ok"
    | { Unknown: number };
export type Info = { free_storage: number; total_storage: number; free_ram: number; total_ram: number; version: Version; boot1_version: Version; boot2_version: Version; hw_type: HardwareType; clock_speed: number; lcd: Lcd; os_extension: string; file_extension: string; name: string; id: string; run_level: RunLevel; battery: Battery; is_charging: boolean };

export type FileInfo = { path: string; isDir: boolean; date: number; size: number };

export type Progress = { remaining: number; total: number };

export type PartialCmd = { action: 'download'; path: [string, number]; dest?: string }
    | { action: 'upload'; path: string } & ({ src: string } | { file: File })
    | { action: 'uploadOs' } & ({ src: string } | { file: File })
    | { action: 'deleteFile'; path: string }
    | { action: 'deleteDir'; path: string }
    | { action: 'createDir'; path: string }
    | { action: 'move'; src: string; dest: string }
    | { action: 'copy'; src: string; dest: string };

export type Cmd = { id: number } & PartialCmd;

export type Device = { name: string; isCxIi: boolean; needsDrivers: boolean; info?: Info; progress?: Progress; queue?: Cmd[]; running?: boolean };

export interface GenericDevices {
    devices: Record<string, Device>;
    enumerating: boolean;
    hasEnumerated: boolean;
    errorHandler?: (e: DOMException) => void;

    enumerate(): Promise<void>;
    open(dev: string): Promise<void>;
    close(dev: string): Promise<void>;
    update(dev: string): Promise<void>;
    listDir(dev: DevId | string, path: string): Promise<FileInfo[]>;
    uploadFiles(dev: DevId | string, path: string, files: File[]): Promise<void>;
    promptUploadFiles(dev: DevId | string, path: string): Promise<void>;
    uploadOs(dev: DevId | string, filter: string): Promise<void>;
    uploadOsFile(dev: DevId | string, file: File): Promise<void>;
    downloadFiles(dev: DevId | string, files: [string, number][]): Promise<void>;
    delete(dev: DevId | string, files: FileInfo[]): Promise<void>;
    createDir(dev: DevId | string, path: string): Promise<void>;
    copy(dev: DevId | string, src: string, dest: string): Promise<void>;
    move(dev: DevId | string, src: string, dest: string): Promise<void>;
}

import type { InjectionKey } from 'vue';

/**
 * n-link-core is consumed by more than one host (the Tauri desktop app and
 * the WebUSB web app), each of which supplies its own GenericDevices
 * implementation. Core components must therefore never import a concrete
 * store — they inject this key. The host provides it at startup.
 */
export const DEVICES_KEY = Symbol('devices') as InjectionKey<GenericDevices>;

declare module 'vue' {
    interface ComponentCustomProperties {
        $devices: GenericDevices;
    }
}

export {};
```

Two separate mechanisms, and both are needed:

- **`ComponentCustomProperties`** replaces the Vue 2 `declare module 'vue/types/vue' { interface Vue { ... } }`. It is what keeps `$devices` type-checked *in templates*, which is where the vast majority of usages are.
- **`DEVICES_KEY`** is for `<script setup>` blocks, which have no `this` and therefore no access to `globalProperties`. A core component whose *script* needs the store injects it:
  ```ts
  import { inject } from 'vue';
  import { DEVICES_KEY, type GenericDevices } from './devices';
  const devices = inject(DEVICES_KEY) as GenericDevices;
  ```

Do **not** make a core component `import devices from '.../desktop/src/components/devices'`. That would hard-wire the shared package to the desktop implementation and permanently break the web consumer.

- [ ] **Step 8: Delete the Vue 2 shims**

```bash
cd /home/jjureta/Projects/n-link
git rm --quiet n-link-core/shims-vue.d.ts n-link-core/shims-tsx.d.ts
```

`shims-vue.d.ts` declared the `vue-async-computed` interfaces, which Task 7 removes entirely. `shims-tsx.d.ts` declared the Vue 2 TSX global namespace, unused.

- [ ] **Step 9: Verify the build still runs**

Run: `cd /home/jjureta/Projects/n-link && npm run build --workspace desktop`
Expected: succeeds. The harness `App.vue` from Plan A is still in place and still compiles; components arrive in later tasks.

- [ ] **Step 10: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/package.json desktop/vite.config.ts desktop/tailwind.config.js \
        desktop/postcss.config.js n-link-core/package.json \
        n-link-core/tailwind.config.js n-link-core/components/devices.ts \
        n-link-core/shims-vue.d.ts n-link-core/shims-tsx.d.ts package-lock.json
git commit -m "feat(frontend): Vue 3 foundation — Tailwind 3, Vite alias, shared types

Migrates n-link-core's Tailwind config from 1.x to 3.x (drops future/
variants/purge, swaps custom-forms for @tailwindcss/forms) and replaces
the Vue 2 'vue/types/vue' augmentation with Vue 3
ComponentCustomProperties. Deletes the Vue 2 shims."
```

---

### Task 3: Port FileIcon.vue — asset resolution

**Files:**
- Modify: `n-link-core/components/FileIcon.vue`

**Interfaces:**
- Consumes: nothing.
- Produces: `<file-icon :path="string" :dir="boolean" :width="number" />` — same props as before. Used by `FileView` (Task 4).

**Why this is its own task:** it contains 30 `require()` calls, which Vite cannot execute. This is the single largest mechanical change in the plan and it either works completely or fails completely.

- [ ] **Step 1: Rewrite the component**

Replace `n-link-core/components/FileIcon.vue`. The extension-to-icon mapping is preserved exactly; only the lookup mechanism changes.

```vue
<template>
  <img :width="width" :src="icon" alt=""/>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  path: string;
  dir?: boolean;
  width?: number;
}>(), {
  dir: false,
  width: 64,
});

// Vite replacement for webpack's require(). Eagerly resolves every SVG in
// assets/ to its emitted URL at build time, keyed by module path.
const assets = import.meta.glob<string>('../assets/**/*.svg', {
  eager: true,
  query: '?url',
  import: 'default',
});

function asset(name: string): string {
  const key = `../assets/${name}`;
  const found = assets[key];
  if (!found) throw new Error(`missing icon asset: ${key}`);
  return found;
}

const FOLDER_ICONS: Record<string, string> = {
  games: 'games',
  images: 'images',
  photos: 'images',
  music: 'music',
  sound: 'music',
  sounds: 'music',
  script: 'scripts',
  scripts: 'scripts',
  program: 'scripts',
  programs: 'scripts',
  template: 'templates',
  templates: 'templates',
  video: 'video',
  videos: 'video',
};

const EXT_ICONS: Record<string, string> = {
  bin: 'binary',
  ical: 'calendar', ics: 'calendar', ifb: 'calendar', icalendar: 'calendar',
  cfg: 'cfg',
  vcf: 'contact',
  csv: 'csv', log: 'csv', logs: 'csv',
  sql: 'db', db: 'db', sqlite: 'db',
  epub: 'epub',
  eot: 'font', otf: 'font', ttf: 'font', woff: 'font', woff2: 'font',
  png: 'image', bmp: 'image', jpg: 'image', jpeg: 'image', jpe: 'image',
  jif: 'image', jfif: 'image', jfi: 'image', jp2: 'image', j2k: 'image',
  jpf: 'image', jpx: 'image', jpm: 'image', mj2: 'image', gif: 'image',
  tiff: 'image', tif: 'image', ppm: 'image', pgm: 'image', pbm: 'image',
  pnm: 'image', webp: 'image', heif: 'image', heifs: 'image', heic: 'image',
  heics: 'image', avci: 'image', avcs: 'image', avif: 'image', avifs: 'image',
  ico: 'image', icon: 'image', icns: 'image', pcx: 'image', pgf: 'image',
  tga: 'image', psd: 'image', xcf: 'image',
  js: 'js', mjs: 'js',
  json: 'json', json5: 'json',
  lua: 'lua',
  pdf: 'pdf',
  py: 'python',
  rom: 'rom', '89u': 'rom', smc: 'rom', sfc: 'rom', srm: 'rom', img: 'rom',
  svg: 'svg', svgz: 'svg',
  txt: 'txt',
  mp4: 'video',
  xml: 'xml',
  zip: 'zip', tar: 'zip', gz: 'zip',
};

const name = computed(() => props.path.split('/').pop() as string);

const ext = computed(() => {
  const parts = name.value.split('.');
  if (parts[parts.length - 1] === 'tns') parts.pop();
  if (parts.length < 2) return '';
  return parts.pop() as string;
});

const icon = computed(() => {
  if (props.dir) {
    const key = FOLDER_ICONS[name.value.trim().toLowerCase()] ?? 'folder';
    return asset(`folders/${key}.svg`);
  }
  if (name.value === 'ndless_resources.tns') return asset('files/resources.svg');
  if (name.value.startsWith('ndless_installer')) return asset('files/installer.svg');
  return asset(`files/${EXT_ICONS[ext.value.toLowerCase()] ?? 'unknown'}.svg`);
});
</script>
```

Note the original had `alt="name"` — a literal string, not a binding, so it rendered the word "name" for every icon. These are decorative icons beside a filename label, so `alt=""` is correct. This is a deliberate fix, not an oversight.

- [ ] **Step 2: Verify every mapped asset exists**

The lookup now throws on a missing file rather than failing silently at build time. Confirm the mapping matches what is on disk:

```bash
cd /home/jjureta/Projects/n-link/n-link-core/assets
ls files/ folders/
```

Expected in `folders/`: `folder.svg games.svg images.svg music.svg scripts.svg templates.svg video.svg`
Expected in `files/`: `binary.svg calendar.svg cfg.svg contact.svg csv.svg db.svg epub.svg exec.svg font.svg image.svg installer.svg js.svg json.svg lua.svg pdf.svg python.svg resources.svg rom.svg svg.svg txt.svg unknown.svg video.svg xml.svg zip.svg`

`exec.svg` is present but unreferenced by the original mapping. Leave it unreferenced — do not invent a mapping for it.

- [ ] **Step 3: Verify it compiles**

Run: `cd /home/jjureta/Projects/n-link && npm run build --workspace desktop`
Expected: succeeds. `FileIcon` is not yet imported by anything, so this only proves it parses and type-checks.

- [ ] **Step 4: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add n-link-core/components/FileIcon.vue
git commit -m "feat(core): port FileIcon to Vue 3, replace require() with import.meta.glob

30 webpack require() calls replaced by a single eager import.meta.glob.
The extension-to-icon mapping is preserved exactly, restructured from a
switch chain into lookup tables. Fixes alt=\"name\" rendering the literal
string 'name'."
```

---

### Task 4: Port CalcInfo.vue and FileView.vue

**Files:**
- Modify: `n-link-core/components/CalcInfo.vue`, `n-link-core/components/FileView.vue`

**Interfaces:**
- Consumes: `FileIcon` (Task 3), types from `devices.ts` (Task 2).
- Produces:
  - `<calc-info :info="Info" :dev="string" :native-upload="boolean" />`
  - `<file-view :files="FileInfo[]" :show-hidden="boolean" @nav="path => void" @select="FileInfo[] => void" />`

- [ ] **Step 1: Read the current sources**

Both files are Vue 2 class components. Read them in full before editing:
`n-link-core/components/CalcInfo.vue` and `n-link-core/components/FileView.vue`.

- [ ] **Step 2: Port CalcInfo.vue**

Convert to `<script setup lang="ts">`. The template is unchanged except for the `$refs` usage. Specifics:

- `@Prop({type: Object, required: true}) info!: Info` → `defineProps<{ info: Info; dev: string; nativeUpload?: boolean }>()` with `withDefaults` for `nativeUpload: false`.
- `refreshing = false` → `const refreshing = ref(false)`.
- `formatSize`, `formatVersion`, `refresh`, `uploadNative` become plain functions.
- `$refs.upload` → `const upload = ref<HTMLInputElement | null>(null)` plus `ref="upload"` in the template; call `upload.value?.click()`.
- `import fileSize from "filesize"` — filesize 10 changed its export shape. Use `import { filesize } from 'filesize'` and call `filesize(size, { round: 1 })`.
- `$devices` in the **template** still resolves via `globalProperties` (registered in Task 9) — leave template usages alone. The `refresh()` and `uploadNative()` functions need the store in **script**, where there is no `this`: inject it with `const devices = inject(DEVICES_KEY) as GenericDevices` (Task 2). Do not import the desktop store directly — that would break the web consumer.

- [ ] **Step 3: Port FileView.vue**

Convert to `<script setup lang="ts">`. Specifics:

- Props via `defineProps` + `withDefaults`: `files: FileInfo[]` default `[]`, `showHidden?: boolean` default `false`.
- `defineEmits<{ nav: [path: string]; select: [files: FileInfo[]] }>()`.
- `selected: number[] = []` → `const selected = ref<number[]>([])`.
- `@Watch('selected', {deep: true, immediate: true})` → `watch(selected, ..., { deep: true, immediate: true })` emitting `select`.
- `@Watch('files')` → `watch(() => props.files, () => { selected.value = [] })`.
- `filteredFiles` and `formatPath` become `computed` / a function.
- The template's mouse modifiers (`@mousedown.exact`, `.ctrl.stop`, `.shift.stop`, `@dblclick`) are all still valid in Vue 3 — leave them.

- [ ] **Step 4: Verify**

Run: `cd /home/jjureta/Projects/n-link && npm run build --workspace desktop`
Expected: succeeds.

- [ ] **Step 5: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add n-link-core/components/CalcInfo.vue n-link-core/components/FileView.vue
git commit -m "feat(core): port CalcInfo and FileView to Vue 3 script setup"
```

---

### Task 5: Port DeviceSelect.vue and DeviceQueue.vue — element-plus

**Files:**
- Modify: `n-link-core/components/DeviceSelect.vue`, `n-link-core/components/DeviceQueue.vue`

**Interfaces:**
- Consumes: types from `devices.ts`.
- Produces:
  - `<device-select v-model:selected="string | null" :scan-hint="boolean" />`
  - `<device-queue :device="Device" />`

- [ ] **Step 1: Port DeviceSelect.vue**

Four distinct migrations in this one file:

1. **`@PropSync('selected')`** has no Vue 3 equivalent. Replace with `const selectedCalculator = defineModel<string | null>('selected')`. At the call site this changes `:selected.sync="x"` to `v-model:selected="x"` (Task 9 updates `Home.vue`).
2. **element-ui → element-plus.** Replace
   `import ElPopover from 'element-ui/packages/popover/src/main.vue'` and
   `import 'element-ui/lib/theme-chalk/popover.css'`
   with
   `import { ElPopover } from 'element-plus'` and
   `import 'element-plus/es/components/popover/style/css'`.
3. **Popover visibility binding.** element-ui used `v-model="active"`; element-plus uses `v-model:visible="active"`.
4. **Slot syntax.** `<div slot="reference">` is Vue 2. Replace with `<template #reference><div>…</div></template>`.

Also replace the webpack tilde image:
`<img src="~feather-icons/dist/icons/refresh-cw.svg" class="w-5"/>`
becomes an ESM import — `import refreshCw from 'feather-icons/dist/icons/refresh-cw.svg'` — bound as `<img :src="refreshCw" class="w-5"/>`.

The `active` ref, `select()`, and the `calc` / `anyCalcs` computeds convert mechanically. Keep both `<style>` blocks (scoped and unscoped `.dev-select-pop`) as they are.

- [ ] **Step 2: Port DeviceQueue.vue**

Same element-plus popover migration and the same tilde-image fix (`x-circle.svg`). `@Prop({type: Object}) device` → `defineProps<{ device: Device }>()`.

- [ ] **Step 3: Verify**

Run: `cd /home/jjureta/Projects/n-link && npm run build --workspace desktop`
Expected: succeeds.

- [ ] **Step 4: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add n-link-core/components/DeviceSelect.vue n-link-core/components/DeviceQueue.vue
git commit -m "feat(core): port DeviceSelect and DeviceQueue to Vue 3 + element-plus

@PropSync -> defineModel, element-ui popover -> element-plus
(v-model:visible), slot=\"reference\" -> #reference, and webpack tilde
image paths -> ESM imports."
```

---

### Task 6: Port FileData.vue

**Files:**
- Modify: `n-link-core/components/FileData.vue`

**Interfaces:**
- Consumes: types from `devices.ts`; element-plus popover pattern from Task 5.
- Produces: `<file-data :files="FileInfo[]" :show-hidden="boolean" :dev="string" :path="string" :native-upload="boolean" />`

This is the most involved component: three `el-popover` instances (file info, delete confirmation, create-directory), each needing the Task 5 treatment.

- [ ] **Step 1: Read the source in full**

`n-link-core/components/FileData.vue`, 163 lines.

- [ ] **Step 2: Port it**

Every change this file needs, enumerated — do not infer them from other tasks:

**Imports.** Replace
```ts
import { Component, Prop, Vue } from 'vue-property-decorator';
import ElPopover from 'element-ui/packages/popover/src/main.vue';
import 'element-ui/lib/theme-chalk/popover.css';
```
with
```ts
import { computed, inject, ref } from 'vue';
import { ElPopover } from 'element-plus';
import 'element-plus/es/components/popover/style/css';
import { DEVICES_KEY, type GenericDevices, type FileInfo } from './devices';

const devices = inject(DEVICES_KEY) as GenericDevices;
```

**Props.** The class had `@Prop` declarations for `files`, `showHidden`, `dev`, `path`, and `nativeUpload`. Replace with:
```ts
const props = withDefaults(defineProps<{
  files: FileInfo[];
  dev: string;
  path: string;
  showHidden?: boolean;
  nativeUpload?: boolean;
}>(), {
  showHidden: false,
  nativeUpload: false,
});
```

**State.** `deletePopup`, `createDirPopup`, and `newName` become `ref(false)`, `ref(false)`, and `ref('')`.

**Computeds.** Every `get x()` on the class becomes `const x = computed(() => ...)`. Inside the script, referencing a prop needs `props.`, and referencing a ref needs `.value` — the class form had neither.

**Template — three popovers.** Each `<el-popover>` needs two changes:
1. `v-model="deletePopup"` → `v-model:visible="deletePopup"` (element-plus renamed the binding). Same for `createDirPopup`. The first popover has no `v-model` — leave it.
2. Any `<x slot="reference">` → `<template #reference><x>…</x></template>`.

`@show="newName = ''"` on the create-directory popover is still valid in element-plus — keep it.

**`$refs`.** If the file uses `this.$refs.<name>`, replace with a matching `ref<HTMLElement | null>(null)` and access `.value`.

Template `$devices` usages stay as they are — `globalProperties` resolves them.

- [ ] **Step 3: Verify**

Run: `cd /home/jjureta/Projects/n-link && npm run build --workspace desktop`
Expected: succeeds.

- [ ] **Step 4: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add n-link-core/components/FileData.vue
git commit -m "feat(core): port FileData to Vue 3 + element-plus"
```

---

### Task 7: Port FileBrowser.vue — replace vue-async-computed

**Files:**
- Modify: `n-link-core/components/FileBrowser.vue`

**Interfaces:**
- Consumes: `FileView` (Task 4), `FileData` (Task 6), `DeviceQueue` (Task 5).
- Produces: `<file-browser :dev="string" :show-hidden="boolean" :native-upload="boolean" />`

**Why this is its own task:** it is the only consumer of `vue-async-computed`, a Vue 2–only library with no Vue 3 successor. It provides both the async directory listing and the spinner state (`$asyncComputed.files.updating`).

- [ ] **Step 1: Replace the async computed with an explicit watcher**

Do not add a dependency for this. One call site does not justify `@vueuse/core`. Hand-roll it:

```ts
import { inject, ref, watch } from 'vue';
import { DEVICES_KEY, type GenericDevices, type FileInfo } from './devices';

const devices = inject(DEVICES_KEY) as GenericDevices;

const files = ref<FileInfo[] | null>(null);
const loading = ref(false);
const updateIndex = ref(0);

async function loadFiles() {
  loading.value = true;
  try {
    const contents = await devices.listDir(props.dev, path.value);
    files.value = contents.map((file) => ({
      ...file,
      path: `${path.value}/${file.path}`,
    }));
  } catch (e) {
    console.error(e);
    files.value = null;
  } finally {
    loading.value = false;
  }
}

watch([() => props.dev, path, updateIndex], loadFiles, { immediate: true });
```

This preserves the original's three behaviours: it re-runs when `path` changes, it re-runs when `goToIndex` bumps `updateIndex` (that counter exists so clicking the *current* breadcrumb forces a refresh), and it exposes a `loading` flag for the spinner.

One deliberate behavioural difference: the original let the error propagate out of the async computed, which left the previous listing on screen. Setting `files.value = null` instead shows the empty state. Given `list_dir` currently hangs rather than erroring on this hardware, neither path is reachable today — the explicit null is the clearer contract.

- [ ] **Step 2: Convert the rest of the component**

`path`, `selected` become refs; `parts` becomes a computed; `@Watch('path')` clearing `selected` becomes a `watch`; `goToIndex` stays a function. Replace the tilde image (`chevron-right.svg`) with an ESM import as in Task 5.

- [ ] **Step 3: Remove the dependency**

Confirm nothing else references it:

```bash
cd /home/jjureta/Projects/n-link
grep -rn "asyncComputed\|vue-async-computed" n-link-core desktop/src desktop/package.json || echo "clean"
```

Expected: `clean`. If `desktop/package.json` still lists `vue-async-computed`, remove it.

- [ ] **Step 4: Verify**

Run: `cd /home/jjureta/Projects/n-link && npm run build --workspace desktop`
Expected: succeeds.

- [ ] **Step 5: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add n-link-core/components/FileBrowser.vue desktop/package.json
git commit -m "feat(core): port FileBrowser to Vue 3, drop vue-async-computed

Replaces the Vue 2-only async computed with an explicit watch + refs,
preserving the path/updateIndex re-fetch triggers and the loading flag."
```

---

### Task 8: Port the device store

**Files:**
- Modify: `desktop/src/components/devices.ts`

**Interfaces:**
- Consumes: types and `GenericDevices` from `n-link-core/components/devices.ts`.
- Produces: a default-exported reactive store implementing `GenericDevices`, registered globally in Task 9.

**Why this carries the most risk:** it holds ~10 `$set`/`$delete` calls. Vue 3's proxy reactivity makes them unnecessary, so they are deleted rather than translated — and a mistake here produces silent non-reactivity rather than an error.

- [ ] **Step 1: Convert the class to a reactive object**

The Vue 2 form was `@Component class Devices extends Vue` instantiated with `new Devices()`. Vue 3 has no such pattern. Replace with `reactive()`:

```ts
import { reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import type { GenericDevices, Device, DevId, FileInfo, PartialCmd, Cmd } from 'n-link-core/components/devices';

const state = reactive<GenericDevices>({
  devices: {},
  enumerating: false,
  hasEnumerated: false,
  // ...methods assigned below
} as GenericDevices);
```

Note the import path changes: `@tauri-apps/api/tauri` → `@tauri-apps/api/core`, and `@tauri-apps/api/dialog` → `@tauri-apps/plugin-dialog`.

- [ ] **Step 2: Delete every `$set` / `$delete`**

Each has a direct plain-JS equivalent:

| Vue 2 | Vue 3 |
|---|---|
| `this.$set(this.devices, str, value)` | `state.devices[str] = value` |
| `this.$delete(this.devices, str)` | `delete state.devices[str]` |
| `this.$set(device, 'progress', payload)` | `device.progress = payload` |
| `this.$delete(device, 'progress')` | `delete device.progress` |
| `this.$set(device, 'queue', [])` | `device.queue = []` |
| `this.$set(device, 'running', true)` | `device.running = true` |

Verify none remain:

```bash
grep -n '\$set\|\$delete' desktop/src/components/devices.ts || echo "clean"
```
Expected: `clean`.

- [ ] **Step 3: Preserve the queue semantics exactly**

`runQueue` is the subtlest logic in the file. Keep these behaviours:
- the early return when `!device?.queue || device.running`
- the `while (true)` loop that exits when `this.devices[dev]` disappears (device removed mid-queue)
- `device.running = false` and return when the queue empties
- `console.error` on a failed command, then continue to the next
- deleting `progress` and shifting the queue after each command
- the trailing `await this.update(dev)`

Replace `this.` with `state.` throughout. `queueId` stays a module-level counter.

- [ ] **Step 4: Keep the dialog calls current**

`openDialog({ filters: [...], multiple: true })` and `openDialog({ directory: true })` keep the same shape in the v2 plugin. Their return types are `string | string[] | null`; the original's `as string | null` casts still apply. Preserve the null checks — `promptUploadFiles` iterates the result, so guard against `null` before looping.

- [ ] **Step 5: Verify**

Run: `cd /home/jjureta/Projects/n-link && npm run build --workspace desktop`
Expected: succeeds.

- [ ] **Step 6: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/src/components/devices.ts
git commit -m "feat(desktop): port device store to Vue 3 reactive

Replaces the Vue 2 component-as-store (new Devices() extending Vue) with
reactive(). All ~10 \$set/\$delete calls are deleted — Vue 3 proxy
reactivity tracks plain assignment and delete. Tauri 2 import paths."
```

---

### Task 9: Port the app shell and delete the harness

**Files:**
- Modify: `desktop/src/App.vue`, `desktop/src/main.ts`
- Create: `desktop/src/views/Home.vue`, `desktop/src/views/About.vue`, `desktop/src/router/index.ts`
- Delete: `desktop/src/assets/vue.svg`, `desktop/public/vite.svg`, `desktop/public/tauri.svg`

**Interfaces:**
- Consumes: every component from Tasks 3–7, the store from Task 8.
- Produces: the running application. This task removes Plan A's verification harness.

- [ ] **Step 1: Port the router to vue-router 4**

Create `desktop/src/router/index.ts`:

```ts
import { createRouter, createWebHashHistory } from 'vue-router';
import Home from '../views/Home.vue';

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'Home', component: Home },
    { path: '/about', name: 'About', component: () => import('../views/About.vue') },
  ],
});
```

`new VueRouter({ mode: 'history' })` becomes `createRouter({ history: ... })`, and `Vue.use(VueRouter)` is gone. **Use hash history, not web history**: the app is served from a custom protocol in production, where path-based history has no server to resolve deep links against. The original's `process.env.BASE_URL` is a Vue CLI variable that does not exist under Vite.

- [ ] **Step 2: Port Home.vue**

Copy the template from `git show 0472908:desktop/src/views/Home.vue` and convert the script. Specifics:

- `:selected.sync="selectedCalculator"` → `v-model:selected="selectedCalculator"` (matches Task 5's `defineModel`).
- The three `@Watch` decorators become `watch` calls. `@Watch('$devices.hasEnumerated')` and `@Watch('$devices.devices')` watch the imported store directly: `watch(() => devices.hasEnumerated, ...)` and `watch(() => devices.devices, ..., { deep: true })`.
- `import { open } from '@tauri-apps/api/shell'` → `import { open } from '@tauri-apps/plugin-shell'`.
- `class="form-checkbox h-5 w-5 text-blue-600 cursor-pointer"` — `@tailwindcss/forms` styles bare inputs rather than providing `.form-checkbox`. Drop `form-checkbox` and keep the rest.
- `navigator.platform` is deprecated but functional; keep it. The `isLinux` branch shows udev instructions and is correct on this machine.
- Keep the `sleep(1000)` race-avoidance in `onDeviceChange` — it exists so a device arriving slightly after enumeration still gets auto-selected.

- [ ] **Step 3: Port App.vue**

```vue
<template>
  <div id="app" class="h-screen">
    <router-view/>
  </div>
</template>

<style lang="scss">
@import 'n-link-core/assets/tailwind.css';

#app {
  user-select: none;
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}
</style>
```

The webpack tilde in `@import '~n-link-core/assets/tailwind.css'` is dropped — the Vite alias from Task 2 resolves the bare specifier.

This replaces Plan A's harness. That is the intended end of the harness's life.

- [ ] **Step 4: Port main.ts and register the store globally**

```ts
import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import devices from './components/devices';
import { DEVICES_KEY } from 'n-link-core/components/devices';
import 'n-link-core/assets/tailwind.css';

const app = createApp(App);
app.use(router);
// Two registrations, both required:
//   globalProperties -> templates across n-link-core reference $devices
//   provide          -> <script setup> blocks have no `this`, so they inject
app.config.globalProperties.$devices = devices;
app.provide(DEVICES_KEY, devices);
app.mount('#app');
```

Both lines are load-bearing. Omitting `provide` makes every core component that injects the store receive `undefined` and fail at runtime with a confusing null-property error rather than a build error. Omitting `globalProperties` makes every `$devices` reference in a template silently render nothing.

`new Vue({ render: h => h(App) }).$mount('#app')` becomes `createApp(App).mount('#app')`. `Vue.use(AsyncComputed)` is gone (Task 7). `Vue.config.productionTip` no longer exists in Vue 3.

`app.config.globalProperties.$devices` is what makes `$devices` resolve in templates; the `ComponentCustomProperties` augmentation from Task 2 is what makes TypeScript accept it.

- [ ] **Step 5: Port About.vue and remove scaffold assets**

`About.vue` is 5 lines — copy it from `git show 0472908:desktop/src/views/About.vue`, converting if it has any script block.

```bash
cd /home/jjureta/Projects/n-link/desktop
git rm --quiet src/assets/vue.svg public/vite.svg public/tauri.svg
```

Check `index.html` for a `<link rel="icon" href="/vite.svg">` and remove it if present.

- [ ] **Step 6: Verify the build**

Run: `cd /home/jjureta/Projects/n-link && npm run build --workspace desktop`
Expected: succeeds with no unresolved imports.

- [ ] **Step 7: Commit**

```bash
cd /home/jjureta/Projects/n-link
git add desktop/src desktop/index.html desktop/public
git commit -m "feat(desktop): port app shell to Vue 3, replace verification harness

Router to vue-router 4 with hash history (custom protocol has no server
for path-based deep links). createApp replaces new Vue. The device store
is registered on globalProperties so n-link-core templates keep resolving
\$devices. Removes Plan A's temporary harness and scaffold assets."
```

---

### Task 10: Verify against the calculator

**Files:** none (verification only)

**Interfaces:**
- Consumes: the complete application.
- Produces: evidence, recorded in the commit message and the progress ledger.

- [ ] **Step 1: Launch**

```bash
cd /home/jjureta/Projects/n-link/desktop && npm run tauri dev
```

**Do not run this from a snap-confined terminal** (for example VS Code installed as a snap) — snap injects an `LD_LIBRARY_PATH` whose `libpthread` shadows the system one and the binary dies with `undefined symbol: __libc_pthread_init`. Use a normal terminal.

- [ ] **Step 2: Verify the device list**

Expected: the sidebar's device dropdown lists the CX II by name ("TI-Nspire CX II CAS" once opened, "TI-Nspire(tm) CX II Handheld" before). The refresh button re-enumerates.

This exercises `DeviceSelect`, the store's `enumerate`, and the `addDevice` listener.

- [ ] **Step 3: Verify device info renders**

Expected: `CalcInfo` shows the name, OS version (6.4.0.74), the ID string, and two progress bars for storage and RAM with plausible values (roughly 22MB of 96MB storage used, per Plan A's verified `open_device` output).

This exercises `open_device` end to end through the new store and the Vue 3 reactivity — if `$set` removal broke anything, the panel will stay blank or fail to update.

- [ ] **Step 4: Verify hotplug**

Unplug the calculator. Expected: it disappears from the dropdown and the info panel clears. Replug. Expected: it reappears.

This exercises `removeDevice`/`addDevice` and — importantly — reactive deletion, the riskiest part of Task 8.

- [ ] **Step 5: Confirm the known file-browser limitation**

Expected: the file browser shows its loading spinner and does not populate, because `list_dir` hangs against this firmware.

**This is the known upstream libnspire limitation, not a defect in this port.** Two things to confirm, both of which are about *this* work:
- The spinner appears — proving the `loading` ref from Task 7 is wired correctly.
- The rest of the UI stays responsive: the dropdown still opens, the refresh button still works. This proves Task 1's async fix worked. If the whole window freezes, Task 1 is incomplete.

- [ ] **Step 6: Verify the release build**

```bash
cd /home/jjureta/Projects/n-link/desktop && npm run tauri build
./src-tauri/target/release/n-link
```

Expected: the real UI renders. A blank window means a CSP regression — Plan A's final review confirmed the current CSP allows all of this UI's images (bundled SVGs resolve to same-origin or `data:` URLs), so a blank screen would mean something new was introduced.

- [ ] **Step 7: Record the results**

```bash
cd /home/jjureta/Projects/n-link
git commit --allow-empty -m "test(desktop): verify Vue 3 UI against CX II

Device enumeration, selection, and info panel verified against hardware.
Hotplug add/remove verified. File browser hangs at list_dir — known
upstream libnspire limitation, confirmed the UI stays responsive during
the hang (Task 1's async fix working)."
```

---

## Definition of Done

- [ ] All 8 `n-link-core` components are Vue 3 `<script setup>`; no `vue-class-component`, `vue-property-decorator`, or `vue-async-computed` remains
- [ ] No `$set` / `$delete` anywhere in `desktop/src`
- [ ] No `require()` or webpack `~` specifiers in any ported file
- [ ] element-ui is fully replaced by element-plus
- [ ] `npm run build --workspace desktop` and `npm run tauri build` both succeed
- [ ] The device list, info panel, and hotplug all work against the CX II
- [ ] The UI stays responsive while `list_dir` hangs
- [ ] Plan A's harness `App.vue` is gone
- [ ] `web/` is untouched
- [ ] All work is on `port/tauri2-vue3`

## Known limitations carried forward

- **`list_dir` hangs** against current CX II firmware (frozen upstream libnspire). The file browser cannot be verified end to end. Owned by the user, out of scope here.
- **`web/` remains broken.** It imports `n-link-core`, which is Vue 3 after this plan while `web` is Nuxt 2. This was accepted in the spec and is deferred to its own project.
- **`{Unknown: n}` firmware enums.** `hw_type`, `run_level`, and `battery` deserialize to `{Unknown: n}` on 2026 firmware. Verified harmless: `CalcInfo` does not render any of them.
