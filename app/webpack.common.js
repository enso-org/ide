const CopyWebpackPlugin = require('copy-webpack-plugin')
const CompressionPlugin = require('compression-webpack-plugin')

const path = require('path');
const mb = 1024 * 1024;

module.exports = {
    entry: {
        index: './index.js',
        wasm_imports: './wasm_imports.js',
    },
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: '[name].js',
        libraryTarget: 'umd',
    },
    node: {
        fs: 'empty'
    },
    plugins: [
        new CompressionPlugin(),
        new CopyWebpackPlugin(['index.html']),
    ],
    devServer: {
        historyApiFallback: {
            index: 'index.html'
        }
    },
    resolve: {
        modules: [path.resolve(__dirname, 'node_modules')]
    },
    performance: {
        hints: false,
//        maxAssetSize: 5.0 * mb,
    },

    devServer: {
//        compress: true
    },
};
