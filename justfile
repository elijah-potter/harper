format:
  cargo +nightly fmt  
  cd {{justfile_directory()}}/packages; yarn prettier -w .

# Build the WebAssembly for a specific target (usually either `web` or `bundler`)
build-wasm target:
  cd {{justfile_directory()}}/harper-wasm && wasm-pack build --target {{target}}

dev-web:
  #! /bin/bash
  set -eo pipefail

  just build-wasm bundler

  cd {{justfile_directory()}}/packages/web
  yarn install -f
  yarn dev

build-web:
  #! /bin/bash
  set -eo pipefail
  just build-wasm bundler
  
  cd {{justfile_directory()}}/packages/web
  yarn install -f
  yarn run check
  yarn run build

build-obsidian:
  #! /bin/bash
  set -eo pipefail
  
  just build-wasm web
  cd {{justfile_directory()}}/packages/obsidian-plugin

  yarn install -f
  yarn build

  zip harper-obsidian-plugin.zip manifest.json main.js

precommit:
  #! /bin/bash
  set -eo pipefail

  cargo +nightly fmt --check
  cargo clippy -- -Dwarnings
  cargo test
  cargo test --release
  cargo doc
  cargo build
  cargo build --release
  cargo bench

  cd {{justfile_directory()}}/packages
  yarn install
  yarn prettier --check .
  yarn eslint .

  just build-obsidian
  just build-web
