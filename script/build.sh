#!/bin/bash
wasm-pack build --target web --no-typescript --out-dir '../../target/web' $@ lib/examples
rm -Rf examples/dist/wasm
cp -R target/web examples/dist/wasm
