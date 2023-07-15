require('@rushstack/eslint-patch/modern-module-resolution')

module.exports = {
  env: {
    browser: true,
    es2021: true,
    'vue/setup-compiler-macros': true,
  },
  ignorePatterns: [".eslintrc.cjs", "postcss.config.mjs"],
  plugins: [
    'vue',
    '@typescript-eslint',
  ],
  // Ref: https://eslint.vuejs.org/user-guide/#usage
  parser: 'vue-eslint-parser',
  parserOptions: {
    ecmaVersion: 12,
    parser: '@typescript-eslint/parser',
    sourceType: 'module',
    tsconfigRootDir: __dirname,
    project: ['./tsconfig.json'],
    extraFileExtensions: ['.vue'],
  },
  extends: [
    'plugin:vue/vue3-recommended',
    '@vue/eslint-config-typescript',
  ],
  rules: {
    // override/add rules settings here, such as:
    'vue/no-unused-vars': 'warn',
    '@typescript-eslint/no-unused-vars': ['warn', { 'argsIgnorePattern': '^_', 'varsIgnorePattern': '^_' }],
    'vue/script-setup-uses-vars': 'error',
    'vue/html-quotes': ['warn', 'single', { 'avoidEscape': true }],
    'vue/no-v-html': 'off',
    'vue/multi-word-component-names': ['error', { 'ignores': ['Icon', 'Paginator'] }],
    'vue/object-curly-spacing': ['error', 'always'],
    'object-curly-spacing': ['error', 'always'],
    'vue/comma-dangle': ['warn', 'always-multiline'],
    'comma-dangle': ['warn', 'always-multiline'],
    'vue/comma-spacing': ['error', { 'before': false, 'after': true }],
    'vue/block-spacing': ['error', 'always'],
    '@typescript-eslint/no-floating-promises': ['warn', {
      ignoreVoid: true,
    }],
  },
}
