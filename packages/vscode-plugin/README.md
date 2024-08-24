# Harper for VS Code

Currently, `harper-ls` needs to be in your `PATH` for this extension to work. [Read here to see how to install it](/harper-ls/README.md#installation).

## Manually Packaging and Installing

### Requirements

- [`yarn`](https://classic.yarnpkg.com/en), as the package manager
- [`just`](https://just.systems), as the command runner

### Steps

1. Clone or download the Harper repository:

   ```console
   git clone https://github.com/elijah-potter/harper && cd harper
   ```

2. Package the extension:

   ```console
   just package-vscode
   ```

3. Install the extension:

   ```console
   code --install-extension packages/vscode-plugin/harper-0.0.1.vsix
   ```
