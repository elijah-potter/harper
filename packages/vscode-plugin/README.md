# Harper for VS Code

Read [main readme](/README.md) or visit [website](https://writewithharper.com).

## Installation

## Features

### Commands

| Command                         | Id                              | Description         |
| ------------------------------- | ------------------------------- | ------------------- |
| Harper: Restart Language Server | `harper.languageserver.restart` | Restart `harper-ls` |

### Settings

| Setting                                          | Possible Values                                   | Default Value   | Description                                                                |
| ------------------------------------------------ | ------------------------------------------------- | --------------- | -------------------------------------------------------------------------- |
| `harper-ls.linters.spell_check`                  | `true`, `false`                                   | `true`          | Detect and provide suggestions for misspelled words.                       |
| `harper-ls.linters.spelled_numbers`              | `true`, `false`                                   | `false`         | Detect and fix instances where small numbers should be spelled out.        |
| `harper-ls.linters.an_a`                         | `true`, `false`                                   | `true`          | Detect and fix improper articles.                                          |
| `harper-ls.linters.sentence_capitalization`      | `true`, `false`                                   | `true`          | Ensure your sentences are capitalized.                                     |
| `harper-ls.linters.unclosed_quotes`              | `true`, `false`                                   | `true`          | Make sure you close your quotation marks.                                  |
| `harper-ls.linters.wrong_quotes`                 | `true`, `false`                                   | `false`         | Make sure you use the correct unicode characters for your quotation marks. |
| `harper-ls.linters.long_sentences`               | `true`, `false`                                   | `true`          | Warn about run-on sentences.                                               |
| `harper-ls.linters.repeated_words`               | `true`, `false`                                   | `true`          | Detect and fix commonly repeated words.                                    |
| `harper-ls.linters.spaces`                       | `true`, `false`                                   | `true`          | Detect improper spacing between words.                                     |
| `harper-ls.linters.matcher`                      | `true`, `false`                                   | `true`          | A collection of hand-crafted common grammar mistakes.                      |
| `harper-ls.linters.correct_number_suffix`        | `true`, `false`                                   | `true`          | Make sure you provide the correct suffix for numbers.                      |
| `harper-ls.linters.number_suffix_capitalization` | `true`, `false`                                   | `true`          | Make sure you correctly capitalize your number suffixes.                   |
| `harper-ls.linters.multiple_sequential_pronouns` | `true`, `false`                                   | `true`          | Detect improper sequences of pronouns.                                     |
| `harper-ls.linters.linking_verbs`                | `true`, `false`                                   | `true`          | Detect improper use of linking verbs.                                      |
| `harper-ls.linters.avoid_curses`                 | `true`, `false`                                   | `true`          | Catch use of curse/swear words.                                            |
| `harper-ls.diagnosticSeverity`                   | `"error"`, `"hint"`, `"information"`, `"warning"` | `"information"` | How severe do you want diagnostics to appear in the editor?                |

## Developing and Contributing

See the [Development Guide](/packages/vscode-plugin/development-guide.md).
