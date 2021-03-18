require('dotenv').config();
const { notarize } = require('electron-notarize');

exports.default = async function notarizing(context) {
    const { electronPlatformName, appOutDir } = context;
    if (electronPlatformName !== 'darwin') {
        return;
    }
    // We need to manually re-sign our build artifacts before notarisation.
    // See the script for more information.
    console.log("  • Extra Signing.")
    await require("./signArchives").default()

    // Notarize the application.
    const appName = context.packager.appInfo.productFilename;
    console.log("  • Notarizing.")
    return await notarize({
        appBundleId: 'com.enso.ide',
        appPath: `${appOutDir}/${appName}.app`,
        appleId: process.env.APPLEID,
        appleIdPassword: process.env.APPLEIDPASS,
    });
};
