#! /bin/bash

set -eo pipefail

R=`pwd`

cd $R/harper-wasm
wasm-pack build --release

cd $R/web
yarn install
yarn build
