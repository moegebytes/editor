import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import eslintConfigPrettier from 'eslint-config-prettier';
import tsEslintPlugin from '@typescript-eslint/eslint-plugin';
import tsEslintParser from '@typescript-eslint/parser';

export default [
  {
    rules: {
      // external
      ...js.configs.recommended.rules,
      ...eslintConfigPrettier.rules,

      // overrides
      'max-len': ['error', { code: 200 }],
      'no-console': 'error',
    },
  },
  {
    files: ['src/**/*.ts'],
    ignores: ['src/**/*.svelte.ts'],
    rules: {
      ...tsEslintPlugin.configs.recommended.rules,

    },
    plugins: { '@typescript-eslint': tsEslintPlugin },
    languageOptions: {
      globals: globals.browser,
      parser: tsEslintParser,
    },
  },
  ...svelte.configs.recommended,
  {
    files: ['src/**/*.svelte', 'src/**/*.svelte.ts'],
    rules: {
      // external
      ...tsEslintPlugin.configs.recommended.rules,

      // overrides
      'svelte/require-each-key': 'off',
    },
    plugins: { '@typescript-eslint': tsEslintPlugin },
    languageOptions: {
      globals: globals.browser,
      parserOptions: {
        parser: tsEslintParser,
      },
    },
  },
  {
    files: ['*.config.js'],
    rules: {
      'no-console': 'off',
    },
    languageOptions: {
      globals: globals.node,
    },
  },
];
