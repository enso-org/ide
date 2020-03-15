
module.exports = {
    config: {
        name: "enso-studio-content",
        scripts: {
            "build": "webpack",
            "watch": "webpack-dev-server"
        },
        dependencies: {
            "enso-studio-common": "1.0.0",
            "copy-webpack-plugin": "^5.1.1"
        },
        devDependencies: {
            "compression-webpack-plugin": "^3.1.0",
            "copy-webpack-plugin": "^5.1.1"
        }
    }
}
