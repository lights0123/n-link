# Slate GUI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restyle the n-link desktop app with a dark-first "Slate" visual language (semantic CSS-variable theming, light/dark via OS preference, Inter type) while keeping the existing three-pane workflow unchanged.

**Architecture:** Introduce a semantic color-token layer as CSS custom properties mapped into Tailwind, then migrate every hardcoded color utility to a token. Replace Element-Plus's `ElPopover` (its only usage) and duplicated button/progress/icon markup with four small in-repo components (`Popover`, `AppButton`, `ProgressBar`, `Icon`) that carry real logic and get unit tests. The remaining work is per-component restyle verified by build + visual check.

**Tech Stack:** Vue 3 (`<script setup>`), Vite 6, Tailwind 3, Tauri 2, SCSS. New: Vitest + @vue/test-utils + happy-dom for the logic-bearing primitives; @fontsource/inter for bundled typography.

## Global Constraints

- **No workflow/layout changes.** Panes, breadcrumbs, and flow stay as they are today. Visual restyle only.
- **No changes** to Rust, the vendored `libnspire`, or `nlink-cli`.
- **Dark is the default/fallback theme.** The app follows the OS `prefers-color-scheme`; when unknown it renders dark.
- **Semantic tokens only.** After this work, no component may use a hardcoded palette color (`bg-blue-500`, `text-gray-700`, `#4d88e8`, `bg-teal-400`, etc.) or a bare `border` utility for themed borders — use the token utilities defined in Task 2.
- **No `element-plus` import may remain** anywhere under `n-link-core/` or `desktop/` when the plan is complete.
- Token utility names (defined in Task 2, used everywhere after): `bg-surface`, `bg-raised`, `border-edge`, `text-fg`, `text-muted`, `bg-accent` / `text-accent` / `text-accent-fg`, `bg-selected`, `bg-ok`, `bg-warn`, `bg-danger`, `bg-success`.
- All new components live in `n-link-core/components/` and are imported by desktop views via the existing `n-link-core/...` path alias.
- Feather icons already ship with `stroke="currentColor"`; the only fix needed is rendering them inline (not via `<img>`) so they inherit theme color.
- Visual verification: run `npm run dev` in `desktop/` and open `http://localhost:1420` in a browser (device features need the full Tauri build, but layout/theme render in the plain browser). Do **not** rely on `npm run tauri dev` for visual checks in this environment — it fails to launch under snap VS Code (a known env issue, not a code issue).

---

## File Structure

**New files:**
- `n-link-core/components/ProgressBar.vue` — themed progress bar; one responsibility: render a 0–1 fraction as a filled track in a token color.
- `n-link-core/components/AppButton.vue` — themed button with variants; replaces duplicated `.button` SCSS.
- `n-link-core/components/Popover.vue` — headless anchored popover with outside-click / Escape close; replaces `el-popover`.
- `n-link-core/components/Icon.vue` — inline feather-icon renderer that inherits `currentColor`.
- `n-link-core/components/*.spec.ts` — unit tests colocated with the four primitives.
- `desktop/vitest.config.ts` — Vitest config (happy-dom, vue plugin, `n-link-core` alias, cross-package include).

**Modified files:**
- `n-link-core/assets/tailwind.css` — token definitions (dark default + light), remove dead `.el-popover` rule.
- `n-link-core/tailwind.config.js` — semantic token colors, Inter in `fontFamily.sans`, remove dead `ui.*` palette.
- `n-link-core/package.json` — add `feather-icons`; drop `element-plus` peer dep.
- `desktop/package.json` — add `vitest`, `@vue/test-utils`, `happy-dom`, `@fontsource/inter`, `test` script; drop `element-plus`.
- `desktop/src/main.ts` — import Inter font CSS.
- `desktop/src/App.vue` — app-shell base background/text/font from tokens.
- `desktop/src/views/Home.vue` — token migration for the device sidebar + spinner.
- `n-link-core/components/DeviceSelect.vue` — Popover + Icon + tokens.
- `n-link-core/components/CalcInfo.vue` — AppButton + ProgressBar + tokens.
- `n-link-core/components/DeviceQueue.vue` — Popover + ProgressBar + Icon + tokens.
- `n-link-core/components/FileData.vue` — Popover + AppButton + tokens.
- `n-link-core/components/FileBrowser.vue` — breadcrumb + detail-rail tokens, Icon.
- `n-link-core/components/FileView.vue` — selection/hover tokens.

`FileIcon.vue` is intentionally **not** changed: it renders multicolor file-type glyphs that must keep their own colors, so `currentColor` inheritance does not apply.

---

## Task 1: Test infrastructure

**Files:**
- Create: `desktop/vitest.config.ts`
- Modify: `desktop/package.json`
- Test: `n-link-core/components/smoke.spec.ts` (temporary, deleted in Step 6)

**Interfaces:**
- Produces: a working `npm test` (alias `vitest run`) in `desktop/` that discovers `*.spec.ts` under `n-link-core/components/` and resolves `.vue` single-file components.

- [ ] **Step 1: Add dev dependencies and test script**

In `desktop/package.json`, add to `devDependencies`:
```json
"@vue/test-utils": "^2.4.6",
"happy-dom": "^15.11.7",
"vitest": "^2.1.8"
```
Add to `scripts`:
```json
"test": "vitest run"
```
Then run: `npm install` (from repo root, workspaces are configured).

- [ ] **Step 2: Write the Vitest config**

Create `desktop/vitest.config.ts`:
```ts
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';
import { fileURLToPath, URL } from 'node:url';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      'n-link-core': fileURLToPath(new URL('../n-link-core', import.meta.url)),
    },
  },
  test: {
    environment: 'happy-dom',
    include: ['../n-link-core/components/**/*.spec.ts', 'src/**/*.spec.ts'],
  },
});
```

