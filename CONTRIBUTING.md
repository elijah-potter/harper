# Contributing

Harper is completely open to outside contributors of any kind.

If you have a feature request or bug to report, please [create an issue](https://github.com/elijah-potter/harper/issues).

## Setup Your Environment

To use the tooling required to build and debug Harper, you'll need to the following programs available in your `PATH`.

- [`just`](https://github.com/casey/just)
- `bash`
- [`cargo`](https://www.rust-lang.org/) (we develop against the latest version of Rust)
- `yarn`
- `node`
- `grep`
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)

To list out the various tools for development, run:

```bash
just --list
```

I would highly recommend that you run `just setup` to populate your build caches and install the necessary build-tools.

## Committing

Harper follows [conventional commit practices](https://www.conventionalcommits.org/en/v1.0.0/).
Before creating a pull request, please make sure all your commits follow the linked conventions.

Additionally, to minimize the labor required to review your commit, we run a relatively strict suite of formatting and linting programs.
We highly recommend that you run both `just format` and `just precommit` before submitting a pull request.
If those scripts don't work in your environment, we run `just precommit` through GitHub actions inside of pull requests, so you may make modifications and push until the checks pass.

If this sounds intimidating, don't worry.
We are entirely willing to work with you to make sure your code can make it into Harper, just know it might take a little longer.

## How does it work?

Please take a read of the [architecture document](./ARCHITECTURE.md).
