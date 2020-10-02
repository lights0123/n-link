module.exports = {
  plugins: {
    tailwindcss: {},
    'vue-cli-plugin-tailwind/purgecss': {
      whitelistPatternsChildren: [/^el-/],
    },
    autoprefixer: {}
  }
}
