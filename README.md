<div id="header" align="center">
    <img src="logo.svg" width="400px" />
    <h1>Harper</h1>
</div>

[![Harper LS](https://github.com/elijah-potter/harper/actions/workflows/build_harper_ls.yml/badge.svg)](https://github.com/elijah-potter/harper/actions/workflows/build_harper_ls.yml)
[![Web](https://github.com/elijah-potter/harper/actions/workflows/build_web.yml/badge.svg)](https://github.com/elijah-potter/harper/actions/workflows/build_web.yml)
[![Precommit](https://github.com/elijah-potter/harper/actions/workflows/precommit.yml/badge.svg)](https://github.com/elijah-potter/harper/actions/workflows/precommit.yml)
[![Crates.io](https://img.shields.io/crates/v/harper-ls)](https://crates.io/crates/harper-ls)

Harper is an English grammar checker designed to be _just right._
I created it after years of dealing with the shortcomings of the competition.

Grammarly was too expensive and too overbearing. 
Its suggestions lacked context, and were often just plain _wrong_.
Not to mention: it's a privacy nightmare.
Everything you write with Grammarly is sent to their servers.
Their privacy policy claims they don't sell the data, but that doesn't mean they don't use it to train large language models and god knows what else.
Not only that, but the round-trip-time of the network request makes revising your work all the more tedious.

LanguageTool is great, if you have gigabytes of RAM to spare and are willing to download the ~16GB n-gram dataset.
Besides the memory requirements, I found LanguageTool too slow: it would take several seconds to lint even a moderate-size document.

That's why I created Harper: it is the grammar checker that fits my needs.
Not only does it take milliseconds to lint a document, take less than 1/50th of LanguageTool's memory footprint, 
but it is also completely private.

Harper is even small enough to load via [WebAssembly.](https://writewithharper.com)

## Installation

If you want to use Harper on your machine, you will want to look at the [documentation for
`harper-ls`](./harper-ls/README.md), the Language Server Protocol implementation.

## Performance Issues

We consider long lint times bugs.
If you encounter any significant performance issues, please create an issue on the topic.