- [ ] **Step 3: Write a smoke test that mounts a trivial SFC**

Create `n-link-core/components/smoke.spec.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import { defineComponent, h } from 'vue';

describe('vitest setup', () => {
  it('mounts a component', () => {
    const C = defineComponent({ render: () => h('div', 'hello') });
    const wrapper = mount(C);
    expect(wrapper.text()).toBe('hello');
  });
});
```

- [ ] **Step 4: Run the smoke test**

Run: `cd desktop && npm test`
Expected: PASS — 1 passed, prints "hello" test green.

- [ ] **Step 5: Commit**

```bash
git add desktop/package.json desktop/vitest.config.ts package-lock.json n-link-core/components/smoke.spec.ts
git commit -m "test: set up vitest for n-link-core components"
```

- [ ] **Step 6: Remove the smoke test and commit**

```bash
git rm n-link-core/components/smoke.spec.ts
git commit -m "test: drop vitest smoke test"
```

---

## Task 2: Semantic theme tokens

**Files:**
- Modify: `n-link-core/assets/tailwind.css`
- Modify: `n-link-core/tailwind.config.js`
- Modify: `desktop/package.json`
- Modify: `desktop/src/main.ts`
- Modify: `desktop/src/App.vue`

**Interfaces:**
- Produces: the token utility classes listed in Global Constraints, backed by CSS variables that switch on OS theme. Every later task consumes these.

- [ ] **Step 1: Add the Inter font dependency**

In `desktop/package.json` `dependencies`, add:
```json
"@fontsource/inter": "^5.1.0"
```
Run: `npm install` from repo root.

- [ ] **Step 2: Define the token variables in the global stylesheet**

Replace the entire contents of `n-link-core/assets/tailwind.css` with:
```css
@tailwind base;

@tailwind components;

@tailwind utilities;

/* Slate theme tokens. Dark is the default (and the fallback when the OS
   preference is unknown); light applies when the OS asks for it. The
   [data-theme] blocks let a future manual toggle force a theme. */
:root,
:root[data-theme='dark'] {
    --surface: #0f1117;
    --raised: #161a25;
    --edge: #1d2130;
    --fg: #e6e8ee;
    --muted: #8b91a3;
    --accent: #2d4bd8;
    --accent-fg: #ffffff;
    --selected: #1a2547;
    --ok: #2dd4bf;
    --warn: #f5b942;
    --danger: #ef4444;
    --success: #22c55e;
}

@media (prefers-color-scheme: light) {
    :root:not([data-theme='dark']) {
        --surface: #f7f8fa;
        --raised: #ffffff;
        --edge: #e6e9ef;
        --fg: #1c2330;
        --muted: #6b7280;
        --accent: #2d4bd8;
        --accent-fg: #ffffff;
        --selected: #e4ebff;
        --ok: #0d9488;
        --warn: #d97706;
        --danger: #dc2626;
        --success: #16a34a;
    }
}

:root[data-theme='light'] {
    --surface: #f7f8fa;
    --raised: #ffffff;
    --edge: #e6e9ef;
    --fg: #1c2330;
    --muted: #6b7280;
    --accent: #2d4bd8;
    --accent-fg: #ffffff;
    --selected: #e4ebff;
    --ok: #0d9488;
    --warn: #d97706;
    --danger: #dc2626;
    --success: #16a34a;
}

@keyframes lds-dual-ring {
    0% {
        transform: rotate(0deg);
    }
    100% {
        transform: rotate(360deg);
    }
}
```
(The dead `.el-popover` rule is dropped — Element-Plus is being removed. The light values are intentionally duplicated across the media query and the `[data-theme='light']` override; CSS variables require concrete values in each block.)

- [ ] **Step 3: Map tokens into Tailwind and add Inter**

In `n-link-core/tailwind.config.js`: (a) prepend `'Inter'` to `theme.fontFamily.sans`; (b) replace the entire `colors: { ui: {...} }` block under `theme.extend` with the semantic token colors below (the `ui.*` palette is unused dead scaffolding — remove it):
```js
      colors: {
        surface: 'var(--surface)',
        raised: 'var(--raised)',
        edge: 'var(--edge)',
        fg: 'var(--fg)',
        muted: 'var(--muted)',
        accent: {
          DEFAULT: 'var(--accent)',
          fg: 'var(--accent-fg)',
        },
        selected: 'var(--selected)',
        ok: 'var(--ok)',
        warn: 'var(--warn)',
        danger: 'var(--danger)',
        success: 'var(--success)',
      },
```
The `fontFamily.sans` array's first element becomes `'Inter',` followed by the existing `'Cantarell', 'Roboto', ...` entries unchanged.

- [ ] **Step 4: Import the Inter font**

In `desktop/src/main.ts`, add these imports directly below the existing `import 'n-link-core/assets/tailwind.css';` line:
```ts
import '@fontsource/inter/400.css';
import '@fontsource/inter/500.css';
import '@fontsource/inter/600.css';
import '@fontsource/inter/700.css';
```

- [ ] **Step 5: Apply base tokens to the app shell**

In `desktop/src/App.vue`, replace the `<style lang="scss">` block with:
```scss
@import 'n-link-core/assets/tailwind.css';

#app {
  user-select: none;
  font-family: 'Inter', theme('fontFamily.sans');
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  background-color: var(--surface);
  color: var(--fg);
}
```
(Removes the hardcoded `color: #2c3e50` and the `Avenir, Helvetica, Arial` stack.)

