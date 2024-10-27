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

## Language Support

Harper currently only supports American English, but the core is extensible to support other languages, so we welcome contributions that allow for other language support.

## Installation

If you want to use Harper on your machine, you have three choices.

### `harper-ls`

`harper-ls` provides an integration that works for most code editors.

[Read more here.](./harper-ls/README.md)

### Harper Obsidian Integration

If you use [Obsidian](https://obsidian.md/), you may install the [Harper Obsidian Plugin](https://github.com/elijah-potter/harper-obsidian-plugin) by searching for "Harper" in the community plugin store.

### Zed Plugin

If you use [Zed](https://zed.dev/), [Stef16Robbe](https://github.com/Stef16Robbe) has developed a fantastic [plugin](https://github.com/Stef16Robbe/harper_zed) that works out-of-the box.
No setup required.

## Performance Issues

We consider long lint times bugs.
If you encounter any significant performance issues, please create an issue on the topic.

If you find a fix to any performance issue, we are open the contribution.
Just make sure to read [our contribution guidelines first.](https://github.com/elijah-potter/harper/blob/master/CONTRIBUTING.md)

## FAQs

### Where did the name Harper come from?

See [this blog post](https://elijahpotter.dev/articles/naming_harper).

## Huge Thanks

This project would not be possible without the hard work from those who [contribute](/CONTRIBUTING.md).

<a href="https://github.com/elijah-potter/harper/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=elijah-potter/harper" />
</a>
