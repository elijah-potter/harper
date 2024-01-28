# `harper-serve`

This crates exists as a way to debug both the web interface and the underlying engine at the same time.
To set the web interface to use a server as the backend, edit the first line of `analysis.ts`.

When I'm debugging, I then start the `harper-serve` binary with [`bacon`](https://github.com/Canop/bacon).
