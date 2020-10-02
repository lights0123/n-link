<template>
  <img :width="width" :src="icon" alt="name"/>
</template>

<script lang="ts">
import {Component, Prop, Vue} from 'vue-property-decorator';

@Component
export default class FileView extends Vue {
  @Prop({type: String, required: true}) private path!: string;
  @Prop({type: Boolean, default: false}) private dir!: string;
  @Prop({type: Number, default: 64}) private width!: number;

  get name() {
    return this.path.split('/').pop() as string;
  }

  get ext() {
    const parts = this.name.split('.');
    if (parts[parts.length - 1] === 'tns') parts.pop();
    if (parts.length < 2) return '';
    return parts.pop() as string;
  }

  get icon() {
    if (this.dir) {
      switch (this.name.trim().toLowerCase()) {
        case 'games':
          return require('../assets/folders/games.svg');
        case 'images':
        case 'photos':
          return require('../assets/folders/images.svg');
        case 'music':
        case 'sound':
        case 'sounds':
          return require('../assets/folders/music.svg');
        case 'script':
        case 'scripts':
        case 'program':
        case 'programs':
          return require('../assets/folders/scripts.svg');
        case 'template':
        case 'templates':
          return require('../assets/folders/templates.svg');
        case 'video':
        case 'videos':
          return require('../assets/folders/video.svg');
        default:
          return require('../assets/folders/folder.svg');
      }
    }
    if (this.name === 'ndless_resources.tns') return require('../assets/files/resources.svg');
    if (this.name.startsWith('ndless_installer')) return require('../assets/files/installer.svg');
    switch (this.ext.toLowerCase()) {
      case 'bin':
        return require('../assets/files/binary.svg');
      case 'ical':
      case 'ics':
      case 'ifb':
      case 'icalendar':
        return require('../assets/files/calendar.svg');
      case 'cfg':
        return require('../assets/files/cfg.svg');
      case 'vcf':
        return require('../assets/files/contact.svg');
      case 'csv':
      case 'log':
      case 'logs':
        return require('../assets/files/csv.svg');
      case 'sql':
      case 'db':
      case 'sqlite':
        return require('../assets/files/db.svg');
      case 'epub':
        return require('../assets/files/epub.svg');
      case 'eot':
      case 'otf':
      case 'ttf':
      case 'woff':
      case 'woff2':
        return require('../assets/files/font.svg');
      case 'png':
      case 'bmp':
      case 'jpg':
      case 'jpeg':
      case 'jpe':
      case 'jif':
      case 'jfif':
      case 'jfi':
      case 'jp2':
      case 'j2k':
      case 'jpf':
      case 'jpx':
      case 'jpm':
      case 'mj2':
      case 'gif':
      case 'tiff':
      case 'tif':
      case 'ppm':
      case 'pgm':
      case 'pbm':
      case 'pnm':
      case 'webp':
      case 'heif':
      case 'heifs':
      case 'heic':
      case 'heics':
      case 'avci':
      case 'avcs':
      case 'avif':
      case 'avifs':
      case 'ico':
      case 'icon':
      case 'icns':
      case 'pcx':
      case 'pgf':
      case 'tga':
      case 'psd':
      case 'xcf':
        return require('../assets/files/image.svg');
      case 'js':
      case 'mjs':
        return require('../assets/files/js.svg');
      case 'json':
      case 'json5':
        return require('../assets/files/json.svg');
      case 'lua':
        return require('../assets/files/lua.svg');
      case 'pdf':
        return require('../assets/files/pdf.svg');
      case 'py':
        return require('../assets/files/python.svg');
      case 'rom':
      case '89u':
      case 'smc':
      case 'sfc':
      case 'srm':
      case 'img':
        return require('../assets/files/rom.svg');
      case 'svg':
      case 'svgz':
        return require('../assets/files/svg.svg');
      case 'txt':
        return require('../assets/files/txt.svg');
      case 'mp4':
        return require('../assets/files/video.svg');
      case 'xml':
        return require('../assets/files/xml.svg');
      case 'zip':
      case 'tar':
      case 'gz':
        return require('../assets/files/zip.svg');
      default:
        return require('../assets/files/unknown.svg');
    }
  }
}
</script>

<style scoped lang="scss">
</style>
