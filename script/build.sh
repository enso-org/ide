#!/bin/bash
wasm-pack build --target web --no-typescript --out-dir '../../target/web' $@ lib/gui
mv target/web/gui_bg.wasm target/web/gui.wasm

# https://github.com/rustwasm/wasm-pack/issues/790
perl -i -p0e 's/if \(\(typeof URL.*}\);/return imports/gs' target/web/gui.js
perl -i -p0e 's/if \(typeof module.*let result/let result/gs' target/web/gui.js

gzip --keep --best --force target/web/gui.wasm
rm -Rf app/src-rust-gen
cp -R target/web app/src-rust-gen
