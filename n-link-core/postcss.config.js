module.exports = {
  plugins: {
    tailwindcss: {},
    'vue-cli-plugin-tailwind/purgecss': {
      whitelistPatternsChildren: [/^el-/],
      content: ['./public/**/*.html', './src/**/*.vue', '../n-link-core/components/**/*.vue'],
    },
    autoprefixer: {}
  }
}
