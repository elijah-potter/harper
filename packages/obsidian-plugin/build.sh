#! /bin/bash

set -eo pipefail

R=`pwd`
cd ../../harper-wasm
wasm-pack build --target web
cd $R

yarn install
yarn build

zip harper-obsidian-plugin.zip manifest.json main.js
