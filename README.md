<h1 align="center">
  <br>
  <a href="http://luna-lang.org"><img src="https://user-images.githubusercontent.com/1623053/75657359-50c92300-5c66-11ea-9cb8-61da8ee34df1.png" alt="Enso" width="128"></a>
  <br>
  Enso Studio
  <br>
</h1>

<h4 align="center">A Visual Programming language of the future.</h4>


<p align="center">
  <a href="https://github.com/luna/basegl/blob/master/LICENSE">
    <img src="https://img.shields.io/static/v1?label=License&message=MIT&color=2ec352&labelColor=2c3239"
         alt="License">
  </a>
  <a href="https://github.com/luna/basegl/actions">
    <img src="https://github.com/luna/basegl/workflows/Build%20%28MacOS%2C%20Linux%2C%20Windows%29/badge.svg"
         alt="Actions Status">
  </a>
  <a href="https://codecov.io/gh/luna/basegl/branch/master">
    <img src="https://img.shields.io/codecov/c/github/luna/basegl?label=Coverage&labelColor=2c3239"
         alt="Coverage">
  </a>
  <a>
    <img src="https://img.shields.io/static/v1?label=Stability&message=Unstable&color=d52229&labelColor=2c3239"
         alt="Stability">
  </a>
</p>

<p align="center">
  <a href="#key-features">Key Features</a> •
  <a href="#how-to-use">How To Use</a> •
  <a href="#download">Download</a> •
  <a href="#credits">Credits</a> •
  <a href="#related">Related</a> •
  <a href="#license">License</a>
</p>

## Key Features

- Uses EnsoGL to display millions of shapes 60 frames per second in a web browser.



## Building From Sources

### Development Environment

- **The Rust Toolchain**  
  This project uses several features available only in the nightly Rust toolchain.
Please use the [the Rust toolchain installer](https://rustup.rs) to install it:

  ```bash
  rustup toolchain install nightly-2019-11-04 # Install the nightly channel.
  rustup default nightly                      # Set it as the default one.
  rustup component add clippy                 # Install the linter.
  ```

- **Node Version Manager**  
  In order to build the web and desktop applications you will need [node](https://nodejs.org) and 
[npm](https://www.npmjs.com). Please note that the versions available in your system package manager
will probably crash while building native-extensions. The only known stable solution is to use the 
[Node Version Manager](https://github.com/nvm-sh/nvm). Please note that installing it from any 
package manager is officially not supported and can cause issues. Follow the 
[official guide](https://github.com/nvm-sh/nvm#installing-and-updating) to install it:

  ```bash
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.2/install.sh | bash
  ```

### Building Rust Sources
Run `script/watch.py --dev` to start a file-watch utility which will build the
project on every source change. The usage of `--dev` shortens the build time drastically.
In order to build the project in a release mode, use `script/build.py` instead. The scripts 
are thin wrappers for
[wasm-pack](https://github.com/rustwasm/wasm-pack) and accept the same [command
line arguments](https://rustwasm.github.io/wasm-pack/book/commands/build.html).


### Minimizing the WASM binary size (optional)
After building the project you can use the `scripts/minimize_wasm.py` to optimize 
the binary and compress it by using `gzip`. After the script is complete, the
final size is printed to stdout.

### Building the Web Application
Enter the `app` directory and follow the steps:
```bash
nvm use # Sets the correct node / npm versions and updates them if needed.
```


### Running application and examples
For best experience, it is recommended to use the 
`scripts/watch.sh --dev` in a second shell. In order to build the IDE application, 
follow the steps below:

```bash
cd app
npm install
npm run start 
```

You can now navigate to http://localhost:8080 and play with it! The example
scenes will be available at http://localhost:8080/debug.

While Webpack provides handy utilities for development, like live-reloading on
sources change, it also adds some runtime overhead. In order to run the compiled
examples using a lightweight http-server (without live-reloading functionality),
please use the `npm run prod-server` command.

**Please remember to disable the cache in your browser during development!**



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
