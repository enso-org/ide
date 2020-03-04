# Contributing Guide


## Development Environment
The project builds on MacOS, Windows, and Linux. Please note that cross-platform builds work on all
of these platforms, however, MacOS packages built on Windows and on Linux will not have a proper 
icon, as generation of MacOS icons is a non-trivial task on these platforms. In order to develop the
source code you will need the following setup:

- **The Rust Toolchain (nightly-2019-11-04)**  
  This project uses several features available only in the nightly Rust toolchain.
  Please use the [the Rust toolchain installer](https://rustup.rs) to install it:

  ```bash
  rustup toolchain install nightly-2019-11-04 # Install the nightly channel.
  rustup default nightly                      # Set it as the default one.
  rustup component add clippy                 # Install the linter.
  ```

- **Node (12.16.1) and Node Package Manager (6.13.4)**  
  In order to build the web and desktop applications you will need 
  [node and npm](https://nodejs.org/en/download/). Even minor release changes are known to cause 
  serious issues, thus **we only support the Latest LTS Version of node and npm. Please do not 
  report build issues if you use other versions.** If you run MacOS or Linux the easiest way to 
  setup the proper version is by installing [Node Version Manager](https://github.com/nvm-sh/nvm) 
  and running `nvm use` in the root of this codebase.

<br/>
<br/>
<br/>

## Working with sources

### Code Style Guide
Please note that you should not use a code auto-formatter in this codebase. Please read the following
documents to learn more about reasons behind this decision and the recommended code style guide. 
Be sure to carefully read the documents before contributing to this repository:
- [Rust style guide 1](https://github.com/luna/basegl/blob/master/docs/style-guide.md)
- [Rust style guide 2](https://github.com/luna/enso/blob/master/doc/rust-style-guide.md) 


### Development
As this is a multi-part project with many complex dependencies, it was equipped with a build script
which both validates your working environment as well as takes care of providing most suitable 
compilation flags for a particular development stage. In order to learn more about the commands and 
available options, simply run `./run` (or `node run` if you are using Windows) and read the manual. 
All arguments after `--` will be passed to sub-commands. For example `./run build -- --dev` will
pass the `--dev` flag to `cargo` (Rust build tool). The most common options are presented below:

- **Interactive mode**  
  Run `./run watch` to start a local web-server and a source-file watch utility which will build the 
  project on every change. Open `http://localhost:8080` (the port may vary and will be reported in
  the terminal if `8080` was already in use) to run the application, or `http://localhost:8080/debug`
  to open example demo scenes. Please remember to disable the cache in your browser during the 
  development! By default, the script disables heavyweight optimizations to provide interactive 
  development experience. The scripts are thin wrappers for 
  [wasm-pack](https://github.com/rustwasm/wasm-pack) and accept the same 
  [command line arguments](https://rustwasm.github.io/wasm-pack/book/commands/build.html).

- **Production mode**  
  In order to compile in a production mode (enable all optimizations, strip WASM debug symbols, 
  minimize the output binaries, etc.), run `./run build`. To create platform-specific packages and
  installers use `./run dist` instead.


## Testing, Linting, and Validation
After changing the code it's always a good idea to lint and test the code. We have prepared several 
scripts which maximally automate the process:

- **Size Validation**
  Use `run check-size` to check if the size of the final binary did not grew too much in comparison
  to the previous release. Watching the resulting binary size is one of the most important 
  responsibility of each contributor in order to keep the project small and suitable for web-based
  usage.
  
- **Testing**
  Use `run test` run both unit and web-based test.
  
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



