let config = {
    name: "enso-studio-client",
    description: "The standalone client for the Enso IDE.",
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
        "electron": "8.1.1",
        "electron-builder": "^22.3.2"
    },

    scripts: {
        "start": `electron ${paths.dist.content} -- `,
        "build": "webpack ",
        "dist": "electron-builder --publish never",
        "dist:crossplatform": "electron-builder --mac --win --linux --publish never"
    }
}

config.build = {
    appId: "org.enso.studio",
    productName: "Enso Studio",
    copyright: "Copyright © 2020 ${author}.",
    mac: {
        icon: `${paths.dist.root}/icons/icon.icns`,
        category: "public.app-category.developer-tools",
        darkModeSupport: true,
        type: "distribution"
    },
    win: {
        icon: `${paths.dist.root}/icons/icon.ico`,
    },
    linux: {
        icon: `${paths.dist.root}/icons/png`,
        category: "Development"
    },
    files: [
        { from: paths.dist.content, to: "." }
    ],
    fileAssociations: [
        {
            ext: "enso",
            name: "Enso Source File",
            role: "Editor"
        },
        {
            ext: "enso-studio",
            name: "Enso Studio Project",
            role: "Editor"
        }
    ],
    directories: {
        "output": paths.dist.client
    },
    publish: []
}

module.exports = {config}
