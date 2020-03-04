let spawn = require('child_process').spawn
let exec = require('child_process').exec

let root = __dirname + '/../..'

process.chdir(root)


async function with_cwd(dir,fn) {
    let cwd = process.cwd()
    process.chdir(dir)
    let out = await fn()
    process.chdir(cwd)
    return out
}

function run(cmd,args) {
    let out = ''
    return new Promise((resolve, reject) => {
        let proc = spawn(cmd,args,{stdio: "inherit"})
        proc.on('exit', () => resolve(out))
    })
}

function run_read(cmd,args) {
    let out = ''
    return new Promise((resolve, reject) => {
        let proc = spawn(cmd,args)
        proc.stderr.pipe(process.stderr)
        proc.stdout.on('data', (data) => { out += data })
        proc.on('exit', () => resolve(out))
    })
}

async function checkVersion (name,required) {
    let version = await run_read(name,['--version'])
    version     = version.trim()
    console.log(`Checking '${name}' version.`)
    if (version != required) {
        throw `[ERROR] The '${name}' version '${version}' does not match the required one '${required}'.`
    }
}

module.exports = {root,run,checkVersion,with_cwd}