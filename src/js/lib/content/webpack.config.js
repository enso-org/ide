const CopyWebpackPlugin = require('copy-webpack-plugin')
const CompressionPlugin = require('compression-webpack-plugin')
const path              = require('path')
const webpack           = require('webpack')

const thisPath = path.resolve(__dirname)
const root     = path.resolve(thisPath,'..','..','..','..')
const distPath = path.resolve(root,'dist')
const wasmPath = path.resolve(distPath,'wasm')

const child_process = require('child_process');
function git(command) {
    return child_process.execSync(`git ${command}`, { encoding: 'utf8' }).trim();
}

const BUILD_INFO = require('fs').readFileSync('../../../../dist/build.json', 'utf8');

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
            path.resolve(thisPath,'src','run.js'),
            path.resolve(thisPath,'src','style.css'),
            path.resolve(wasmPath,'ide.wasm'),
        ]),
        new webpack.DefinePlugin({
            GIT_HASH: JSON.stringify(git('rev-parse HEAD')),
            GIT_STATUS: JSON.stringify(git('status --short --porcelain')),
            BUILD_INFO: JSON.stringify(BUILD_INFO),
        })
    ],
    devServer: {
        publicPath: '/assets/',
        historyApiFallback: {
            index: '/assets/'
        }
    },
    resolve: {
        alias: {
            wasm_rust_glue$: path.resolve(wasmPath,'ide.js')
        }
    },
    performance: {
        hints: false,
    },
    mode: 'none',
    stats: 'minimal',
    module: {
        rules: [
            {
                test: /\.ya?ml$/,
                type: 'json',
                use: 'yaml-loader'
            }
        ]
    }
}
