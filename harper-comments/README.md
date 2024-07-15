# `harper-comments`

This crate holds a number of functions, but it is primarily a wrapper around `tree-sitter` that allows Harper to locate the comments of a wide variety of programming languages.
It also has purpose-built parsers for the structured comments of a number of languages, including Go.
These additional parsers are available through the `CommentParser` and are enabled automatically through there.
