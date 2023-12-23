// eslint-disable-next-line import/no-extraneous-dependencies
require('@rushstack/eslint-patch/modern-module-resolution');

module.exports = {
    root: true,
    env: {
        browser: true,
        es2021: true,
    },
    extends: [
        'plugin:vue/vue3-recommended',
        'eslint:recommended',
        '@vue/eslint-config-airbnb-with-typescript',
        'prettier',
    ],
    overrides: [],
    plugins: ['vue', 'prettier'],
    rules: {
        // extra rules
        'prettier/prettier': 'warn',
        'vue/eqeqeq': 'error',
        // disabled rules
        'import/prefer-default-export': 'warn',
        'vue/multi-word-component-names': 'off',
        'vuejs-accessibility/form-control-has-label': 'off',
    },
    ignorePatterns: ['bindings/*.ts', 'vite.config.ts'],
    settings: {},
};
