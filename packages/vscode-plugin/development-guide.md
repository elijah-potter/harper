# Development Guide

This document details how to develop the extension locally. If you're interested in how it's packaged for distribution, you can checkout the [Package VS Code Plugin](/.github/workflows/package_vscode_plugin.yml) workflow.

## Notes

- The extension code and its tests live in the `src` directory. Most changes you'll need to make will be there.
- VS Code can only pickup the tasks and launch configurations set in `packages/vscode-plugin/.vscode` if this directory, `packages/vscode-plugin`, not the root of the Harper repository, is open.
- You can look at the project's [`justfile`](/justfile) to see exactly what running the `just` recipes below do.

## Prerequisites

- Make sure to read the [Contributing guide](/CONTRIBUTING.md) and follow the "Setup Your Environment" section.
- Before running or testing the extension using VS Code's Debugger, make sure you have `harper-ls` in `packages/vscode-plugin/bin`. You can either manually create the directory, compile `harper-ls`, and put it there or you can run `just test-vscode` or `just package-vscode` which will do that for you.

## Running the Extension

1. Open the Run and Debug view by selecting it from the Activity Bar or by pressing `Ctrl+Shift+D`.
2. Choose `Run Extension`, if not chosen already.
3. Click the play (Start Debugging) button or press `F5`.

## Running the Tests

### Using VS Code's Debugger

1. Open the Run and Debug view by selecting it from the Activity Bar or by pressing `Ctrl+Shift+D`.
2. Choose `Test Extension`, if not chosen already.
3. Click the play (Start Debugging) button or press `F5`.

### Using the Command Line

```console
just test-vscode
```

## Packaging and Installing the Extension

1. Package the extension:

   ```console
   just package-vscode
   ```

2. Install the extension:

   ```console
   code --install-extension path/to/created/.vsix
   ```
