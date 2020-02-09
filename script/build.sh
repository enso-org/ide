#!/bin/bash
wasm-pack build --target web --no-typescript --out-dir '../../target/web' $@ lib/examples
rm -Rf app/dist/wasm
cp -R target/web app/dist/wasm
