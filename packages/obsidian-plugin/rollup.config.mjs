import { wasm } from "@rollup/plugin-wasm";
import { nodeResolve } from "@rollup/plugin-node-resolve";

export default {
  input: "src/main.js",
  output: {
    dir: ".",
    format: "cjs",
  },
  external: [
    "obsidian",
    "electron",
    "@codemirror/autocomplete",
    "@codemirror/collab",
    "@codemirror/commands",
    "@codemirror/language",
    "@codemirror/lint",
    "@codemirror/search",
    "@codemirror/state",
    "@codemirror/view",
    "@lezer/common",
    "@lezer/highlight",
    "@lezer/lr",
  ],
  plugins: [
    wasm({ maxFileSize: Math.pow(2, 32), publicPath: "./" }),
    nodeResolve(),
  ],
};
