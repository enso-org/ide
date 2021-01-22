const fss = require('fs')

function get_build_config() {
    const buildInfoPath = paths.dist.buildInfo

    let exists = fss.existsSync(buildInfoPath)
    if (exists) {
        let configFile = fss.readFileSync(buildInfoPath)
        return JSON.parse(configFile.toString())
    }
}
const build = get_build_config()

let config = {
    name: "Enso",
    description: "Enso Data Processing Environment.",
    main: "index.js",

    dependencies: {
        "create-servers": "^3.1.0",
        "electron-is-dev": "^1.1.0",
        "enso-studio-common": "2.0.0-alpha.0",
        "enso-studio-content": "2.0.0-alpha.0",
        "enso-studio-icons": "2.0.0-alpha.0",
        "yargs": "^15.3.0"
    },

    devDependencies: {
        "compression-webpack-plugin": "^3.1.0",
        "copy-webpack-plugin": "^5.1.1",
        "devtron": "^1.4.0",
        "electron": "11.1.1",
        "electron-builder": "^22.9.1"
    },

    scripts: {
        start: `electron ${paths.dist.content} -- `,
        build: 'webpack ',
        dist: 'electron-builder --publish never' + ' --' + build.target,
    },
}

config.build = {
    appId: 'org.enso',
    productName: 'Enso',
    copyright: 'Copyright © 2020 ${author}.',
    mac: {
        icon: `${paths.dist.root}/icons/icon.icns`,
        category: 'public.app-category.developer-tools',
        darkModeSupport: true,
        type: 'distribution',
    },
    win: {
        icon: `${paths.dist.root}/icons/icon.ico`,
    },
    linux: {
        icon: `${paths.dist.root}/icons/png`,
        category: 'Development',
    },
    files: [
        { from: paths.dist.content, to: '.' }
    ],
    extraResources: [
        { from: paths.dist.bin, to: '.' , filter: ["!**.tar.gz", "!**.zip"]}
    ],
    fileAssociations: [
        {
            ext: 'enso',
            name: 'Enso Source File',
            role: 'Editor',
        }
    ],
    directories: {
        output: paths.dist.client,
    },
    publish: [],
}

module.exports = {config}
