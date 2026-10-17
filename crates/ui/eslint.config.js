import { defineConfig, globalIgnores } from "eslint/config";
import babelParser from "@babel/eslint-parser";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tailwindcss from "eslint-plugin-tailwindcss";

function buildLanguageOptions(babelParserPlugins) {
  return {
    parser: babelParser,
    parserOptions: {
      requireConfigFile: false,
      babelOptions: {
        parserOpts: { plugins: babelParserPlugins },
      },
    },
  };
}

const compilerBailoutRules = {
  "react-hooks/todo": "error",
  "react-hooks/invariant": "error",
  "react-hooks/syntax": "error",
};

export default defineConfig([
  globalIgnores(["dist", "node_modules", "src-tauri", "src/bindings.ts"]),
  {
    files: ["src/**/*.tsx"],
    extends: [
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
      tailwindcss.configs.recommended,
    ],
    languageOptions: buildLanguageOptions(["typescript", "jsx"]),
    settings: {
      tailwindcss: {
        cssConfigPath: "src/App.css",
      },
    },
    rules: {
      ...compilerBailoutRules,
      "tailwindcss/no-custom-classname": [
        "warn",
        { whitelist: ["fi", "fi\\-[a-z]{2}", "scroll-fade", "input"] },
      ],
    },
  },
  {
    files: ["src/**/*.ts"],
    extends: [reactHooks.configs.flat.recommended],
    languageOptions: buildLanguageOptions(["typescript"]),
    rules: compilerBailoutRules,
  },
  {
    files: ["src/contexts/**/*.tsx"],
    rules: {
      "react-refresh/only-export-components": "off",
    },
  },
]);
