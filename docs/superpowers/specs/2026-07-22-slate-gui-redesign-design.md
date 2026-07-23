# n-link "Slate" GUI redesign — design

**Date:** 2026-07-22
**Status:** Approved for planning
**Scope:** Visual restyle of the desktop app and the shared components it renders.

## Goal

Replace n-link's dated, generic look with a calm, modern dark-first visual
language ("Slate"): Inter typography, soft borders, one confident blue accent,
and full light/dark theming. The current three-pane workflow (device rail →
breadcrumb file browser → detail panel → transfer queue) stays exactly as it is.
This is a look-and-feel change, not a workflow or feature change.

## Non-goals

- No layout restructuring — panes, breadcrumbs, and flow are unchanged.
- No new features.
- No changes to Rust, the vendored `libnspire`, or `nlink-cli`.
- No manual theme-toggle UI (a `[data-theme]` hook is left for the future).
- The `web/` target is not directly reworked; it inherits whatever it renders
  from the shared `n-link-core` components.

## Affected code

- `desktop/` — Tauri/Vue app shell, Tailwind config, entry styles.
- `n-link-core/components/` — shared components rendered by the desktop app:
  `DeviceSelect.vue`, `CalcInfo.vue`, `FileBrowser.vue`, `FileView.vue`,
  `FileIcon.vue`, `FileData.vue`, `DeviceQueue.vue`.
- `n-link-core/assets/tailwind.css` and the Tailwind config.

## Design

### 1. Theming mechanic

Introduce a semantic color layer as CSS custom properties, defined once at the
app root:

- `--surface` — base window background
- `--surface-raised` — rails, cards, pills, popovers
- `--border` — subtle dividers
- `--text` — primary text
- `--text-muted` — secondary/metadata text
- `--accent` — the blue accent (buttons, active states)
- `--accent-fg` — text/icon color on top of accent
- `--selected` — selected file-row / grid background
- `--ok`, `--warn` — storage/RAM progress bars

These are exposed to Tailwind by mapping them to named colors in the Tailwind
config (e.g. `colors.surface`, `colors.accent`, `border.subtle`), so existing
utility-class usage is preserved in idiom. Hardcoded utilities
(`bg-blue-500`, `text-gray-700`, `border`, the `#4d88e8` in FileView) are swapped
to the semantic equivalents (`bg-accent`, `text-muted`, `border-subtle`,
`bg-selected`).

**Light/dark:**

- `:root` carries the **dark** token values by default (dark fallback).
- `@media (prefers-color-scheme: light)` overrides `:root` with light token
  values.
- Net behavior: the app follows the OS theme, and defaults to dark when the OS
  preference is unknown.
- A `:root[data-theme="dark"]` / `:root[data-theme="light"]` override pair is
  defined so a future manual toggle can force a theme, but no toggle UI ships in
  this work.

Palette (starting values, refined during implementation):

| Token             | Dark      | Light     |
|-------------------|-----------|-----------|
| `--surface`       | `#0f1117` | `#f7f8fa` |
| `--surface-raised`| `#161a25` | `#ffffff` |
| `--border`        | `#1d2130` | `#e6e9ef` |
| `--text`          | `#e6e8ee` | `#1c2330` |
| `--text-muted`    | ~55% text | ~55% text |
| `--accent`        | `#2d4bd8` | `#2d4bd8` |
| `--accent-fg`     | `#ffffff` | `#ffffff` |
| `--selected`      | `#1a2547` | `#e4ebff` |
| `--ok` / `--warn` | teal / amber | teal / amber |

### 2. Icon fix (required for dark mode)

Feather icons are currently loaded as `<img src="…svg">` (refresh-cw,
chevron-right, x-circle). These are black-stroke SVGs and cannot recolor, so
they would be invisible on dark surfaces. Convert these usages to **inline SVG
that inherits `currentColor`** (either small inline SVG or a `currentColor`
masking technique), so icons follow the theme. Affected: `DeviceSelect.vue`,
`FileBrowser.vue`, `DeviceQueue.vue`. `FileIcon.vue` is reviewed and adjusted
only if it has the same problem.

### 3. Remove Element-Plus, add three shared primitives

Element-Plus is used only for `ElPopover`, in `DeviceSelect.vue`,
`FileData.vue`, and `DeviceQueue.vue`. Replace it with a small in-repo set so
all UI shares one visual language:

- **`Popover.vue`** (n-link-core) — a headless, positioned popover with Slate
  styling. Replaces every `el-popover`. Must cover the three existing uses:
  device dropdown (anchored, click-to-open list), file-data actions, and the
  transfer-queue panel anchored to the progress ring.
- **`AppButton.vue`** (n-link-core) — `primary` and `secondary` variants with
  focus rings and disabled styling. Replaces the duplicated `.button` /
  `.gray-button` SCSS in `CalcInfo.vue`.
- **`ProgressBar.vue`** (n-link-core) — a themed bar taking a 0–1 (or
  used/total) value and a color token. Replaces the copy-pasted bar markup in
  `CalcInfo.vue` (storage, RAM) and `DeviceQueue.vue` (transfer progress).

Removing these usages lets the `element-plus` dependency be dropped from
`n-link-core/package.json` and `desktop/package.json`.

### 4. Per-component restyle

- **DeviceSelect** (top header bar): Slate top bar; the device dropdown uses the
  new `Popover` + `AppButton`/list styling; the "Click to connect a device"
  hint restyled onto accent surface.
- **CalcInfo**: cleaner info block; storage/RAM bars via `ProgressBar`
  (`--ok`/`--warn`); Refresh and Upload OS via `AppButton` primary/secondary.
- **FileView**: selection background from `--selected` instead of hardcoded
  `#4d88e8`; refined hover.
- **FileBrowser**: breadcrumb pills and the detail rail placed on Slate
  surfaces; active breadcrumb uses `--accent`.
- **DeviceQueue**: progress ring fill and the queue list recolored to tokens;
  panel via `Popover`; bar via `ProgressBar`.
- **Spinner**: `.lds-dual-ring` recolored to theme tokens (behavior unchanged).
- **Global** (`App.vue` / entry styles): font family `Avenir/Helvetica` → Inter
  with `system-ui` fallback; base `--surface`/`--text` applied to the app shell;
  token definitions loaded app-wide.

## Testing

This is a restyle, so verification is primarily visual and build-level:

1. `desktop` app builds with no `element-plus` import remaining (grep clean).
2. Both themes render correctly — verified by toggling the OS light/dark
   setting; dark is shown when OS preference is unset.
3. All three former-popover interactions work: device dropdown opens/selects,
   file-data actions, transfer-queue panel opens from the ring.
4. Icons are visible in both light and dark.
5. Storage/RAM/transfer progress bars display correct proportions.

There are no existing automated UI tests to preserve.

## Open implementation details (decided during build, not blockers)

- Exact final palette values and muted-text opacity.
- Popover positioning approach (lightweight manual positioning vs. a tiny
  floating-ui-style helper), chosen to keep dependencies minimal.
- Whether `FileIcon.vue` needs the same inline-SVG treatment.
