/**
 This script signs the content of all archives that we have for macOS. For this to work this needs
 to run on macOS with `codesign`, and a JDK installed. `codesign` is needed to sign the files,
 while the JDK is needed for correct packing and unpacking of java archives.

 We require this extra step as our dependencies contain files that require us to re-sign jar
 contents that cannot be opened as pure zip archives, but require a java toolchain to extract
 and re-assemble to preserve manifest information. This functionality is not provided by
 `electron-osx-sign` out of the box.

 This code is based on https://github.com/electron/electron-osx-sign/pull/231 but our use-case
 is unlikely to be supported by electron-osx-sign as it adds a java toolchain as additional
 dependency.
 This script should be removed once the engine is signed.
**/
const path = require('path')
const child_process = require('child_process')
const { dist } = require('../../../../../build/paths')

const contentRoot = path.join(dist.root, 'client', 'mac', 'Enso.app', 'Contents')
const resRoot = path.join(contentRoot, 'Resources')

// TODO: Refactor this once we have a better wau to get the used engine version.
//  See the tracking issue for more information https://github.com/enso-org/ide/issues/1359
const ENGINE = '0.2.12'
const ID = '"Developer ID Application: New Byte Order Sp. z o. o. (NM77WTZJFQ)"'
// Placeholder name for temporary archives.
const tmpArchive = 'temporary_archive.zip'

const GRAALVM = 'graalvm-ce-java11-21.1.0'

// Helper to execute a command in a given directory and return the output.
const run = (cmd, cwd) => child_process.execSync(cmd, { shell: true, cwd }).toString()

// Run the signing command.
function sign(targetPath, cwd) {
    console.log(`Signing ${targetPath} in ${cwd}`)
    const entitlements_path = path.resolve('./', 'entitlements.mac.plist')
    return run(
        `codesign -vvv --entitlements ${entitlements_path} --force --options=runtime ` +
            `--sign ${ID} ${targetPath}`,
        cwd
    )
}

// Create and return an empty directory in the current folder. The directory will be named `.temp`.
// If it already exists all content will be deleted.
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
            if (archiveName.includes(`runner`)) {
                run(`jar -cfm ${tmpArchive} META-INF/MANIFEST.MF . `, workingDir)
            } else {
                run(`jar -cf ${tmpArchive} . `, workingDir)
            }
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

