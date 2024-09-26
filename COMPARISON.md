# Comparison to Other Grammar Checkers

|              | Suggestion Time | License                  | LSP Support                                                                                          | Ruleset                     | Multi-Lingual/Multi-Dialect |
| ------------ | --------------- | ------------------------ | ---------------------------------------------------------------------------------------------------- | --------------------------- | --------------------------- |
| harper       | 10ms            | Apache-2.0               | ✅                                                                                                   | hunspell/MySpell derivative | ❌                          |
| LanguageTool | 650ms           | LGPL-2.1                 | 🟨 Through [ltex-ls](https://github.com/valentjn/ltex-ls)                                            | LanguageTool(XML)           | 🟨 Not simultaneously       |
| hunspell     |                 | LGPL/GPL/MPL tri-license | ❌                                                                                                   | hunspell/MySpell            | 🟨 Not simultaneously       |
| Grammarly    | 4000ms          | Proprietary              | 🟨 Through [grammarly-language-server](https://github.com/emacs-grammarly/grammarly-language-server) | Proprietary                 | ❌                          |
