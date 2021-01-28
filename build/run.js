const cmd      = require('./cmd')
const fs       = require('fs').promises
const fss      = require('fs')
const glob     = require('glob')
const ncp      = require('ncp').ncp
const os       = require('os')
const path     = require('path')
const paths    = require('./paths')
const prettier = require("prettier")
const stream   = require('stream')
const yargs    = require('yargs')
const zlib     = require('zlib')
const child_process = require('child_process')

process.on('unhandledRejection', error => { throw(error) })
process.chdir(paths.root)


const { promisify } = require('util')
const pipe = promisify(stream.pipeline)

async function gzip(input, output) {
  const gzip        = zlib.createGzip()
  const source      = fss.createReadStream(input)
  const destination = fss.createWriteStream(output)
  await pipe(source,gzip,destination)
}

const yaml = require('js-yaml');

let doc = yaml.load(`
defaults: &defaults
  A: 1
  B: 2
mapping:
  << : *defaults
  A: 23
  C: 99
 `)


// =================
// === Constants ===
// =================

const NODE_VERSION      = '14.15.0'
const RUST_VERSION      = 'nightly-2019-11-04'
const WASM_PACK_VERSION = '0.9.1'



// =============
// === Utils ===
// =============

function job(platforms,name,steps,cfg) {
    if (!cfg) { cfg = {} }
    return {
        name: name,
        "runs-on": "${{ matrix.os }}",
        strategy: {
            matrix: {
              os: platforms
            },
            "fail-fast": false
        },
        steps : list({uses:"actions/checkout@v2"}, ...steps),
        ...cfg
    }
}

function job_on_all_platforms(...args) {
    return job(["windows-latest", "macOS-latest", "ubuntu-latest"],...args)
}

function job_on_macos(...args) {
    return job(["macOS-latest"],...args)
}

function list(...args) {
    let out = []
    for (let arg of args) {
        if (Array.isArray(arg)) {
            out.push(...arg)
        } else {
            out.push(arg)
        }
    }
    return out
}



// ====================
// === Dependencies ===
// ====================

let installRust = {
    name: "Install Rust",
    uses: "actions-rs/toolchain@v1",
    with: {
        toolchain: RUST_VERSION,
        override: true
    }
}

let installNode = {
    name: "Install Node",
    uses: "actions/setup-node@v1",
    with: {
        "node-version": NODE_VERSION,
    }
}

let installPrettier = {
    name: "Install Prettier",
    run: "npm install --save-dev --save-exact prettier"
}

let installClippy = {
    name: "Install Clippy",
    run: "rustup component add clippy"
}


function installWasmPackOn(name,sys,pkg) {
    return {
        name: `Install wasm-pack (${name})`,
        env: {
            WASMPACKURL: `https://github.com/rustwasm/wasm-pack/releases/download/v${WASM_PACK_VERSION}`,
            WASMPACKDIR: `wasm-pack-v${WASM_PACK_VERSION}-x86_64-${pkg}`,
        },
        run: `
            curl -L "$WASMPACKURL/$WASMPACKDIR.tar.gz" | tar -xz -C .
            mv $WASMPACKDIR/wasm-pack ~/.cargo/bin
            rm -r $WASMPACKDIR`,
        shell: "bash",
        if: `matrix.os == '${sys}-latest'`,
    }
}

let installWasmPackOnMacOS   = installWasmPackOn('macOS','macOS','apple-darwin')
let installWasmPackOnWindows = installWasmPackOn('Windows','windows','pc-windows-msvc')
let installWasmPackOnLinux   = installWasmPackOn('Linux','ubuntu','unknown-linux-musl')

// We could use cargo install wasm-pack, but that takes 3.5 minutes compared to few seconds.
let installWasmPack = [installWasmPackOnMacOS, installWasmPackOnWindows, installWasmPackOnLinux]



// =============================
// === Build, Lint, and Test ===
// =============================

function buildOn(name,sys) {
    return {
        name: `Build (${name})`,
        run: `node ./run dist --skip-version-validation --target ${name}`,
        if: `matrix.os == '${sys}-latest'`
    }
}

buildOnMacOS   = buildOn('macos','macos')
buildOnWindows = buildOn('win','windows')
buildOnLinux   = buildOn('linux','ubuntu')

let lintJavaScript = {
    name: "Lint JavaScript sources",
    run: "npx prettier --check 'src/**/*.js'",
}

