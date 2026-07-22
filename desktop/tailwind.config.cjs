module.exports = {
  content: [
    './index.html',
    './src/**/*.{vue,js,ts}',
    '../n-link-core/components/**/*.{vue,js,ts}',
  ],
  presets: [require('../n-link-core/tailwind.config.js')],
};