- [ ] **Step 6: Verify the app builds and tokens resolve**

Run: `cd desktop && npm run build`
Expected: build succeeds (no vue-tsc or Tailwind errors).

Then run: `cd desktop && npm run dev`, open `http://localhost:1420`. Expected: the window background is dark slate (`#0f1117`) and text is light; switching your OS to light mode and reloading shows the light palette. Stop the dev server when confirmed.

- [ ] **Step 7: Commit**

```bash
git add n-link-core/assets/tailwind.css n-link-core/tailwind.config.js \
        desktop/package.json desktop/src/main.ts desktop/src/App.vue package-lock.json
git commit -m "feat(ui): add Slate semantic theme tokens and Inter font"
```

---

## Task 3: ProgressBar component

**Files:**
- Create: `n-link-core/components/ProgressBar.vue`
- Test: `n-link-core/components/ProgressBar.spec.ts`

**Interfaces:**
- Produces: `ProgressBar.vue` with props `{ fraction: number; color?: 'accent' | 'ok' | 'warn' }` (default `'accent'`). `fraction` is 0–1 filled and is clamped to that range. The filled element carries `data-testid="fill"` and an inline `width` percentage, and a `bg-{color}` class.

- [ ] **Step 1: Write the failing test**

Create `n-link-core/components/ProgressBar.spec.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ProgressBar from './ProgressBar.vue';

const fill = (w: ReturnType<typeof mount>) => w.get('[data-testid="fill"]');

describe('ProgressBar', () => {
  it('renders fraction as a width percentage', () => {
    const wrapper = mount(ProgressBar, { props: { fraction: 0.5 } });
    expect(fill(wrapper).attributes('style')).toContain('width: 50%');
  });

  it('clamps fractions below 0 and above 1', () => {
    expect(fill(mount(ProgressBar, { props: { fraction: -0.3 } }))
      .attributes('style')).toContain('width: 0%');
    expect(fill(mount(ProgressBar, { props: { fraction: 1.8 } }))
      .attributes('style')).toContain('width: 100%');
  });

  it('applies the accent color class by default', () => {
    const wrapper = mount(ProgressBar, { props: { fraction: 0.5 } });
    expect(fill(wrapper).classes()).toContain('bg-accent');
  });

  it('applies a chosen color class', () => {
    const wrapper = mount(ProgressBar, { props: { fraction: 0.5, color: 'ok' } });
    expect(fill(wrapper).classes()).toContain('bg-ok');
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd desktop && npm test -- ProgressBar`
Expected: FAIL — cannot resolve `./ProgressBar.vue`.

- [ ] **Step 3: Write the component**

Create `n-link-core/components/ProgressBar.vue`:
```vue
<template>
  <div class="w-full bg-edge rounded-full overflow-hidden">
    <div data-testid="fill" class="py-1 rounded-full" :class="`bg-${color}`"
         :style="{ width: `${percent}%` }"/>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  fraction: number;
  color?: 'accent' | 'ok' | 'warn';
}>(), {
  color: 'accent',
});

const percent = computed(() => Math.min(100, Math.max(0, props.fraction * 100)));
</script>
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd desktop && npm test -- ProgressBar`
Expected: PASS — 4 passed.

- [ ] **Step 5: Commit**

```bash
git add n-link-core/components/ProgressBar.vue n-link-core/components/ProgressBar.spec.ts
git commit -m "feat(ui): add ProgressBar component"
```

---

## Task 4: AppButton component

**Files:**
- Create: `n-link-core/components/AppButton.vue`
- Test: `n-link-core/components/AppButton.spec.ts`

**Interfaces:**
- Produces: `AppButton.vue` with props `{ variant?: 'primary' | 'secondary' | 'danger' | 'success'; size?: 'md' | 'sm'; disabled?: boolean }` (defaults `'primary'`, `'md'`, `false`). Renders a `<button>` wrapping the default slot, forwards native `click`, and reflects `disabled`.

- [ ] **Step 1: Write the failing test**

Create `n-link-core/components/AppButton.spec.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import AppButton from './AppButton.vue';

describe('AppButton', () => {
  it('renders slot content', () => {
    const wrapper = mount(AppButton, { slots: { default: 'Download' } });
    expect(wrapper.text()).toBe('Download');
  });

  it('applies the primary accent style by default', () => {
    const wrapper = mount(AppButton, { slots: { default: 'Go' } });
    expect(wrapper.get('button').classes()).toContain('bg-accent');
  });

  it('applies the danger variant', () => {
    const wrapper = mount(AppButton, {
      props: { variant: 'danger' }, slots: { default: 'Delete' },
    });
    expect(wrapper.get('button').classes()).toContain('bg-danger');
  });

  it('emits click when enabled', async () => {
    const wrapper = mount(AppButton, { slots: { default: 'Go' } });
    await wrapper.get('button').trigger('click');
    expect(wrapper.emitted('click')).toHaveLength(1);
  });

  it('reflects the disabled prop', () => {
    const wrapper = mount(AppButton, {
      props: { disabled: true }, slots: { default: 'Go' },
    });
    expect(wrapper.get('button').attributes('disabled')).toBeDefined();
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd desktop && npm test -- AppButton`
Expected: FAIL — cannot resolve `./AppButton.vue`.

- [ ] **Step 3: Write the component**

