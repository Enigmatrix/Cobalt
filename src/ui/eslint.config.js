import globals from "globals";
import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";
import pluginReact from "eslint-plugin-react";
import { FlatCompat } from "@eslint/eslintrc";

const compat = new FlatCompat();

/** @type {import('eslint').Linter.Config[]} */
export default [
  { files: ["app/**/*.{js,mjs,cjs,ts,jsx,tsx}"] },
  { languageOptions: { globals: globals.browser } },
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended,
  pluginReact.configs.flat["jsx-runtime"],
  ...compat.extends("plugin:react-hooks/recommended"),
];
