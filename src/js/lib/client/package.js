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
        "enso-studio-common": "1.0.0",
        "enso-studio-content": "1.0.0",
        "enso-studio-icons": "1.0.0",
        "yargs": "^15.3.0"
    },

    devDependencies: {
        "compression-webpack-plugin": "^3.1.0",
        "copy-webpack-plugin": "^5.1.1",
        "devtron": "^1.4.0",
        "electron": "11.1.1",
        "electron-builder": "^22.10.5",
        "crypto-js": "4.0.0",
        "electron-notarize" : "1.0.0",
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
    copyright: 'Copyright © 2021 ${author}.',
    artifactName: 'enso-${os}-${version}.${ext}',
    mac: {
        // We do not use compression as the build time is huge and file size saving is almost zero.
        target: ['dmg'],
        icon: `${paths.dist.root}/icons/icon.icns`,
        category: 'public.app-category.developer-tools',
        darkModeSupport: true,
        type: 'distribution',
        // Settings required for notarisation.
        hardenedRuntime: true,
        gatekeeperAssess: false,
        entitlements: './entitlements.mac.plist',
        entitlementsInherit: './entitlements.mac.plist',
    },
    win: {
        // We do not use compression as the build time is huge and file size saving is almost zero.
        target: ['nsis'],
        icon: `${paths.dist.root}/icons/icon.ico`,
    },
    linux: {
        // We do not use compression as the build time is huge and file size saving is almost zero.
        target: ['AppImage'],
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
    nsis: {
        // Disables "block map" generation during electron building. Block maps
        // can be used for incremental package update on client-side. However,
        // their generation can take long time (even 30 mins), so we removed it
        // for now. Moreover, we may probably never need them, as our updates
        // are handled by us. More info:
        // https://github.com/electron-userland/electron-builder/issues/2851
        // https://github.com/electron-userland/electron-builder/issues/2900
        differentialPackage: false
    },
    dmg: {
        // Disables "block map" generation during electron building. Block maps
        // can be used for incremental package update on client-side. However,
        // their generation can take long time (even 30 mins), so we removed it
        // for now. Moreover, we may probably never need them, as our updates
        // are handled by us. More info:
        // https://github.com/electron-userland/electron-builder/issues/2851
        // https://github.com/electron-userland/electron-builder/issues/2900
        writeUpdateInfo: false,
        sign: false
    },
    publish: [],
    afterAllArtifactBuild: 'tasks/computeHashes.js',
    afterSign: "tasks/notarize.js",
}

module.exports = {config}
