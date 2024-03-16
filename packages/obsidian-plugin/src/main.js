import { linter } from "@codemirror/lint";
import { Plugin } from "obsidian";
import { EditorView } from "@codemirror/view";
import wasm from "wasm/harper_wasm_bg.wasm";
import init, { lint, use_spell_check } from "wasm";

function contentToString(content) {
  return content.reduce((p, c) => p + c, "");
}

function suggestionToLabel(sug) {
  if (sug === "Remove") {
    return "Remove";
  } else {
    return `Replace with "${contentToString(sug.ReplaceWith)}"`;
  }
}

const harperLinter = linter(
  async (view) => {
    let text = view.state.doc.sliceString(-1);

    await init(await wasm());
    use_spell_check(false);
    let lints = lint(text);

    return lints.map((lint) => {
      return {
        from: lint.span.start,
        to: lint.span.end,
        severity: "error",
        message: lint.message,
        actions: lint.suggestions.map((sug, i) => {
          return {
            name: suggestionToLabel(sug),
            apply: (view) => {
              if (sug === "Remove") {
                view.dispatch({
                  changes: {
                    from: lint.span.start,
                    to: lint.span.end,
                    insert: "",
                  },
                });
              } else {
                view.dispatch({
                  changes: {
                    from: lint.span.start,
                    to: lint.span.end,
                    insert: contentToString(sug.ReplaceWith),
                  },
                });
              }
            },
          };
        }),
      };
    });
  },
  { delay: -1 },
);

const theme = EditorView.baseTheme({
  ".cm-diagnostic": {
    padding: "4px !important",
    marginLeft: "0px !important",
    display: "block",
    whiteSpace: "pre-wrap",
  },
  ".cm-diagnostic-error": { borderLeft: "none !important" },
  ".cm-diagnostic-warning": { borderLeft: "none !important" },
  ".cm-diagnostic-info": { borderLeft: "none !important" },
  ".cm-diagnostic-hint": { borderLeft: "none !important" },

  // The buttons
  ".cm-diagnosticAction": {
    margin: "0px !important",
    display: "flex !important",
    alignItems: "center !important",
    gap: "var(--size-4-2) !important",
    padding: "var(--size-4-1) var(--size-4-2) !important",
    cursor: "var(--cursor) !important",
    fontSize: "var(--font-ui-small) !important",
    borderRadius: "var(--radius-s) !important",
    whiteSpace: "nowrap !important",
    backgroundColor: "white !important",
    color: "black !important",
    width: "100%",
  },

  ".cm-diagnosticSource": {
    fontSize: "70%",
    opacity: 0.7,
  },

  ".cm-tooltip-lint": {
    padding: 0,
    margin: 0,
  },

  ".cm-tooltip": {
    padding: "var(--size-2-3) !important",
    border: "1px solid var(--background-modifier-border-hover) !important",
    backgroundColor: "var(--background-secondary) !important",
    borderRadius: "var(--radius-m) !important",
    boxShadow: "var(--shadow-s) !important",
    zIndex: "var(--layer-menu) !important",
    userSelect: "none !important",
    maxHeight: "calc(100% - var(--header-height)) !important",
    overflow: "hidden !important",
  },

  ".cm-lintPoint": {
    position: "relative",

    "&:after": {
      content: '""',
      position: "absolute",
      bottom: 0,
      left: "-2px",
      borderLeft: "3px solid transparent",
      borderRight: "3px solid transparent",
      borderBottom: "4px solid #d11",
    },
  },

  ".cm-lintPoint-warning": {
    "&:after": { borderBottomColor: "orange" },
  },
  ".cm-lintPoint-info": {
    "&:after": { borderBottomColor: "#999" },
  },
  ".cm-lintPoint-hint": {
    "&:after": { borderBottomColor: "#66d" },
  },
});

export default class HarperPlugin extends Plugin {
  async onload() {
    this.registerEditorExtension([harperLinter, theme]);
  }
}
