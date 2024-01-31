<div id="header" align="center">
    <h1>Harper</h1>
</div>

Harper is a an English grammar checker designed to be _just right._
I created it after years of dealing with the shortcomings of the competition.

Grammarly was too expensive and too overbearing. 
It's suggestions lacked context, and were often just plain _wrong_.
Not to mention: it's a privacy nightmare.
Everything you write with Grammarly is sent to their servers.
Their privacy policy claims they don't sell the data, but that doesn't mean they don't use it to train large language models god knows what else.
Not only that, but the round-trip-time of the network request makes revising your work all the more tedious.

LanguageTool is great, if you have gigabytes of RAM to spare and are willing to download the ~16GB n-gram dataset.
Besides the memory requirements, I found LanguageTool too slow: it would take several seconds to lint even a moderate-size document.

That's why I created Harper: it is the grammar checker that fits my needs.
Not only does it take milliseconds to lint a document, take less than 1/50th of LanguageTool's memory footprint, 
but it is also completely private.

Harper is even small enough to load via [WebAssembly.](https://harper.elijahpotter.dev)

## Installation

Binary releases are coming soon, so if you are looking for a single file download, you'll have to wait.

However, if you happen to have [Rust installed](https://www.rust-lang.org/tools/install), you're in luck!
To install `harper-ls`, the variant of Harper for text editors like Neovim, simply run:

```bash
cargo install harper-ls
```
