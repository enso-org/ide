#!/bin/bash

echoerr() { echo "$@" 1>&2; }

actual_size=$(./script/minimize_wasm.sh)
max_size=1.0M
if (( $(echo "$actual_size <= $max_size" |bc -l) ));
then
  echo OK
else
  echoerr "FAIL. Expected maximum size: ${max_size} current size: ${actual_size}."
  exit 1
fi