let lintRust = {
    name: "Lint Rust sources",
    run: "node ./run lint --skip-version-validation",
}

let testNoWASM = {
    name: "Run tests (no WASM)",
    run: "node ./run test --no-wasm --skip-version-validation",
}

let testWASM = {
    name: "Run tests (WASM)",
    run: "node ./run test --no-native --skip-version-validation",
}



// =================
// === Artifacts ===
// =================

let uploadContentArtifacts = {
    name: `Upload Content Artifacts`,
    uses: "actions/upload-artifact@v1",
    with: {
       name: 'content',
       path: `dist/content`
    },
    if: `matrix.os == 'macOS-latest'`
}

function uploadBinArtifactsFor(name,sys,ext,sfx) {
    return {
        name: `Upload Artifacts (${name}, ${ext})`,
        uses: "actions/upload-artifact@v1",
        with: {
           name: `Enso (${name})`,
           path: `dist/client/Enso${sfx}2.0.0-alpha.0.${ext}`
        },
        if: `matrix.os == '${sys}-latest'`
    }
}

uploadBinArtifactsForMacOS   = uploadBinArtifactsFor('Linux','ubuntu','AppImage','-')
uploadBinArtifactsForWindows = uploadBinArtifactsFor('Windows','windows','exe',' Setup ')
uploadBinArtifactsForLinux   = uploadBinArtifactsFor('macOS','macos','dmg','-')

let downloadArtifacts = {
    name: "Download artifacts",
    uses: "actions/download-artifact@v2",
    with: {
        path: "artifacts"
    }
}



// ======================
// === GitHub Release ===
// ======================

let getCurrentReleaseChangelogInfo = {
    id: 'changelog',
    run: `
        content=\`cat CURRENT_RELEASE_CHANGELOG.json\`
        echo "::set-output name=content::$content"
    `
}

let uploadGitHubRelease = {
    name: `Upload GitHub Release`,
    uses: "softprops/action-gh-release@v1",
    env: {
        GITHUB_TOKEN: "${{ secrets.GITHUB_TOKEN }}"
    },
    with: {
        files:    "artifacts/**/Enso*",
        tag_name: "v${{fromJson(steps.changelog.outputs.content).version}}",
        path:     "${{fromJson(steps.changelog.outputs.content).body}}",
    },
}



// ===================
// === CDN Release ===
// ===================

prepareDistributionVersionCDN = {
    shell: "bash",
    run: `
        ref=\${{ github.ref }}
        refversion=\${ref#"refs/tags/ide-"}
        echo "DIST_VERSION=$refversion" >> $GITHUB_ENV
    `
}

prepareAwsSessionCDN = {
    shell: "bash",
    run: `
        aws configure --profile s3-upload <<-EOF > /dev/null 2>&1
        \${{ secrets.ARTEFACT_S3_ACCESS_KEY_ID }}
        \${{ secrets.ARTEFACT_S3_SECRET_ACCESS_KEY }}
        us-west-1
        text
        EOF
    `
}

function uploadToCDN(name) {
    return {
        name: `Upload '${name}' to CDN`,
        shell: "bash",
        run: `
            aws s3 cp ./artifacts/content/assets/${name}
            s3://ensocdn/ide/\${{ env.DIST_VERSION }}/${name} --profile
            s3-upload --acl public-read --content-encoding gzip
        `
    }
}



// ================
// === Workflow ===
// ================

let workflow = {
    name : "GUI CI",
    on: ['push'],
    jobs: {
        lint: job_on_macos("Linter", [
            installNode,
            installRust,
            installPrettier,
            installClippy,
            lintJavaScript,
            lintRust
        ]),
        test: job_on_macos("Tests", [
            installNode,
            installRust,
            testNoWASM,
        ]),
        "wasm-test": job_on_macos("WASM Tests", [
            installNode,
            installRust,
            installWasmPack,
            testWASM
        ]),
        build: job_on_all_platforms("Build", [
            installNode,
            installRust,
            installWasmPack,
            buildOnMacOS,
            buildOnWindows,
            buildOnLinux,
            uploadContentArtifacts,
            uploadBinArtifactsForMacOS,
            uploadBinArtifactsForWindows,
            uploadBinArtifactsForLinux,
        ],{
            // FIXME:
            if: "startsWith(github.ref,'refs/tags/') || github.ref == 'refs/heads/unstable' || github.ref == 'refs/heads/stable' || github.ref == 'refs/heads/wip/wd/ci'",
        }),
        release_to_github: job_on_macos("GitHub Release", [
              downloadArtifacts,
              getCurrentReleaseChangelogInfo,
              uploadGitHubRelease,
        ],{
            needs: ["lint","test","wasm-test","build"],
            if: "startsWith(github.ref,'refs/tags/')",
        }),
        release_to_cdn: job_on_macos("CDN Release", [
              downloadArtifacts,
              prepareDistributionVersionCDN,
              prepareAwsSessionCDN,
              uploadToCDN('index.js.gz'),
              uploadToCDN('style.css'),
              uploadToCDN('style.css'),
              uploadToCDN('ide.wasm'),
              uploadToCDN('wasm_imports.js.gz'),
        ],{
            needs: ["lint","test","wasm-test","build"],
            if: "startsWith(github.ref,'refs/tags/')",
        }),
    }
}




