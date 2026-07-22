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
