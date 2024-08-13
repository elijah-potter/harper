# Format both Rust and JavaScript
format:
  cargo +nightly fmt  
  cd {{justfile_directory()}}/packages; yarn prettier -w .

# Build the WebAssembly for a specific target (usually either `web` or `bundler`)
build-wasm target:
  cd {{justfile_directory()}}/harper-wasm && wasm-pack build --target {{target}}

# Compile the web demo's dependencies and start a development server. Note that if you make changes to `harper-wasm`, you will have to re-run this command.
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
  yarn run build

build-obsidian:
  #! /bin/bash
  set -eo pipefail
  
  just build-wasm web
  cd {{justfile_directory()}}/packages/obsidian-plugin

  yarn install -f
  yarn build

  zip harper-obsidian-plugin.zip manifest.json main.js

build-vscode:
  #! /bin/bash
  set -eo pipefail

  cd {{justfile_directory()}}/packages/vscode-plugin
  yarn install -f 
  yarn compile

check:
  #! /bin/bash
  set -eo pipefail

  cargo +nightly fmt --check
  cargo clippy -- -Dwarnings

  cd {{justfile_directory()}}/packages
  yarn install
  yarn prettier --check .
  yarn eslint .

  cd web
  just build-web
  yarn run check

precommit:
  #! /bin/bash
  set -eo pipefail

  just check
  just test

  cargo doc
  cargo build
  cargo build --release
  cargo bench

  just build-obsidian
  just build-vscode
  just build-web

install:
  cargo install --path harper-ls
  cargo install --path harper-cli

# Run `harper-cli` on the Harper repository
dogfood:
  #! /bin/bash
  for file in `fd -e rs`
  do
    echo Linting $file
    cargo run --bin harper-cli --quiet -- lint $file
  done

test:
  cargo test
  cargo test --release

parse file:
  cargo run --bin harper-cli -- parse {{file}}

lint file:
  cargo run --bin harper-cli -- lint {{file}}
