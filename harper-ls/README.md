# `harper-ls`

`harper-ls` is the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) frontend for [Harper](https://harper.elijahpotter.dev).
Out of the box, it has built-in support for parsing the comments of most programming languages, as well as any and all markdown files.

## Installation

Binary releases are coming soon, so if you are looking for a single file download, you'll have to wait.

However, if you happen to have [Rust installed](https://www.rust-lang.org/tools/install), you're in luck!
To install `harper-ls`, the variant of Harper for text editors like Neovim, simply run:

```bash
cargo install harper-ls
```

### Neovim Setup

Right now we only support using `nvim-lspconfig` to run Harper in Neovim.
Refer to [the documentation](https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md#harper_ls) for more information.
