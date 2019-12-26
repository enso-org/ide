const CopyWebpackPlugin = require("copy-webpack-plugin");
const HtmlWebpackPlugin = require('html-webpack-plugin')
const path = require('path');

module.exports = {
    entry: "./bootstrap.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "bootstrap.js",
    },
    mode: "development",
    node: {
        fs: 'empty'
    },
    plugins: [
        new CopyWebpackPlugin(['index.html']),
//    new HtmlWebpackPlugin({template: 'index.html'}),
    ],
    devServer: {
        historyApiFallback: {
            index: 'index.html'
        }
    }
};
