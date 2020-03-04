# Development and Contributing


## Development

### Code Style Guide
Please note that this codebase does not use autoformatters. Please read the following
documents to learn more about reasons behind this decision and the recommended
code style guide. Be sure to carefully read the documents before contributing to
this repository:
- [Rust style guide 1](https://github.com/luna/basegl/blob/master/docs/style-guide.md)
- [Rust style guide 2](https://github.com/luna/enso/blob/master/doc/rust-style-guide.md) 

<br/>

### Setup
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

### Working with the sources
Run `./run watch` to start a source-file watch utility which will build the project on every change. 
By default, the script disables heavyweight optimizations to provide interactive development 
experience. In order to build the project in a release mode, use `./run build` instead. The scripts 
are thin wrappers for [wasm-pack](https://github.com/rustwasm/wasm-pack) and accept the same 
[command line arguments](https://rustwasm.github.io/wasm-pack/book/commands/build.html).

<br/>

### Building for production
In order to enable all optimizations, remove minimize the resulting 
After building the project you can use the `scripts/minimize_wasm.py` to optimize 
the binary and compress it by using `gzip`. After the script is complete, the
final size is printed to stdout.

<br/>

### Building the Web Application
Enter the `app` directory, run `nvm use` to set up the correct node environment, `npm install` to install dependencies, and `npm run web:dev` to start a file-watch server with a hot-reloading utility. Open `http://localhost:8080` to run the application, or `http://localhost:8080/debug` to see example demo scenes. Please remember to disable the cache in your browser during development!

<br/>

### Testing
The sources use both unit tests and web test, which are run in a browser and
produce visual results. To run them, use the `scripts/test.sh` script and follow
the output in the terminal.

<br/>

### Linting 
Please be sure to fix all errors reported by `scripts/lint.sh` before creating a
pull request to this repository.

<br/>
<br/>

## Distribution

### Building the Native Application
Enter the `app` directory, run `nvm use` to set up the correct node environment and run `npm run electron:dev` to build and start a native application in a development mode. 

<br/>

### Packaging the Native Application
Enter the `app` directory, run `nvm use` to set up the correct node environment and run `npm run electron:dist` to create packages for your current platform. You can also run `npm run electron:dist:all` to create package for all supported platforms, however, such multi-platform builds are currently supported only on MacOS. This sitiuation is unlikely to change, as it is very hard to generate icons for MacOS on Windows and Linux.

<br/>
<br/>

## Running

### Running the Native Application
The standalone application provides a rich set of command line switches. Provide it with `--help` to learn more. You can even use it as a standalone app server by running `Enso Studio --no-window`.


### Running the Standalone Server
To be described.



