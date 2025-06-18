import globals from "globals";
import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";
import pluginReact from "eslint-plugin-react";
import { FlatCompat } from "@eslint/eslintrc";

const compat = new FlatCompat();

/** @type {import('eslint').Linter.Config[]} */
export default [
  {
    ignores: ["src-tauri", "build", ".react-router", "app/components/ui"],
  },
  {
    files: ["app/**/*.{js,mjs,cjs,ts,jsx,tsx}"],
  },
  {
    languageOptions: {
      globals: globals.browser,
      parserOptions: {
        projectService: {
          allowDefaultProject: ["*.js"],
        },
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  pluginReact.configs.flat["jsx-runtime"],
  ...compat.extends("plugin:react-hooks/recommended"),
  {
    rules: {
      "@typescript-eslint/no-misused-promises": "off",
    },
  },
];
