# Harper for VS Code

Harper is the grammar checker for developers. It checks for spelling and grammar errors in your Markdown files and code comments. You can find out more by checking it out on [GitHub](https://github.com/elijah-potter/harper) or by visiting the [website](https://writewithharper.com).

## Installation

Installation should be relatively straightforward.
It just depends on which editor and marketplace you're using.

If you use the official Microsoft Visual Studio Code release, go ahead and go to the marketplace and search for "Harper" and click "Install".
You can also visit our [official page](https://marketplace.visualstudio.com/items?itemName=elijah-potter.harper&ssr=false#overview).

If you use OpenVSX, for instance if you use VSCodium, you'll want to install from [here](https://open-vsx.org/extension/elijah-potter/harper).

### Commands

| Command                         | Id                              | Description         |
| ------------------------------- | ------------------------------- | ------------------- |
| Harper: Restart Language Server | `harper.languageserver.restart` | Restart `harper-ls` |

### Settings

| Setting                        | Possible Values                                   | Default Value   | Description                                                       |
| ------------------------------ | ------------------------------------------------- | --------------- | ----------------------------------------------------------------- |
| `harper-ls.linters.*`          | `true`, `false`                                   | Varies          | Detect and provide suggestions in a variety of common situations. |
| `harper-ls.diagnosticSeverity` | `"error"`, `"hint"`, `"information"`, `"warning"` | `"information"` | How severe do you want diagnostics to appear in the editor?       |

## Developing and Contributing

See the [Development Guide](/packages/vscode-plugin/development-guide.md).
