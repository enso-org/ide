const Copy = require('copy-webpack-plugin')
const path = require('path')

const thisPath = path.resolve(__dirname)
const root     = path.resolve(thisPath,'..','..','..','..')
const distPath = path.resolve(root,'dist')

module.exports = {
    entry: {
        index: path.resolve(thisPath,'src','index.js'),
    },
    mode: 'production',
    target: "electron-main",
    output: {
        path: path.resolve(distPath,'unpacked'),
        filename: '[name].js',
    },
    plugins: [
        new Copy([
            {
                from : path.resolve(thisPath,'..','content','dist','assets'),
                to   : path.resolve(distPath,'unpacked','assets')
            },
            {
                from : path.resolve(thisPath,'package.json'),
                to   : path.resolve(distPath,'unpacked','package.json')
            },
            {
                from : path.resolve(thisPath,'src','preload.js'),
                to   : path.resolve(distPath,'unpacked','preload.js')
            }
        ]),
    ],
    performance: {
        hints: false,
    },
    stats: 'minimal',
}
