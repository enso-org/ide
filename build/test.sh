#!/bin/bash
cargo test && cargo run --manifest-path=build/rust/Cargo.toml --bin test-all -- --headless --chrome
