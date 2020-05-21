const cmd    = require('./cmd')
const fs     = require('fs').promises
const fss    = require('fs')
const glob   = require('glob')
const ncp    = require('ncp').ncp
const path   = require('path')
const paths  = require('./paths')
const stream = require('stream');
const yargs  = require('yargs')
const zlib   = require('zlib');

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



// ================
// === Commands ===
// ================

let commands = {}


// === Clean ===

commands.clean = command(`Clean all build artifacts`)
commands.clean.js = async function() {
    await cmd.with_cwd(paths.js.root, async () => {
        await run('npm',['run','clean'])
    })
    try { await fs.unlink(paths.dist.init) } catch {}
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
commands.build.js = async function() {
    console.log(`Building JS target.`)
    await run('npm',['run','build'])
}

commands.build.rust = async function(argv) {
    console.log(`Building WASM target.`)
    let args = ['build','--target','web','--no-typescript','--out-dir',paths.dist.wasm.root,'lib/debug-scenes']
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
        let limit = 3.1
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
    console.log(`Patching ${path}`)
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

commands.start.js = async function() {
    console.log(`Building JS target.`)
    await cmd.with_cwd(paths.js.root, async () => {
        await run('npm',['run','start','--'].concat(targetArgs))
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


// === Watch ===

commands.watch = command(`Start a file-watch utility and run interactive mode`)
commands.watch.parallel = true
commands.watch.rust = async function() {
    let target = '"' + `node ${paths.script.main} build --no-js --dev -- ` + cargoArgs.join(" ") + '"'
    let args   = ['watch','-s',`${target}`]
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

let commandList = Object.keys(commands)
commandList.sort()
for (let command of commandList) {
    let config = commands[command]
    optParser.command(command,config.docs,(args) => {
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
            email: "contact@luna-lang.org"
        },
        homepage: "https://github.com/luna/ide",
        repository: {
            type: "git",
            url: "git@github.com:luna/ide.git"
        },
        bugs: {
            url: "https://github.com/luna/ide/issues"
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

async function updateBuildVersion () {
    let config        = {}
    let configPath    = paths.dist.buildInfo
    let exists        = fss.existsSync(configPath)
    if(exists) {
        let configFile = await fs.readFile(configPath)
        config         = JSON.parse(configFile)
    }
    let commitHashCmd = await cmd.run_read('git',['rev-parse','--short','HEAD'])
    let commitHash    = commitHashCmd.trim()
    if (config.buildVersion != commitHash) {
        config.buildVersion = commitHash
        await fs.mkdir(paths.dist.root,{recursive:true})
        await fs.writeFile(configPath,JSON.stringify(config,undefined,2))
    }
}

async function installJsDeps() {
    let initialized = fss.existsSync(paths.dist.init)
    if (!initialized) {
        console.log('Installing application dependencies')
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

async function main () {
    await processPackageConfigs()
    updateBuildVersion()
    let argv    = optParser.parse()
    let command = argv._[0]
    if(command == 'clean') {
        try { await fs.unlink(paths.dist.init) } catch {}
    } else {
        await installJsDeps()
    }
    await runCommand(command,argv)
}

main()
