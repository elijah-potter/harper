# `harper-ls`

`harper-ls` is the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) frontend for [Harper](https://writewithharper.com).
Out of the box, it has built-in support for parsing the comments of most programming languages, as well as any and all markdown files.

## Installation

How you choose to install `harper-ls` depends on your use-case.
Right now, we only directly support usage through [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md#harper_ls).
Refer to the linked documentation for more information.

If you happen to use [`mason.nvim`](https://github.com/williamboman/mason.nvim), installation will be pretty straightforward.
`harper-ls` is in the official Mason registry, so you can install it the same way you install anything through Mason.

If you __don't__ install your LSPs through Mason, we also have binary releases available on [GitHub](https://github.com/elijah-potter/harper/releases).

Finally, if you have [Rust installed](https://www.rust-lang.org/tools/install), you're in luck!
To install `harper-ls`, simply run:

```bash
cargo install harper-ls --locked
```

## Dictionaries

`harper-ls` has three kinds of dictionaries: user, file-local, and static dictionaries.

### User Dictionary

Each user of `harper-ls` has their own dictionary, located in the following directories on each operating system:

| Operating System |                                       Location |
| :--------------- | ---------------------------------------------: |
| Linux            |                  `$XDG_CONFIG_HOME/harper-ls/` |
| MacOS            | `$HOME/Library/Application Support/harper-ls/` |
| Windows          |           `{FOLDERID_LocalAppData}/harper-ls/` |

This dictionary is a simple word list in plain-text.
You can add and remove words at will.
You can add to the user dictionary with code actions on misspelled words.

#### Configuration

You don't have to stick with the default locations (listed above).
If you use Neovim, you can set the location of the dictionary with the `userDictPath` key:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      userDictPath = "~/dict.txt"
    }
  },
}
```

You can also toggle any particular linter.
The default values are shown below:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      linters = {
        spell_check = true,
        spelled_numbers = false,
        an_a = true,
        sentence_capitalization = true,
        unclosed_quotes = true,
        wrong_quotes = false,
        long_sentences = true,
        repeated_words = true,
        spaces = true,
        matcher = true
      }
    }
  },
}
```

By default, `harper-ls` will mark all diagnostics with HINT.
If you want to configure this, refer below:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
        diagnosticSeverity = "hint" -- Can also be "information", "warning", or "error"
    }
  },
}
```


### File-Local Dictionary

Sometimes, you'll encounter a word (or name) that is only valid within the context of a specific file.
In this case, you can use the code action that adds the word to the file-local dictionary.
Any words added to this dictionary will, like the name implies, only be included in the dictionary when performing corrections on the file at that specific path.

You can find the file-local dictionaries in the following directories on each operation system:

| Operating System |                                                                                         Location |
| :--------------- | -----------------------------------------------------------------------------------------------: |
| Linux            | `$XDG_DATA_HOME/harper-ls/file_dictionaries` or `$HOME/.local/share/harper-ls/file_dictionaries` |
| MacOS            |                                  `$HOME/Library/Application Support/harper-ls/file_dictionaries` |
| Windows          |                                            `{FOLDERID_LocalAppData}/harper-ls/file_dictionaries` |

The format of these files is identical to user dictionaries.

### Static Dictionary

The static dictionary is built into the binary and is (as of now) immutable.
It contains almost all words you could possibly encounter.

I _do_ take pull requests or issues for adding words to the static dictionary.
It is composed of two files: `harper-core/dictionary.dict` and `harper-core/dictionary.dict`
