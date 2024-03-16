#! /bin/bash

# Run the tools necessary to make sure the code is ready for commit.

set -eo pipefail

R=$(pwd)
RUSTDOCFLAGS="-D warnings"

cargo +nightly fmt --check
cargo clippy -- -Dwarnings
cargo test
cargo test --release
cargo doc
cargo build
cargo build --release
cargo bench

cd $R/harper-wasm
wasm-pack build --target bundler

cd $R/packages
yarn install -f
yarn prettier --check .
yarn eslint .

cd $R/packages/web
yarn install -f
yarn run check
yarn run build

cd $R/harper-wasm
wasm-pack build --target web

cd $R/packages/obsidian-plugin
yarn install -f
yarn run build
