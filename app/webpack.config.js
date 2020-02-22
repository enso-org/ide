const CopyWebpackPlugin = require('copy-webpack-plugin')
const CompressionPlugin = require('compression-webpack-plugin')

const path = require('path');
const root = path.resolve(__dirname)

module.exports = {
    entry: {
        index: './src/common/index.js',
        wasm_imports: './src/common/wasm_imports.js',
    },
    output: {
        path: path.resolve(root,'dist','src/common/'),
        filename: '[name].js',
        libraryTarget: 'umd',
    },
    node: {
        fs: 'empty'
    },
    plugins: [
        new CompressionPlugin(),
        new CopyWebpackPlugin([
            path.resolve(root,'src','dev/index.html'),
            path.resolve(root,'src-rust-gen','gui.wasm'),
            path.resolve(root,'src-rust-gen','gui.wasm.gz'),
        ]),
    ],
    devServer: {
        publicPath: 'src/common/',
        historyApiFallback: {
            index: 'src/common/'
        }
    },
    resolve: {
        modules: [path.resolve(root,'node_modules')],
        alias: {
            wasm_rust_glue$: path.resolve(root,'src-rust-gen','gui.js')
        }
    },
    performance: {
        hints: false,
    },
};