Create `n-link-core/components/AppButton.vue`:
```vue
<template>
  <button class="rounded font-bold focus:outline-none focus-visible:ring-2
                 focus-visible:ring-accent transition-colors disabled:opacity-75
                 disabled:cursor-not-allowed"
          :class="[variantClass, sizeClass]" :disabled="disabled">
    <slot/>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  variant?: 'primary' | 'secondary' | 'danger' | 'success';
  size?: 'md' | 'sm';
  disabled?: boolean;
}>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
});

const variantClass = computed(() => ({
  primary: 'bg-accent text-accent-fg',
  secondary: 'bg-raised text-fg border border-edge',
  danger: 'bg-danger text-white',
  success: 'bg-success text-white',
}[props.variant]));

const sizeClass = computed(() => (props.size === 'sm' ? 'px-3 py-2 text-sm' : 'px-6 py-2.5'));
</script>
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd desktop && npm test -- AppButton`
Expected: PASS — 5 passed.

- [ ] **Step 5: Commit**

```bash
git add n-link-core/components/AppButton.vue n-link-core/components/AppButton.spec.ts
git commit -m "feat(ui): add AppButton component"
```

---

## Task 5: Popover component

**Files:**
- Create: `n-link-core/components/Popover.vue`
- Test: `n-link-core/components/Popover.spec.ts`

