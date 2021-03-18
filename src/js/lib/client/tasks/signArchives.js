/**
 This script signs the content of all archives that we have.

 Our use case requires us to re-sign jar contents that cannot be opened as pure
 zip archives, but require a java toolchain to extract and re-assemble.
 This code is based on https://github.com/electron/electron-osx-sign/pull/231 but our use case
 is unlikely to be supported by electron-osx-sign as adds a java toolchain as additional
 dependency..
 This script should be removed once the engine is signed.
*/
const path = require('path')
const child_process = require('child_process')
const { dist } = require('../../../../../build/paths')

const contentRoot = path.join(dist.root, 'client', 'mac', 'Enso.app', 'Contents')
const resRoot = path.join(contentRoot, 'Resources')

// TODO: Remove this once https://github.com/enso-org/ide/issues/1359 has been implemented.
const ENGINE = '0.2.9'
const ID = '"Developer ID Application: New Byte Order Sp. z o. o. (NM77WTZJFQ)"'
// Placeholder name for temporary archives.
const tmpArchive = 'compressed.zip'

// Helper to execute a command in a given working dir and return the output.
const run = (cmd, cwd) => child_process.execSync(cmd, { shell: true, cwd }).toString()

// Run the signing command with our specific settings.
function sign(targetPath, cwd) {
    console.log(`Signing ${targetPath} in ${cwd}`)
    const entitlements_path = path.resolve('./', 'entitlements.mac.plist')
    return run(
        `codesign -vvv --entitlements ${entitlements_path} --force --options=runtime --sign ${ID} ${targetPath}`,
        cwd
    )
}

// Create and return an empty working directory.
function getTmpDir() {
    const workingDir = '.temp'
    run(`rm -rf ${workingDir}`)
    run(`mkdir ${workingDir}`)
    return path.resolve(workingDir)
}

/**
 * Sign content of an archive. This function extracts the archive, signs the required files,
 * re-packages the archive and replaces the original.
 *
 * @param {string} archivePath - folder the archive is located in.
 * @param {string} archiveName - file name of the archive
 * @param {string[]} binPaths - paths of files to be signed. Must be relative to archive root.
 */
function signArchive(archivePath, archiveName, binPaths) {
    const sourceArchive = path.join(archivePath, archiveName)
    const workingDir = getTmpDir()
    try {
        const isJar = archiveName.endsWith(`jar`)

        if (isJar) {
            run(`jar xf ${sourceArchive}`, workingDir)
        } else {
            run(`unzip -d${workingDir} ${sourceArchive}`)
        }

        for (let binary of binPaths) {
            sign(binary, workingDir)
        }

        if (isJar) {
            run(`jar -cf ${tmpArchive} . `, workingDir)
        } else {
            run(`zip -rm ${tmpArchive} . `, workingDir)
        }

        console.log(run(`/bin/mv ${workingDir}/${tmpArchive} ${sourceArchive}`))
        run(`rm -R ${workingDir}`)
        console.log(
            `Successfully repacked ${sourceArchive} to handle signing inner native dependency.`
        )
    } catch (error) {
        run(`rm -R ${workingDir}`)
        console.error(
            `Could not repackage ${archiveName}.  Please check the "signArchives.js" task in ` +
                `client/tasks to ensure that it's working. This jar has to be treated specially` +
                ` because it has a native library and apple's codesign does not sign inner ` +
                `native libraries correctly for jar files`
        )
        throw error
    }
}

// Archives, and their content that need to be signed in an extra step.
const toSign = [
    [
        `enso/dist/${ENGINE}/std-lib/Standard/polyglot/java`,
        'sqlite-jdbc-3.34.0.jar',
        [
            'org/sqlite/native/Mac/aarch64/libsqlitejdbc.jnilib',
            'org/sqlite/native/Mac/x86_64/libsqlitejdbc.jnilib',
        ],
    ],
    [
        `enso/dist/${ENGINE}/component`,
        'runner.jar',
        [
            'org/sqlite/native/Mac/x86_64/libsqlitejdbc.jnilib',
            'com/sun/jna/darwin/libjnidispatch.jnilib',
        ],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.jartool.jmod',
        ['bin/jarsigner', 'bin/jar'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.jdeps.jmod',
        ['bin/javap', 'bin/jdeprscan', 'bin/jdeps'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.jstatd.jmod',
        ['bin/jstatd'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.pack.jmod',
        ['bin/unpack200', 'bin/pack200'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.hotspot.agent.jmod',
        ['bin/jhsdb'],
    ],
    ['enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods', 'jdk.jfr.jmod', ['bin/jfr']],
    ['enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods', 'jdk.rmic.jmod', ['bin/rmic']],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'java.rmi.jmod',
        ['bin/rmid', 'bin/rmiregistry'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'java.base.jmod',
        ['bin/java', 'bin/keytool', 'lib/jspawnhelper'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.jlink.jmod',
        ['bin/jmod', 'bin/jlink', 'bin/jimage'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.scripting.nashorn.shell.jmod',
        ['bin/jjs'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.jcmd.jmod',
        ['bin/jstack', 'bin/jcmd', 'bin/jps', 'bin/jmap', 'bin/jstat', 'bin/jinfo'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.jshell.jmod',
        ['bin/jshell'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.compiler.jmod',
        ['bin/javac', 'bin/serialver'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'java.scripting.jmod',
        ['bin/jrunscript'],
    ],
    ['enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods', 'jdk.jdi.jmod', ['bin/jdb']],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.javadoc.jmod',
        ['bin/javadoc'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.jconsole.jmod',
        ['bin/jconsole'],
    ],
    [
        'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/jmods',
        'jdk.javadoc.jmod',
        ['bin/javadoc'],
    ],
]

// Extra files that need to be signed.
const extra = [
    'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/MacOS/libjli.dylib',
    'enso/runtime/graalvm-ce-java11-21.0.0.2/Contents/Home/languages/llvm/native/bin/ld.lld',
    'enso/runtime/graalvm-ce-java11-21.0.0.2',
]

exports.default = async function () {
    // Sign archive.
    for (let toSignData of toSign) {
        const jarDir = path.join(resRoot, toSignData[0])
        const jarName = toSignData[1]
        const jarContent = toSignData[2]
        signArchive(jarDir, jarName, jarContent)
    }
    // Sign single binaries.
    for (let toSign of extra) {
        const target = path.join(resRoot, toSign)
        sign(target)
    }
    // Finally re-sign the top-level enso.
    sign(path.join(contentRoot, 'MacOs/Enso'))
}
