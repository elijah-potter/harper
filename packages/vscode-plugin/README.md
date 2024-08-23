# Harper for VS Code

Currently, `harper-ls` needs to be in your `PATH` for this extension to work. [Read here to see how to install it](/harper-ls/README.md#installation).

## Packaging and Installing

### Requirements

- [`yarn`](https://yarnpkg.com)
- [`vsce`](https://github.com/microsoft/vscode-vsce)

### Steps

- Clone or download the Harper repository:

  ```console
  git clone https://github.com/elijah-potter/harper
  ```

- Navigate to `packages/vscode-plugin`.
- Install dependencies:

  ```console
  yarn install
  ```

- Package the extension:

  ```console
  vsce package
  ```

- Install the extension:

  ```console
  code --install-extension harper-0.0.1.vsix
  ```
