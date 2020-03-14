const CopyWebpackPlugin = require('copy-webpack-plugin')
const CompressionPlugin = require('compression-webpack-plugin')
const path              = require('path')

const thisPath = path.resolve(__dirname)
const root     = path.resolve(thisPath,'..','..','..','..')
const distPath = path.resolve(root,'dist')
const wasmPath = path.resolve(distPath,'wasm')

module.exports = {
    entry: {
        index: path.resolve(thisPath,'src','index.js'),
        wasm_imports: './src/wasm_imports.js',
    },
    output: {
        path: path.resolve(root,'dist','content','assets'),
        filename: '[name].js',
        libraryTarget: 'umd',
    },
    node: {
        fs: 'empty'
    },
    plugins: [
        new CompressionPlugin(),
        new CopyWebpackPlugin([
            path.resolve(thisPath,'src','index.html'),
            path.resolve(wasmPath,'gui.wasm'),
        ]),
    ],
    devServer: {
        publicPath: '/assets/',
        historyApiFallback: {
            index: '/assets/'
        }
    },
    resolve: {
        alias: {
            wasm_rust_glue$: path.resolve(wasmPath,'gui.js')
        }
    },
    performance: {
        hints: false,
    },
    mode: 'none',
    stats: 'minimal'
}
