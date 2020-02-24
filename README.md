[![License](https://img.shields.io/static/v1?label=License&message=MIT&color=2ec352&labelColor=2c3239)](https://github.com/luna/basegl/blob/master/LICENSE) 
[![Actions Status](https://github.com/luna/basegl/workflows/Build%20%28MacOS%2C%20Linux%2C%20Windows%29/badge.svg)](https://github.com/luna/basegl/actions)
[![Coverage](https://img.shields.io/codecov/c/github/luna/basegl?label=Coverage&labelColor=2c3239)](https://codecov.io/gh/luna/basegl/branch/master) 
![Stability](https://img.shields.io/static/v1?label=Stability&message=Unstable&color=d52229&labelColor=2c3239)

# Enso Studio

Enso Studio is an IDE for hybrid visual and textual functional programming.

Its code uses BaseGL, a blazing fast 2D vector rendering engine with a rich set of
primitives and a GUI component library. It is able to display millions of shapes
60 frames per second in a web browser on a modern laptop hardware. 

This repository is a work in progress. Please refer to BaseGL 1.0
repository for more information: https://github.com/luna/basegl-old.

## Development

### The Rust toolchain 
This project uses several features available only in the nightly Rust toolchain.
To setup the toolchain, please use the [the Rust toolchain
installer](https://rustup.rs/):

```bash
rustup toolchain install nightly-2019-11-04 # Install the nightly channel.
rustup default nightly-2019-11-04           # Set it as the default one.
rustup component add clippy                 # Install the linter.
```

### Building the sources
Please use the `script/build.sh` script to build the project or the
`script/watch.sh` script to run a file watch utility which will build the
project on every source change. The scripts are thin wrappers for
[wasm-pack](https://github.com/rustwasm/wasm-pack) and accept the same [command
line arguments](https://rustwasm.github.io/wasm-pack/book/commands/build.html).
In particular, you can provide them with `--release`, `--dev`, or `--profile`
flags to switch the compilation profile. If not option is provided, the scripts
default to the `--release` profile.

For best experience, it is recommended to use the 
`scripts/watch.sh --dev` in a second shell.

### Web Application
In order to build the IDE web application, follow the steps bellow:

```bash
ide/app$ npm install
ide/app$ npm run web:dev 
```

You can now navigate to http://localhost:8080 and play with it! The example
scenes will be available at http://localhost:8080/?debug.

While Webpack provides handy utilities for development, like live-reloading on
sources change, it also adds some runtime overhead. In order to run the compiled
examples using a lightweight http-server (without live-reloading functionality),
please use the 
```
ide/app$ npm run web:prod
```

**Please remember to disable the cache in your browser during development!**

### Desktop Application
In order to build the IDE desktop application, follow the steps bellow:

```bash
ide/app$ npm install
ide/app$ npm run electron:dev
```

To debug a subsystem, use the `--debug` argument.

```bash
ide/app$ npm run electron:dev -- --debug        # Shows the selection screen.
ide/app$ npm run electron:dev -- --debug=shapes # Runs shapes' subsystem.
```

If you want to distribute the desktop application, run the following steps instead:

```bash
ide/app$ npm install
ide/app$ npm run electron:dist
```

The generated executable will be available at `dist/enso-ide-major.minor.*`. Its extension will
 depend on the host development platform. Keep in mind that it's also possible to pass the
  `--debug`argument to the application. On Linux, for instance, you can run:
  
```bash
ide$ ./dist/enso-ide.0.1.0.AppImage --debug        # Shows the selection screen.
ide$ ./dist/enso-ide.0.1.0.AppImage --debug=shapes # Runs shapes' subsystem.
```

### Minimizing the WASM binary size.
After building the project you can use the `scripts/minimize_wasm.sh` to optimize 
the binary and compress it by using `gzip`. After the script is complete, the
final size is printed to stdout. Please note that in order to run the script, the
[Binaryen](https://github.com/WebAssembly/binaryen) toolkit has to be installed
on your system.

### Running tests
The sources use both unit tests and web test, which are run in a browser and
produce visual results. To run them, use the `scripts/test.sh` script and follow
the output in the terminal.


### Working with the source code

#### Formatting
Please note that this codebase does not use `rustfmt`. Please read the following
documents to learn more about reasons behind this decision and the recommended
code style guide. Be sure to carefully read the documents before contributing to
this repository:
- [Rust style guide 1](https://github.com/luna/basegl/blob/master/docs/style-guide.md)
- [Rust style guide 2](https://github.com/luna/enso/blob/master/doc/rust-style-guide.md) 


#### Linting 
Please be sure to fix all errors reported by `scripts/lint.sh` before creating a
pull request to this repository.
