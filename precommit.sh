#! /bin/bash

# Run the tools necessary to make sure the code is ready for commit.

set -eo pipefail

R=$(pwd)

cargo +nightly fmt
cargo clippy -- -Dwarnings
cargo test
cargo test --release
cargo doc
cargo build
cargo build --release

cd $R/harper-wasm
wasm-pack build

cd $R/web
yarn install
yarn run format
yarn run lint
yarn run check
yarn run build
