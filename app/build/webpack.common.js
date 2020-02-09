const CopyWebpackPlugin = require('copy-webpack-plugin')
const CompressionPlugin = require('compression-webpack-plugin')

const path = require('path');
const root = path.resolve(__dirname, '..')

module.exports = {
    entry: {
        index: './src/index.js',
        wasm_imports: './src/wasm_imports.js',
    },
    output: {
        path: path.resolve(root, 'dist'),
        filename: '[name].js',
        libraryTarget: 'umd',
    },
    node: {
        fs: 'empty'
    },
    plugins: [
        new CompressionPlugin(),
        new CopyWebpackPlugin([path.resolve(root,'src','index.html')]),
    ],
    devServer: {
        historyApiFallback: {
            index: 'index.html'
        }
    },
    resolve: {
        modules: [path.resolve(root, 'node_modules')],
        alias: {
            wasm_rust_glue$: path.resolve(root, 'dist', 'wasm', 'basegl_examples.js')
        }
    },
    performance: {
        hints: false,
    },
};
