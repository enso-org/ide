#!/bin/bash
cargo watch --watch "lib" --clear -s "script/build.py ${@}"
