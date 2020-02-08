const CopyWebpackPlugin = require("copy-webpack-plugin")
const path = require('path');

const mb = 1024 * 1024;

module.exports = {
    entry: "./index.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "index.js",
    },
    node: {
        fs: 'empty'
    },
    plugins: [
        new CopyWebpackPlugin(['index.html']),
    ],
    devServer: {
        historyApiFallback: {
            index: 'index.html'
        }
    },
    resolve: {
        modules: [path.resolve(__dirname, "node_modules")]
    },
    performance: {
        hints: false,
//        maxAssetSize: 5.0 * mb,
    },

    devServer: {
        mimeTypes: { typeMap:{'application/wasm': ['wasm']}, force:true },
    },
};