// Archives, and their content that need to be signed in an extra step. If a new archive is added
// to the engine dependencies this also needs to be added here. If an archive is not added here, it
// will show up as a failure to notarise the IDE. The offending archive will be named in the error
// message provided by Apple and can then be added here.
const toSign = [
    {
        jarDir: `enso/dist/${ENGINE}/std-lib/Standard/polyglot/java`,
        jarName: 'sqlite-jdbc-3.34.0.jar',
        jarContent: [
            'org/sqlite/native/Mac/aarch64/libsqlitejdbc.jnilib',
            'org/sqlite/native/Mac/x86_64/libsqlitejdbc.jnilib',
        ],
    },
    {
        jarDir: `enso/dist/${ENGINE}/component`,
        jarName: 'runner.jar',
        jarContent: [
            'org/sqlite/native/Mac/x86_64/libsqlitejdbc.jnilib',
            'com/sun/jna/darwin/libjnidispatch.jnilib',
        ],
    },
    {
        jarDir: `enso/dist/${ENGINE}/component`,
        jarName: 'runtime.jar',
        jarContent: [
            'org/sqlite/native/Mac/x86_64/libsqlitejdbc.jnilib',
            'com/sun/jna/darwin/libjnidispatch.jnilib',
        ],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'java.base.jmod',
        jarContent: ['bin/java', 'bin/keytool', 'lib/jspawnhelper'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'java.rmi.jmod',
        jarContent: ['bin/rmid', 'bin/rmiregistry'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'java.scripting.jmod',
        jarContent: ['bin/jrunscript'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.compiler.jmod',
        jarContent: ['bin/javac', 'bin/serialver'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.hotspot.agent.jmod',
        jarContent: ['bin/jhsdb'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.jartool.jmod',
        jarContent: ['bin/jarsigner', 'bin/jar'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.javadoc.jmod',
        jarContent: ['bin/javadoc'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.jcmd.jmod',
        jarContent: ['bin/jstack', 'bin/jcmd', 'bin/jps', 'bin/jmap', 'bin/jstat', 'bin/jinfo'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.jconsole.jmod',
        jarContent: ['bin/jconsole'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.jdeps.jmod',
        jarContent: ['bin/javap', 'bin/jdeprscan', 'bin/jdeps'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.jdi.jmod',
        jarContent: ['bin/jdb'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.jfr.jmod',
        jarContent: ['bin/jfr'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.jlink.jmod',
        jarContent: ['bin/jmod', 'bin/jlink', 'bin/jimage'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.jshell.jmod',
        jarContent: ['bin/jshell'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.jstatd.jmod',
        jarContent: ['bin/jstatd'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.pack.jmod',
        jarContent: ['bin/unpack200', 'bin/pack200'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.rmic.jmod',
        jarContent: ['bin/rmic'],
    },
    {
        jarDir: `enso/runtime/${GRAALVM}/Contents/Home/jmods`,
        jarName: 'jdk.scripting.nashorn.shell.jmod',
        jarContent: ['bin/jjs'],
    },
]

// Extra files that need to be signed.
const extra = [
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/graphics/libs/graphics.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/cluster/libs/cluster.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/parallel/libs/parallel.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/nnet/libs/nnet.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/splines/libs/splines.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/rpart/libs/rpart.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/spatial/libs/spatial.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/MASS/libs/MASS.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/grid/libs/grid.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/lattice/libs/lattice.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/class/libs/class.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/tools/libs/tools.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/KernSmooth/libs/KernSmooth.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/foreign/libs/foreign.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/methods/libs/methods.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/grDevices/libs/grDevices.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/Matrix/libs/Matrix.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/base/libs/base.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/utils/libs/utils.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/survival/libs/survival.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/nlme/libs/nlme.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/stats/libs/stats.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/lib/graalpython-38-native-x86_64-darwin/libbz2.so`,

    `enso/runtime/${GRAALVM}/Contents/Home/bin/jrunscript`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/keytool`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/java`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jmap`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/serialver`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/javac`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jimage`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jhsdb`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jar`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jvisualvm`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jshell`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/javadoc`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/rmic`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/unpack200`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jfr`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jdeps`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jdeprscan`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jlink`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/rmid`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jstack`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/rmiregistry`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jinfo`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jstat`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jdb`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/javap`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jstatd`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/pack200`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jcmd`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jconsole`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jjs`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jps`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jmod`,
    `enso/runtime/${GRAALVM}/Contents/Home/bin/jarsigner`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/graphics/libs/graphics.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/cluster/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/cluster/libs/cluster.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/parallel/libs/parallel.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/nnet/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/nnet/libs/nnet.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/splines/libs/splines.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/rpart/help/figures/rpart.png`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/rpart/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/rpart/libs/rpart.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/boot/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/spatial/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/spatial/libs/spatial.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/MASS/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/MASS/libs/MASS.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/codetools/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/grid/libs/grid.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/lattice/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/lattice/libs/lattice.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/class/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/class/libs/class.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/tools/libs/tools.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/KernSmooth/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/KernSmooth/libs/KernSmooth.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/foreign/files/sids.dbf`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/foreign/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/foreign/libs/foreign.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/methods/libs/methods.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/grDevices/libs/grDevices.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/Matrix/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/Matrix/libs/Matrix.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/base/libs/base.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/utils/libs/utils.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/survival/R/cipoisson.R`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/survival/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/survival/libs/survival.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/nlme/html/R.css`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/nlme/libs/nlme.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/library/stats/libs/stats.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-vi`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/BATCH`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-bzip2`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-make`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-yacc`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/LINK`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/INSTALL`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/R`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/rtags`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/REMOVE`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-zip`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/RMain`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/SHLIB`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-open`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/Rdiff`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/f2c`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-less`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/Stangle`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-gcc`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/f2c-wrapper`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/libtool`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/configure_fastr`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-texi2dvi`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/pager`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-gzip`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/install_r_native_image`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/check`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/Rcmd`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/COMPILE`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-sed`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/Sweave`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/Rprof`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/Rdconv`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-lpr`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-gfortran`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-g++`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-ar`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/Rd2pdf`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-tar`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-xdg-open`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-ranlib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/mkinstalldirs`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/Rscript`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/javareconf`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/exec/R`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/build`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/safe-forward-unzip`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/bin/config`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/lib/libR.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/lib/libRblas.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/lib/libRnative.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/lib/libRlapack.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/lib/libRllvm.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/R/lib/libf2c.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/js/bin/js`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/bin/lli`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/bin/graalvm-native-clang`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/bin/graalvm-native-binutil`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/bin/graalvm-native-ld`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/bin/graalvm-native-clang++`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/lib/libsulong.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/lib/libgraalvm-llvm.1.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/lib/libc++.1.0.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/lib/libsulong-native.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/lib/libc++abi.1.0.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/llvm/native/lib/libsulong++.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/libzsupport.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/libposix.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/libbz2support.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/libhpy.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/lib/graalpython-38-native-x86_64-darwin/libbz2.so`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/lib/graalpython-38-native-x86_64-darwin/liblzma.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/lib/graalpython-38-native-x86_64-darwin/liblzma.5.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/lib/graalpython-38-native-x86_64-darwin/libbz2.so.1.0`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/lib/graalpython-38-native-x86_64-darwin/liblzma.5.2.5.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/libzsupport.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/libposix.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/_testmultiphase.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/_cpython_sre.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/libbz2support.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/_cpython_struct.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/libhpy.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/_testcapi.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/_cpython_unicodedata.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/_mmap.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/liblzmasupport.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/_bz2.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/modules/libpython.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/liblzmasupport.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-graalpython/libpython.graalpython-38-native-x86_64-darwin.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/bin/graalpython`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/smtplib.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/lib2to3/pgen2/token.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/quopri.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/encodings/rot_13.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/yinyang.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/planet_and_moon.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/lindenmayer.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/penrose.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/fractalcurves.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/paint.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/forest.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/tree.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/bytedesign.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/minimal_hanoi.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/clock.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/two_canvases.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/turtledemo/peace.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/pdb.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/ctypes/macholib/fetch_macholib`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/smtpd.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/platform.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/tarfile.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/timeit.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/base64.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/idlelib/pyshell.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/idlelib/idle.bat`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/trace.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/tabnanny.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/profile.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/cgi.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/cProfile.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/uu.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/languages/python/lib-python/3/webbrowser.py`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/libjvmcicompiler.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/jspawnhelper`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/polyglot/bin/polyglot`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/visualvm/platform/lib/nbexec`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-reduce`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-dwarfdump`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/clang-format`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-config`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-ifs`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/clang-10`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-link`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/opt`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-dis`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-ar`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/lld`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-objcopy`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-readobj`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-as`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-diff`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-extract`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-nm`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/lli`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llvm-objdump`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/bin/llc`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/lib/libLTO.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/lib/libc++.1.0.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/lib/libclang-cpp.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/lib/libLLVM.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/llvm/lib/libc++abi.1.0.dylib`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/installer/bin/gu`,
    `enso/runtime/${GRAALVM}/Contents/Home/lib/libtrufflenfi.dylib`,
]

exports.default = async function () {
    // Sign archives.
    for (let toSignData of toSign) {
        const jarDir = path.join(resRoot, toSignData.jarDir)
        const jarName = toSignData.jarName
        const jarContent = toSignData.jarContent
        console.log({ jarDir, jarName, jarContent })
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