**Interfaces:**
- Produces: `Popover.vue` with a `visible` v-model (`defineModel<boolean>('visible')`), props `{ width?: number | string; placement?: 'bottom-start' | 'bottom-end' | 'top-start' }` (default `'bottom-start'`), a `#trigger` slot (bound `{ visible }`, click toggles), and a default slot for the panel. The panel renders only when visible, carries `data-testid="panel"`, and closes on outside `mousedown` or `Escape`. Consumers can also set `visible` directly (used by FileData's forms).

- [ ] **Step 1: Write the failing test**

Create `n-link-core/components/Popover.spec.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import Popover from './Popover.vue';

function make() {
  return mount(Popover, {
    attachTo: document.body,
    slots: {
      trigger: '<button class="trg">open</button>',
      default: '<div class="content">panel body</div>',
    },
  });
}

describe('Popover', () => {
  it('hides the panel initially', () => {
    expect(make().find('[data-testid="panel"]').exists()).toBe(false);
  });

  it('opens when the trigger is clicked', async () => {
    const wrapper = make();
    await wrapper.get('.trg').trigger('click');
    expect(wrapper.find('[data-testid="panel"]').exists()).toBe(true);
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([true]);
  });

  it('closes on Escape', async () => {
    const wrapper = make();
    await wrapper.get('.trg').trigger('click');
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="panel"]').exists()).toBe(false);
  });

  it('closes on an outside mousedown', async () => {
    const wrapper = make();
    await wrapper.get('.trg').trigger('click');
    document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="panel"]').exists()).toBe(false);
  });

  it('applies a width style to the panel', async () => {
    const wrapper = mount(Popover, {
      attachTo: document.body,
      props: { width: 200 },
      slots: { trigger: '<button class="trg">o</button>', default: '<div>b</div>' },
    });
    await wrapper.get('.trg').trigger('click');
    expect(wrapper.get('[data-testid="panel"]').attributes('style')).toContain('width: 200px');
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd desktop && npm test -- Popover`
Expected: FAIL — cannot resolve `./Popover.vue`.

- [ ] **Step 3: Write the component**

Create `n-link-core/components/Popover.vue`:
```vue
<template>
  <div ref="root" class="relative inline-block">
    <div @click="toggle">
      <slot name="trigger" :visible="visible"/>
    </div>
    <div v-if="visible" data-testid="panel"
         class="absolute z-50 mt-1 bg-raised border border-edge rounded-lg shadow-lg
                p-3 select-none" :class="placementClass" :style="panelStyle">
      <slot/>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';

const props = withDefaults(defineProps<{
  width?: number | string;
  placement?: 'bottom-start' | 'bottom-end' | 'top-start';
}>(), {
  placement: 'bottom-start',
});

const visible = defineModel<boolean>('visible', { default: false });
const root = ref<HTMLElement | null>(null);

function toggle() {
  visible.value = !visible.value;
}

function onDocMousedown(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) visible.value = false;
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') visible.value = false;
}

watch(visible, (open) => {
  if (open) {
    document.addEventListener('mousedown', onDocMousedown);
    document.addEventListener('keydown', onKeydown);
  } else {
    document.removeEventListener('mousedown', onDocMousedown);
    document.removeEventListener('keydown', onKeydown);
  }
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocMousedown);
  document.removeEventListener('keydown', onKeydown);
});

const placementClass = computed(() => ({
  'bottom-start': 'top-full left-0',
  'bottom-end': 'top-full right-0',
  'top-start': 'bottom-full left-0 mb-1',
}[props.placement]));

const panelStyle = computed(() =>
  props.width === undefined
    ? undefined
    : { width: typeof props.width === 'number' ? `${props.width}px` : props.width });
</script>
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd desktop && npm test -- Popover`
Expected: PASS — 5 passed.

- [ ] **Step 5: Commit**

```bash
git add n-link-core/components/Popover.vue n-link-core/components/Popover.spec.ts
git commit -m "feat(ui): add Popover component to replace element-plus"
```

---

## Task 6: Icon component

**Files:**
- Create: `n-link-core/components/Icon.vue`
- Test: `n-link-core/components/Icon.spec.ts`
- Modify: `n-link-core/package.json`

**Interfaces:**
- Produces: `Icon.vue` with props `{ name: string; size?: number }` (default `20`). Renders the named feather icon inline (via `feather.icons[name].toSvg`), so the SVG's `stroke="currentColor"` inherits the surrounding text color. Throws `Error("unknown feather icon: <name>")` for an unknown name.

- [ ] **Step 1: Declare feather-icons as a dependency of n-link-core**

In `n-link-core/package.json`, add a `dependencies` block (the file currently has only `peerDependencies` and `devDependencies`):
```json
  "dependencies": {
    "feather-icons": "^4.29.2"
  },
```
Run: `npm install` from repo root.

- [ ] **Step 2: Write the failing test**

Create `n-link-core/components/Icon.spec.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import Icon from './Icon.vue';

describe('Icon', () => {
  it('renders an inline svg that inherits currentColor', () => {
    const wrapper = mount(Icon, { props: { name: 'refresh-cw' } });
    expect(wrapper.html()).toContain('<svg');
    expect(wrapper.html()).toContain('currentColor');
  });

  it('applies the size to width and height', () => {
    const wrapper = mount(Icon, { props: { name: 'chevron-right', size: 32 } });
    expect(wrapper.html()).toContain('width="32"');
    expect(wrapper.html()).toContain('height="32"');
  });

  it('throws for an unknown icon name', () => {
    expect(() => mount(Icon, { props: { name: 'not-a-real-icon' } }))
      .toThrow(/unknown feather icon/);
  });
});
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cd desktop && npm test -- Icon`
Expected: FAIL — cannot resolve `./Icon.vue`.

- [ ] **Step 4: Write the component**

Create `n-link-core/components/Icon.vue`:
```vue
<template>
  <span class="inline-flex items-center" v-html="svg"/>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import feather from 'feather-icons';

const props = withDefaults(defineProps<{
  name: string;
  size?: number;
}>(), {
  size: 20,
});

const svg = computed(() => {
  const icon = feather.icons[props.name as keyof typeof feather.icons];
  if (!icon) throw new Error(`unknown feather icon: ${props.name}`);
  return icon.toSvg({ width: props.size, height: props.size });
});
</script>
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cd desktop && npm test -- Icon`
Expected: PASS — 3 passed.

- [ ] **Step 6: Commit**

```bash
git add n-link-core/components/Icon.vue n-link-core/components/Icon.spec.ts \
        n-link-core/package.json package-lock.json
git commit -m "feat(ui): add inline Icon component for themeable feather icons"
```

---

## Task 7: Restyle DeviceSelect

**Files:**
- Modify: `n-link-core/components/DeviceSelect.vue`

**Interfaces:**
- Consumes: `Popover.vue` (Task 5), `Icon.vue` (Task 6), theme tokens (Task 2).

- [ ] **Step 1: Rewrite the component with Popover + Icon + tokens**

Replace the entire contents of `n-link-core/components/DeviceSelect.vue` with:
```vue
<template>
  <div class="header border-b border-edge px-2 py-2 flex w-full bg-raised">
    <button @click="$devices.enumerate()" class="flex-shrink-0 mr-2 focus:outline-none text-fg"
            :class="$devices.enumerating && 'cursor-not-allowed opacity-25'" :disabled="$devices.enumerating">
      <icon name="refresh-cw" :size="20"/>
      <div v-if="scanHint && Object.keys($devices.devices).length === 0" class="p-4 refresh-popup">
        Click to connect a device
      </div>
    </button>
    <popover v-model:visible="active" :width="239" class="w-full">
      <template #trigger>
        <div class="relative w-full focus:outline-none">
          <div class="block w-full bg-surface border border-edge hover:border-muted px-4 py-3/2 pr-8
                      rounded leading-tight focus:outline-none h-8 truncate text-fg">
            <span v-if="selectedCalculator">
              <small class="tabular-nums">{{ selectedCalculator }}</small>
              <span v-if="calc && calc.info"> {{ calc.info.name }}</span>
              <span v-else> {{ calc!.name }}</span>
            </span>
            <span v-else class="text-muted text-sm">
              Select a device...
            </span>
          </div>
          <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-muted">
            <svg class="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
              <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z"/>
            </svg>
          </div>
        </div>
      </template>
      <ul class="-m-3">
        <li v-for="(device, id) in $devices.devices" :key="id"
            class="p-2 hover:bg-accent hover:text-accent-fg w-full cursor-pointer text-fg" @click="select(id)">
          <small class="tabular-nums">{{ id }}</small>
          <span v-if="device.info"> {{ device.info.name }}</span>
          <span v-else> {{ device.name }}</span>
        </li>
        <div v-if="!anyCalcs" class="p-2 w-full text-muted">
          No calculators found
        </div>
      </ul>
    </popover>
  </div>
</template>

<script setup lang="ts">
import {computed, inject, ref} from 'vue';
import {DEVICES_KEY, type GenericDevices} from './devices';
import Popover from './Popover.vue';
import Icon from './Icon.vue';

withDefaults(defineProps<{
  scanHint?: boolean;
}>(), {
  scanHint: false,
});

const selectedCalculator = defineModel<string | null>('selected');

const devices = inject(DEVICES_KEY) as GenericDevices;

const active = ref(false);

function select(dev: string) {
  selectedCalculator.value = dev;
  active.value = false;
}

const calc = computed(() => {
  const id = selectedCalculator.value;
  return id ? devices.devices[id] : undefined;
});

const anyCalcs = computed(() => !!Object.keys(devices.devices).length);
</script>

<style scoped lang="scss">
.header {
  height: 50px;
}

.refresh-popup {
  $offset: 5px;
  $size: 9px;
  margin-left: -4.5px;
  margin-top: 10px;
  @apply absolute bg-accent text-accent-fg;
  &:before {
    content: "";
    position: absolute;
    left: $offset;
    top: $size * -2;
    border-top: $size solid transparent;
    border-right: $size solid transparent;
    border-bottom: $size solid var(--accent);
    border-left: $size solid transparent;
  }
}
</style>
```
(The `el-popover`, its `element-plus` imports, the `feather-icons/dist/icons/refresh-cw.svg` import, and the second `<style>` block for `.dev-select-pop` are all gone. The `<ul class="-m-3">` negative margin lets the list fill the Popover's `p-3` padding.)

- [ ] **Step 2: Verify build and no element-plus remains**

Run: `cd desktop && npm run build`
Expected: build succeeds.

Run: `grep -n "element-plus" n-link-core/components/DeviceSelect.vue`
Expected: no output.

- [ ] **Step 3: Visual check**

Run: `cd desktop && npm run dev`, open `http://localhost:1420`. Expected: the top bar is a raised slate surface, the refresh icon is visible in the theme color, and clicking the device selector opens a themed dropdown. Stop the dev server.

- [ ] **Step 4: Commit**

```bash
git add n-link-core/components/DeviceSelect.vue
git commit -m "feat(ui): restyle DeviceSelect with Popover, Icon, and Slate tokens"
```

---

## Task 8: Restyle CalcInfo

**Files:**
- Modify: `n-link-core/components/CalcInfo.vue`

**Interfaces:**
- Consumes: `AppButton.vue` (Task 4), `ProgressBar.vue` (Task 3), theme tokens.

- [ ] **Step 1: Replace the template markup**

In `n-link-core/components/CalcInfo.vue`, replace the storage/RAM bar `<div>`s and the two `<button>`s in the `<template>` with token-based markup. Replace the block from `<div :title="...free storage...">` through the closing of the RAM block with:
```vue
    <div :title="`${formatSize(info.free_storage)} free`">
      <small class="block select-text text-muted">
        Storage: {{ formatSize(info.total_storage - info.free_storage) }} / {{ formatSize(info.total_storage) }} used
      </small>
      <progress-bar class="mb-2" :fraction="1 - info.free_storage / info.total_storage" color="accent"/>
    </div>
    <div :title="`${formatSize(info.free_ram)} free`">
      <small class="block select-text text-muted">
        RAM: {{ formatSize(info.total_ram - info.free_ram) }} / {{ formatSize(info.total_ram) }} used
      </small>
      <progress-bar class="mb-2" :fraction="1 - info.free_ram / info.total_ram" color="ok"/>
    </div>
```
Replace the two buttons (`Refresh` and `Upload OS`) with:
```vue
    <app-button class="mt-4" :disabled="refreshing" @click="refresh">
      <div class="flex">
        <div v-if="refreshing" class="lds-dual-ring"/>
        Refresh
      </div>
    </app-button>
    <app-button class="mt-4" variant="secondary"
                @click="nativeUpload ? upload?.click() : $devices.uploadOs(dev, osExtension)">
      Upload OS
    </app-button>
```

- [ ] **Step 2: Update the script imports and drop the old button SCSS**

In the `<script setup>` block of `CalcInfo.vue`, add below the existing imports:
```ts
import AppButton from './AppButton.vue';
import ProgressBar from './ProgressBar.vue';
```
In the `<style scoped lang="scss">` block, delete the `.button` and `.gray-button` rules entirely, keeping only the `.lds-dual-ring` rules, and change the spinner border color from `white` to `var(--accent-fg)`:
```scss
.lds-dual-ring:after {
  content: " ";
  display: block;
  width: 64px;
  height: 64px;
  margin: 8px;
  border-radius: 50%;
  border: 6px solid var(--accent-fg);
  border-color: var(--accent-fg) transparent var(--accent-fg) transparent;
  animation: lds-dual-ring 1.2s linear infinite;
}
```
(Keep the `.lds-dual-ring` sizing rule above it unchanged.)

- [ ] **Step 3: Verify build**

Run: `cd desktop && npm run build`
Expected: build succeeds.

Run: `grep -nE "bg-blue|bg-gray|bg-teal" n-link-core/components/CalcInfo.vue`
Expected: no output.

- [ ] **Step 4: Commit**

```bash
git add n-link-core/components/CalcInfo.vue
git commit -m "feat(ui): restyle CalcInfo with AppButton and ProgressBar"
```

---

## Task 9: Restyle DeviceQueue

**Files:**
- Modify: `n-link-core/components/DeviceQueue.vue`

**Interfaces:**
- Consumes: `Popover.vue`, `ProgressBar.vue`, `Icon.vue`, theme tokens.

- [ ] **Step 1: Rewrite the template to use Popover, ProgressBar, and Icon**

In `n-link-core/components/DeviceQueue.vue`, replace the `<template>` block with:
```vue
<template>
  <div>
    <popover :width="300" placement="bottom-end">
      <template #trigger>
        <svg viewBox="-1 -1 2 2" class="circle focus:outline-none">
          <circle r="1" class="bg"/>
          <path class="fg" :d="pathData"/>
        </svg>
      </template>
      <div v-if="!queue.length" class="text-muted">
        Nothing to do
      </div>
      <div v-for="(item, i) in queue" :key="item.id" class="flex items-center text-fg">
        <div class="min-w-0 flex-grow">
          <p class="truncate">{{ item.desc }}</p>
          <div v-if="i === 0 && device.progress">
            <small class="block tabular-nums text-muted">
              {{ (100 - device.progress.remaining / device.progress.total * 100).toFixed(1) }}%
            </small>
            <progress-bar class="mb-2"
                          :fraction="1 - device.progress.remaining / device.progress.total" color="ok"/>
          </div>
        </div>
        <div class="ml-2 flex-shrink-0">
          <button class="focus:outline-none text-fg" :disabled="i===0" :class="i===0 && 'cursor-not-allowed'"
                  @click="device.queue?.splice(i, 1)">
            <icon name="x-circle" :size="20" :class="i===0 && 'opacity-25'"/>
          </button>
        </div>
      </div>
    </popover>
  </div>
</template>
```

- [ ] **Step 2: Update the script imports**

In the `<script setup>` block, remove the two `element-plus` import lines and the `xCircle` feather import, and add:
```ts
import Popover from './Popover.vue';
import ProgressBar from './ProgressBar.vue';
import Icon from './Icon.vue';
```
(The `computed`, `Device` type, `getCoordinatesForPercent`, `trimPath`, `pathData`, and `queue` logic all stay unchanged.)

- [ ] **Step 3: Recolor the progress ring**

In the `<style scoped lang="scss">` block, change the ring fills to tokens:
```scss
.circle {
  width: 32px;
  height: 32px;
  transform: rotate(-90deg);

  .bg {
    fill: var(--edge);
  }

  .fg {
    fill: var(--accent);
  }
}
```

- [ ] **Step 4: Verify build and no element-plus remains**

Run: `cd desktop && npm run build`
Expected: build succeeds.

Run: `grep -n "element-plus" n-link-core/components/DeviceQueue.vue`
Expected: no output.

- [ ] **Step 5: Commit**

```bash
git add n-link-core/components/DeviceQueue.vue
git commit -m "feat(ui): restyle DeviceQueue with Popover, ProgressBar, and Icon"
```

---

## Task 10: Restyle FileData

**Files:**
- Modify: `n-link-core/components/FileData.vue`

**Interfaces:**
- Consumes: `Popover.vue`, `AppButton.vue`, theme tokens.

- [ ] **Step 1: Rewrite the template with Popover + AppButton**

Replace the `<template>` block of `n-link-core/components/FileData.vue` with:
```vue
<template>
  <div class="flex flex-col h-full">
    <div v-if="files.length === 1">
      <div class="w-full flex flex-col items-center">
        <file-icon :path="files[0].path" :dir="files[0].isDir" :width="96"/>
        <p class="text-center w-full break-words text-fg">{{ formatPath(files[0]) }}</p>
        <p v-if="!files[0].isDir" class="mt-2 text-sm text-muted">{{ formatSize(files[0].size) }}</p>
      </div>
      <app-button v-if="!files[0].isDir" class="mt-4 w-full" @click="download">
        Download
      </app-button>
      <popover v-model:visible="renamePopup" :width="190" class="w-full block">
        <template #trigger>
          <app-button class="mt-4 w-full" variant="secondary" @click="newName = formatPath(files[0])">
            Rename
          </app-button>
        </template>
        <form @submit.prevent="rename">
          <input v-model="newName"
                 class="border border-edge rounded w-full py-2 px-3 bg-surface text-fg leading-tight focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
                 type="text" placeholder="New name">
          <app-button class="mt-4 w-full" variant="success" :disabled="!isValidName" type="submit">
            Rename
          </app-button>
        </form>
      </popover>
    </div>
    <div v-else-if="files.length && files.every(file => !file.isDir)">
      <app-button class="mt-4" @click="download">
        Download {{ files.length > 1 ? `${files.length} files` : '' }}
      </app-button>
    </div>
    <div v-if="files.length">
      <popover v-model:visible="deletePopup" :width="170" class="w-full block">
        <template #trigger>
          <app-button class="mt-4 w-full" variant="danger">
            Delete {{ files.length > 1 ? `${files.length} files` : '' }}
          </app-button>
        </template>
        <div class="text-fg">
          Delete {{ files.length }} file{{ files.length === 1 ? '' : 's' }}?
          <div class="flex w-full justify-between">
            <app-button class="mt-4" size="sm" variant="secondary" @click="deletePopup = false">
              Cancel
            </app-button>
            <app-button class="mt-4" size="sm" variant="danger" @click="deleteFiles">
              Delete
            </app-button>
          </div>
        </div>
      </popover>
    </div>
    <div class="flex-grow"/>
    <div class="pb-4">
      <popover v-model:visible="createDirPopup" :width="170" class="w-full block" placement="top-start">
        <template #trigger>
          <app-button class="mt-4 w-full" variant="secondary" size="sm" @click="newName = ''">
            Create directory
          </app-button>
        </template>
        <form @submit.prevent="$devices.createDir(dev, `${path}/${newName}`)">
          <input v-model="newName"
                 class="border border-edge rounded w-full py-2 px-3 bg-surface text-fg leading-tight focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
                 type="text" placeholder="New directory">
          <app-button class="mt-4 w-full" variant="success" :disabled="!newName.length" type="submit">
            Create
          </app-button>
        </form>
      </popover>
      <app-button class="mt-4 w-full" variant="secondary"
                  @click="nativeUpload ? upload?.click() : $devices.promptUploadFiles(dev, path)">
        Upload files
      </app-button>
      <input v-if="nativeUpload" ref="upload" type="file" class="hidden" multiple accept=".tns" @change="uploadNative" />
    </div>
  </div>
</template>
```
(Note: the rename form's submit button previously showed a disabled style via a `.disabled` class; `AppButton`'s `disabled` prop now handles that. The `createDir` popover uses `placement="top-start"` because it sits at the bottom of the rail.)

- [ ] **Step 2: Update the script imports and drop the button SCSS**

In the `<script setup>` block, remove the two `element-plus` import lines and add:
```ts
import AppButton from './AppButton.vue';
import Popover from './Popover.vue';
```
Delete the entire `<style scoped lang="scss">` block (all `.button` variants are now provided by `AppButton`).

- [ ] **Step 3: Verify build and no element-plus remains**

Run: `cd desktop && npm run build`
Expected: build succeeds.

Run: `grep -n "element-plus" n-link-core/components/FileData.vue`
Expected: no output.

- [ ] **Step 4: Commit**

```bash
git add n-link-core/components/FileData.vue
git commit -m "feat(ui): restyle FileData with Popover and AppButton"
```

---

## Task 11: Restyle FileView, FileBrowser, and Home

**Files:**
- Modify: `n-link-core/components/FileView.vue`
- Modify: `n-link-core/components/FileBrowser.vue`
- Modify: `desktop/src/views/Home.vue`

**Interfaces:**
- Consumes: `Icon.vue`, theme tokens.

- [ ] **Step 1: Token-migrate FileView selection**

In `n-link-core/components/FileView.vue`, replace the `<style scoped lang="scss">` block with:
```scss
.selected {
  @apply rounded bg-selected;
  p {
    @apply text-fg;
  }
}
```
(Removes the hardcoded `#4d88e8` background and `text-white`.)

- [ ] **Step 2: Token-migrate FileBrowser breadcrumbs and detail rail, use Icon**

In `n-link-core/components/FileBrowser.vue`: change the header `<div>` border to `border-b border-edge`; change the breadcrumb pill span classes from `bg-gray-200` to `bg-raised text-fg`; replace the chevron `<img :src="chevronRight" .../>` with `<icon name="chevron-right" :size="16" class="inline text-muted"/>`; change the detail rail wrapper `border-l` to `border-l border-edge`. In `<script setup>`, remove the `chevronRight` feather import and add `import Icon from './Icon.vue';`. In `<style scoped>`, change the active breadcrumb rule to:
```scss
.active span {
  @apply bg-accent text-accent-fg;
}
```
Recolor the `.lds-dual-ring:after` border from `theme('colors.gray.400')` to `var(--muted)`.

- [ ] **Step 3: Token-migrate Home sidebar and spinner**

In `desktop/src/views/Home.vue`: change the sidebar wrapper `border-r` to `border-r border-edge`; change the driver-message headings and the `text-blue-600` links to `text-accent`; change the checkbox label `text-gray-700` to `text-muted` and the checkbox `text-blue-600` to `text-accent`. In `<style scoped>`, change the `.lds-dual-ring:after` `$color` from `theme('colors.gray.400')` to `var(--muted)`.

- [ ] **Step 4: Verify build and no stray palette classes**

Run: `cd desktop && npm run build`
Expected: build succeeds.

Run: `grep -nE "bg-gray-200|text-blue-600|#4d88e8|colors.gray.400" n-link-core/components/FileView.vue n-link-core/components/FileBrowser.vue desktop/src/views/Home.vue`
Expected: no output.

- [ ] **Step 5: Visual check**

Run: `cd desktop && npm run dev`, open `http://localhost:1420`. Expected: breadcrumbs, file grid selection, detail rail, and sidebar all render in the Slate palette; the chevron icon is visible. Stop the dev server.

- [ ] **Step 6: Commit**

```bash
git add n-link-core/components/FileView.vue n-link-core/components/FileBrowser.vue desktop/src/views/Home.vue
git commit -m "feat(ui): restyle file browser, file view, and home sidebar with tokens"
```

---

## Task 12: Remove Element-Plus and final verification

**Files:**
- Modify: `n-link-core/package.json`
- Modify: `desktop/package.json`

**Interfaces:**
- Consumes: all prior tasks (every `el-popover` usage must already be gone).

- [ ] **Step 1: Confirm no element-plus usage remains anywhere**

Run: `grep -rn "element-plus\|el-popover\|ElPopover" --include=*.vue --include=*.ts n-link-core desktop | grep -v node_modules`
Expected: no output. (If anything prints, fix that file before continuing — it means a prior task's migration was incomplete.)

- [ ] **Step 2: Drop the dependency**

Remove the `"element-plus": "^2.14.3",` line from `desktop/package.json` `dependencies`, and remove the `"element-plus": "^2.14.3",` line from `n-link-core/package.json` `peerDependencies` (leaving `vue` in that block). Run: `npm install` from repo root.

- [ ] **Step 3: Full build and test sweep**

Run: `cd desktop && npm run build && npm test`
Expected: build succeeds; all unit tests pass (ProgressBar 4, AppButton 5, Popover 5, Icon 3).

- [ ] **Step 4: Full visual smoke test**

Run: `cd desktop && npm run dev`, open `http://localhost:1420`. Walk through: dark theme by default; switch OS to light and reload → light theme; the device dropdown, transfer-queue ring/panel, and file rename/delete/create-dir popovers all open and are themed; icons visible in both themes. Stop the dev server.

- [ ] **Step 5: Commit**

```bash
git add desktop/package.json n-link-core/package.json package-lock.json
git commit -m "chore: drop element-plus dependency"
```

---

## Self-Review Notes

- **Spec coverage:** Theming mechanic → Task 2; icon fix → Task 6 + integrations (7, 9, 11); remove Element-Plus + 3 primitives → Tasks 3–6, 12 (Popover replaces all three `el-popover` sites in Tasks 7, 9, 10); per-component restyle → Tasks 7–11; Inter/global shell → Task 2; testing → Vitest primitives + per-task build/grep/visual checks. `FileIcon.vue` explicitly excluded with rationale (multicolor glyphs).
- **Type consistency:** `ProgressBar` prop `fraction` (0–1) is used consistently in CalcInfo and DeviceQueue by passing `1 - free/total`. `Popover` uses `visible` v-model and `#trigger` slot consistently across DeviceSelect, DeviceQueue, FileData. `AppButton` variant names (`primary`/`secondary`/`danger`/`success`) match every call site. Token utility names match the Global Constraints list everywhere.
- **No placeholders:** every code step shows complete content.
