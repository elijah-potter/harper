# Harper's Architecture

This document seeks to solve one simple problem:

> "Roughly, it takes 2x more time to write a patch if you are unfamiliar with the project, but it takes 10x more time to figure out **where** you should change the code." - [Alex Kladov](https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html)

This document is meant to serve as a kind of table of contents for the Harper project.
Hopefully, we can reduce that 10x down to something a little more reasonable.

## What does Harper do?

Harper tries to do one thing well: find grammatical and spelling errors in English text.
If possible, provide suggestions to correct those errors.
An error and it's possible corrections together form what we call a lint.

In this vein, Harper serves the role of a [Linter](<https://en.wikipedia.org/wiki/Lint_(software)>) for English.

## `harper-core`

`harper-core` is where all the magic happens.
It contains the code need to tokenize, parse, analyze and lint English text.

At a high level, there are just a couple types you need to worry about.

- [Document](https://docs.rs/harper-core/latest/harper_core/struct.Document.html): A representation of an English document. Implements [`TokenStringExt`](https://docs.rs/harper-core/latest/harper_core/trait.TokenStringExt.html) to make it easier to query.
- [Parser](https://docs.rs/harper-core/latest/harper_core/parsers/trait.Parser.html): A trait that describes an object that consumes text and emits tokens. The name is somewhat of a misnomer since it is supposed to only lex English (and emit [Tokens](https://docs.rs/harper-core/latest/harper_core/struct.Token.html)), not parse it. It is called a parser since most types that implement this trait parse _other_ languages (JavaScript) to extract the English text.
  - The [Markdown parser](https://docs.rs/harper-core/latest/harper_core/parsers/struct.Markdown.html) is a great example.
- [Linter](https://docs.rs/harper-core/latest/harper_core/linting/trait.Linter.html): A trait that, provided a document, will produce zero or more [Lints](https://docs.rs/harper-core/latest/harper_core/linting/struct.Lint.html#). This is usually done using direct queries on the document or by implementing a [`PatternLinter`](https://docs.rs/harper-core/latest/harper_core/linting/trait.PatternLinter.html).

If you want to add a linter to Harper, create a new file under the `linters` module in `harper-core` and create a public struct that implements the `Linter` trait.
There are a couple places in other parts of the codebase you'll need to update before it will show up in editors and have persistent settings, but that's a problem for after you've opened your pull request.

## `harper-ls`

`harper-ls` is a language server that wraps around `harper-core`.
In essence, it enables text editors and IDEs to access the capabilities of Harper over a network or via standard input/output.

If you aren't familiar with what a language server does, I would suggest reading [this](https://tamerlan.dev/an-introduction-to-the-language-server-protocol/) or the [official language server protocol documentation](https://microsoft.github.io/language-server-protocol/).

When Harper is used through Neovim, Visual Studio Code, Helix or Emacs, `harper-ls` is the interface.

## `harper-wasm`

`harper-wasm` is a small library that wraps `harper-core` and compiles to WebAssembly with [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen).
This allows web applications (like the [Harper](https://writewithharper.com) and the [Obsidian Plugin](https://github.com/elijah-potter/harper-obsidian-plugin) to run Harper without downloading any additional executables. It all runs inside the JavaScript engine.
