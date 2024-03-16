#! /bin/bash

set -eo pipefail

cargo +nightly fmt

R=$(pwd)

cd $R/packages
yarn prettier -w .
