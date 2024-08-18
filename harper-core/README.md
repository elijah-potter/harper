# `harper-core`

This is the core engine behind Harper.
Right now, it only exists to hold the core functioning of `harper-ls` and `harper-wasm` (and by extension the web interface).

`harper-core` _is_ [available on `crates.io`](https://crates.io/crates/harper-core), however improving the API is currently not a high priority.
Feel free to use `harper-core` in your projects, but if you run into issues, create a pull request.

## Features

`concurrent`: Whether to use thread-safe primitives (`Arc` vs `Rc`). Disabled by default.
It is not recommended unless you need thread-safely (i.e. you want to use something like `tokio`).
