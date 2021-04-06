module.exports = {
  root: true,
  env: {
    es2020: true,
    browser: true,
    node: true,
  },
  extends: [
    '@nuxtjs/eslint-config-typescript',
    'prettier',
    'prettier/vue',
    'plugin:prettier/recommended',
    'plugin:nuxt/recommended',
  ],
  plugins: ['prettier'],
  // add your custom rules here
  rules: {
    'import/no-webpack-loader-syntax': 0,
    'require-await': 0,
    'unicorn/number-literal-case': 0,
    'no-console':
      process.env.NODE_ENV === 'production'
        ? ['warn', { allow: ['warn', 'error'] }]
        : 'off',
  },
};
