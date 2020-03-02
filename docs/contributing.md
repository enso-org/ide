# Development and Contributing


## Development

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

<br/>

### Building Rust Sources
Run `script/watch.py` to start a source-file watch utility which will build the
project on every change. By default, the `watch` script disables some optimizations to provide interactive development experience. In order to build the project in a release mode, use `script/build.py` instead. The scripts 
are thin wrappers for [wasm-pack](https://github.com/rustwasm/wasm-pack) and accept the same [command
line arguments](https://rustwasm.github.io/wasm-pack/book/commands/build.html).

<br/>

### Minimizing the WASM binary size (optional)
After building the project you can use the `scripts/minimize_wasm.py` to optimize 
the binary and compress it by using `gzip`. After the script is complete, the
final size is printed to stdout.

<br/>

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