let header = `
# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
# DO NOT CHANGE THIS FILE. IT WAS GENERATED FROM 'workflows.js'. READ DOCS THERE TO LEARN MORE.
# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
`


let release_workflow_out = header + '\n' + yaml.dump(workflow,{noRefs:true})
fss.writeFileSync(path.join(paths.github.workflows,'gui-ci.yml'),release_workflow_out)


const CHANGELOG_FILE_NAME = 'CHANGELOG.md'
const CHANGELOG_FILE      = path.join(paths.root,CHANGELOG_FILE_NAME)



const semver = require('semver')

class ChangelogEntry {
    constructor(version,body) {
        let semVersion = semver.valid(version)
        if (version !== semVersion) {
            throw `The version '${version}' is not a valid semantic varsion.`
        }
        this.version = version
        this.body    = body
    }
}

function extractChangelog(version) {
    let text    = '\n' + fss.readFileSync(CHANGELOG_FILE,"utf8")
    let chunks  = text.split(/\r?\n## /)
    let entries = chunks.filter((s) => s != '')
    let header  = `Enso ${version}`
    for (let entry of entries) {
        if (entry.startsWith(header)) {
            let body = entry.split(header.length)
            return body
        }
    }
    throw `The changelog for version '${version}' was not found. Please update it in the ${CHANGELOG_FILE_NAME}.`
}

function changelogSections() {
    let text    = '\n' + fss.readFileSync(CHANGELOG_FILE,"utf8")
    let chunks  = text.split(/\r?\n# /)
    return chunks.filter((s) => s != '')
}

function changelogEntries() {
    let sections = changelogSections()
    let prefix   = "Enso "
    let entries  = []
    for (let section of sections) {
        if (!section.startsWith(prefix)) {
            throw `Improper changelog entry header: ${section}`
        } else {
            let splitPoint = section.indexOf('\n')
            let body       = section.substring(splitPoint).trim()
            let header     = section.substring(0,splitPoint).trim()
            let version    = header.substring(prefix.length)
            entries.push(new ChangelogEntry(version,body))
        }
    }

    var lastVersion = null
    for (let entry of entries) {
        if (lastVersion !== null) {
            if (!semver.lt(entry.version,lastVersion)) {
                throw `Versions are not properly ordered in the changelog (${entry.version} >= ${lastVersion}).`
            }
        }
        lastVersion = entry.version
    }
    return entries
}

function changelogNewestEntry() {
    return changelogEntries()[0]
}

let out = changelogNewestEntry()
console.log(out)


fss.writeFileSync('CURRENT_RELEASE_CHANGELOG.json',JSON.stringify({version:out.version,body:out.body}))


let foo = `
                  content=\`cat CURRENT_RELEASE_CHANGELOG.json\`
                  echo "::set-output name=content::$content"
              `

//console.log(foo)

// ========================
// === Global Variables ===
// ========================

/// Arguments passed to cargo build system called from this script. This variable is set to a
/// specific value after the command line args get parsed.
let cargoArgs = undefined

/// Arguments passed to a target binary if any. This variable is set to a specific value after the
// command line args get parsed.
let targetArgs = undefined



// =============
// === Utils ===
// =============

/// Copy files and directories.
async function copy(src,tgt) {
    return new Promise((resolve, reject) => {
        ncp(src,tgt,(err) => {
            if (err) { reject(`${err}`) }
            resolve()
        })
    })
}

/// Run the command with the provided args and all args passed to this script after the `--` symbol.
async function run_cargo(command,args) {
    await cmd.run(command,args.concat(cargoArgs))
}

/// Run the command with the provided args and all args passed to this script after the `--` symbol.
async function run(command,args) {
    await cmd.run(command,args)
}


/// Defines a new command argument builder.
function command(docs) {
    return {docs}
}

/// Build the project manager module, which downloads the project manager binary for the current
/// platform.
async function build_project_manager() {
    console.log(`Getting project manager manager.`)
    await cmd.with_cwd(paths.js.lib.projectManager , async () => {
        await run('npm',['run-script build'])
    })
}

/// Run the local project manager binary.
function run_project_manager() {
   const bin_path = paths.get_project_manager_path(paths.dist.bin)
   console.log(`Starting the project manager from "${bin_path}".`)
   child_process.execFile(bin_path, [], (error, stdout, stderr) => {
       console.error(stderr)
       if (error) {
           throw error
       }
       console.log(stdout)
       console.log(`Project manager running.`)
   })
}

// ================
// === Commands ===
// ================

const DEFAULT_CRATE = 'ide'
let commands = {}


// === Clean ===

commands.clean = command(`Clean all build artifacts`)
commands.clean.js = async function() {
    await cmd.with_cwd(paths.js.root, async () => {
        await run('npm',['run','clean'])
    })
    try {
        await fs.unlink(paths.dist.init)
        await fs.unlink(paths.dist.buildInit)
    } catch {}
}

commands.clean.rust = async function() {
    await run_cargo('cargo',['clean'])
}


// === Check ===

commands.check = command(`Fast check if project builds (only Rust target)`)
commands.check.rust = async function() {
    await run_cargo('cargo',['check'])
}


// === Build ===

commands.build = command(`Build the sources in release mode`)
commands.build.options = {
    'crate': {
        describe : 'Target crate to build',
        type     : 'string',
    }
}
commands.build.js = async function() {
    console.log(`Building JS target.`)
    await run('npm',['run','build'])
}

commands.build.rust = async function(argv) {
    let crate     = argv.crate || DEFAULT_CRATE
    let crate_sfx = crate ? ` '${crate}'` : ``
    console.log(`Building WASM target${crate_sfx}.`)
    let args = ['build','--target','web','--out-dir',paths.dist.wasm.root,'--out-name','ide',crate]
    if (argv.dev) { args.push('--dev') }
    await run_cargo('wasm-pack',args)
    await patch_file(paths.dist.wasm.glue, js_workaround_patcher)
    await fs.rename(paths.dist.wasm.mainRaw, paths.dist.wasm.main)
    if (!argv.dev) {
        // TODO: Enable after updating wasm-pack
        // https://github.com/rustwasm/wasm-pack/issues/696
        // console.log('Optimizing the WASM binary.')
        // await cmd.run('npx',['wasm-opt','-O3','-o',paths.dist.wasm.mainOpt,paths.dist.wasm.main])
        console.log('Minimizing the WASM binary.')
        await gzip(paths.dist.wasm.main,paths.dist.wasm.mainOptGz) // TODO main -> mainOpt

        console.log('Checking the resulting WASM size.')
        let stats = fss.statSync(paths.dist.wasm.mainOptGz)
        let limit = 4.28
        let size = Math.round(100 * stats.size / 1024 / 1024) / 100
        if (size > limit) {
            throw(`Output file size exceeds the limit (${size}MB > ${limit}MB).`)
        }
    }
}

/// Workaround fix by wdanilo, see: https://github.com/rustwasm/wasm-pack/issues/790
function js_workaround_patcher(code) {
    code = code.replace(/if \(\(typeof URL.*}\);/gs,'return imports')
    code = code.replace(/if \(typeof module.*let result/gs,'let result')
    code = code.replace(/export default init;/gs,'export default init')
    code += '\nexport function after_load\(w,m\) { wasm = w; init.__wbindgen_wasm_module = m;}'
    return code
}

async function patch_file(path,patcher) {
    let code_to_patch = await fs.readFile(path,'utf8')
    let patched_code  = patcher(code_to_patch)
    await fs.writeFile(path,patched_code)
}


// === Start ===

commands.start = command(`Build and start desktop client`)
commands.start.rust = async function(argv) {
   let argv2 = Object.assign({},argv,{dev:true})
   await commands.build.rust(argv2)
}

commands.start.js = async function (argv) {
    console.log(`Building JS target.` + argv)
    const args = targetArgs.concat([
        `--backend-path ${paths.get_project_manager_path(paths.dist.bin)}`,
    ])
    if (argv.dev) { args.push('--dev') }
    await cmd.with_cwd(paths.js.root, async () => {
        await run('npm', ['run', 'start', '--'].concat(args))
    })
}


// === Test ===

commands.test = command(`Run test suites`)
commands.test.rust = async function(argv) {
    if (argv.native) {
        console.log(`Running Rust test suite.`)
        await run_cargo('cargo',['test'])
    }

    if (argv.wasm) {
        console.log(`Running Rust WASM test suite.`)
        let args = ['run','--manifest-path=test/Cargo.toml','--bin','test_all','--','--headless','--chrome']
        await run_cargo('cargo',args)
    }
}


// === Lint ===

commands.lint = command(`Lint the codebase`)
commands.lint.rust = async function() {
    // We run clippy-preview due to https://github.com/rust-lang/rust-clippy/issues/4612
    await run_cargo('cargo',['clippy-preview','-Z','unstable-options','--','-D','warnings'])
}


// === TomlFmt ===

commands['toml-fmt'] = command(`Lint the codebase`)
commands['toml-fmt'].rust = async function() {
    console.log("Looking for all TOML files.")
    let files = glob.sync(paths.rust.root + "/**/*.toml", {cwd:paths.root});
    console.log(`Found ${files.length} entries. Running auto-formatter.`)
    for (let file of files) {
        console.log(`    Formatting '${file}'.`)
        let text = fss.readFileSync(file, "utf8")
        let out  = prettier.format(text,{parser:'toml'})
        fss.writeFileSync(file,out)
    }
}


// === Watch ===

commands.watch          = command(`Start a file-watch utility and run interactive mode`)
commands.watch.options  = Object.assign({},commands.build.options)
commands.watch.parallel = true
commands.watch.rust = async function(argv) {
    let build_args = []
    if (argv.crate !== undefined) {
        build_args.push(`--crate=${argv.crate}`)
    }
    if (argv.backend !== 'false') {
        build_project_manager().then(run_project_manager)
    }

    build_args = build_args.join(' ')
    let target =
        '"' +
        `node ${paths.script.main} build --skip-version-validation --no-js --dev ${build_args} -- ` +
        cargoArgs.join(' ') +
        '"'
    let args = ['watch', '-s', `${target}`]
    await cmd.with_cwd(paths.rust.root, async () => {
        await cmd.run('cargo',args)
    })
}

commands.watch.js = async function() {
    await cmd.with_cwd(paths.js.root, async () => {
        await run('npm',['run','watch'])
    })
}


// === Dist ===

commands.dist = command(`Build the sources and create distribution packages`)
commands.dist.rust = async function(argv) {
    await commands.build.rust(argv)
}

commands.dist.js = async function() {
    await cmd.with_cwd(paths.js.root, async () => {
        await run('npm',['run','dist'])
    })
}



// ===========================
// === Command Line Parser ===
// ===========================

let usage = `run command [options]

All arguments after '--' will be passed to cargo build system.
All arguments after second '--' will be passed to target executable if any.
For example, 'run start -- --dev -- --debug-scene shapes' will pass '--dev' to cargo \
and '--debug-scene shapes' to the output binary.`

let optParser = yargs
    .scriptName("")
    .usage(usage)
    .help()
    .parserConfiguration({'populate--':true})
    .demandCommand()

optParser.options('rust', {
    describe : 'Run the Rust target',
    type     : 'bool',
    default  : true
})

optParser.options('js', {
    describe : 'Run the JavaScript target',
    type     : 'bool',
    default  : true
})

optParser.options('release', {
    describe : "Enable all optimizations",
    type     : 'bool',
})

optParser.options('dev', {
    describe : "Optimize for fast builds",
    type     : 'bool',
})

optParser.options('target', {
    describe:
        'Set the build target. Defaults to the current platform. ' +
        'Valid values are: "linux" "macos" and "win"',
    type: 'string',
})

optParser.options('backend', {
    describe: 'Start the backend process automatically [true]',
    type: 'bool',
    default: true,
})

let commandList = Object.keys(commands)
commandList.sort()
for (let command of commandList) {
    let config = commands[command]
    optParser.command(command,config.docs,(args) => {
        for (let option in config.options) {
            args.options(option,config.options[option])
        }
        for (let arg in config.args) {
            args.positional(arg,config.args[arg])
        }
        args.options('native', {
            describe : 'Run native tests',
            type     : 'bool',
            default  : true
        })
        args.options('wasm', {
            describe : 'Run WASM tests',
            type     : 'bool',
            default  : true
        })
    })
}



// ======================
// === Package Config ===
// ======================

function defaultConfig() {
    return {
        version: "2.0.0-alpha.0",
        author: {
            name: "Enso Team",
            email: "contact@enso.org"
        },
        homepage: "https://github.com/enso-org/ide",
        repository: {
            type: "git",
            url: "git@github.com:enso-org/ide.git"
        },
        bugs: {
            url: "https://github.com/enso-org/ide/issues"
        },
    }
}

async function processPackageConfigs() {
    let files = []
    files = files.concat(glob.sync(paths.js.root + "/package.js", {cwd:paths.root}))
    files = files.concat(glob.sync(paths.js.root + "/lib/*/package.js", {cwd:paths.root}))
    for (file of files) {
        let dirPath = path.dirname(file)
        let outPath = path.join(dirPath,'package.json')
        let src     = await fs.readFile(file,'utf8')
        let modSrc  = `module = {}\n${src}\nreturn module.exports`
        let fn      = new Function('require','paths',modSrc)
        let mod     = fn(require,paths)
        let config  = mod.config
        if (!config) { throw(`Package config '${file}' do not export 'module.config'.`) }
        config = Object.assign(defaultConfig(),config)
        fs.writeFile(outPath,JSON.stringify(config,undefined,4))
    }
}



// ============
// === Main ===
// ============

async function updateBuildVersion (argv) {
    const target =  get_target_platform(argv)
    let config        = {}
    let configPath    = paths.dist.buildInfo
    let exists        = fss.existsSync(configPath)
    if(exists) {
        let configFile = await fs.readFile(configPath)
        config         = JSON.parse(configFile)
    }

    let commitHashCmd = await cmd.run_read('git', [
        'rev-parse',
        '--short',
        'HEAD'
    ])
    let commitHash =  commitHashCmd.trim()

    if (config.buildVersion !== commitHash || config.target !== target){
        config.target = target
        config.buildVersion = commitHash
        await fs.mkdir(paths.dist.root, { recursive: true })
        await fs.writeFile(configPath, JSON.stringify(config, undefined, 2))
    }

}

async function installJsDeps() {
    let initialized = fss.existsSync(paths.dist.init)
    if (!initialized) {
        console.log('Installing application dependencies.')
        await cmd.with_cwd(paths.js.root, async () => {
            await cmd.run('npm',['run','install'])
        })
        await fs.mkdir(paths.dist.root, {recursive:true})
        await fs.open(paths.dist.init,'w')
    }
}

async function runCommand(command,argv) {
    let config = commands[command]
    cargoArgs  = argv['--']
    if (config === undefined) {
        console.error(`Invalid command '${command}'.`)
        return
    }
    if(cargoArgs === undefined) { cargoArgs = [] }
    let index = cargoArgs.indexOf('--')
    if (index == -1) {
        targetArgs = []
    }
    else {
        targetArgs = cargoArgs.slice(index + 1)
        cargoArgs  = cargoArgs.slice(0,index)
    }
    let runner = async function () {
        let do_rust = argv.rust && config.rust
        let do_js   = argv.js   && config.js
        let rustCmd = () => cmd.with_cwd(paths.rust.root, async () => await config.rust(argv))
        let jsCmd   = () => cmd.with_cwd(paths.js.root  , async () => await config.js(argv))
        if(config.parallel) {
            let promises = []
            if (do_rust) { promises.push(rustCmd()) }
            if (do_js)   { promises.push(jsCmd()) }
            await Promise.all(promises)
        } else {
            if (do_rust) { await rustCmd() }
            if (do_js)   { await jsCmd()   }
        }
    }
    cmd.section(command)
    runner()
}

function get_target_platform(argv) {
    let target = argv.target
    if (target === undefined) {
        const local_platform = os.platform()
        switch (local_platform) {
            case 'darwin':
                return 'macos'
            case 'win32':
                return 'win'
            default:
                return local_platform
        }
    }
    return target
}

async function main () {
    let argv = optParser.parse()
    await updateBuildVersion(argv)
    await processPackageConfigs()
    let command = argv._[0]
    if(command === 'clean') {
        try {
            await fs.unlink(paths.dist.init)
            await fs.unlink(paths.dist.buildInit)
        } catch {}
    } else {
        await installJsDeps()
    }

    await runCommand(command,argv)
}

main()
