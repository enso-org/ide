#!/bin/bash
<<<<<<< HEAD
cargo watch --watch "lib" --clear -s "script/build.sh ${@}"
=======
cargo watch -i .gitignore -i "pkg/*" -s "script/build.sh ${@}"
>>>>>>> master
