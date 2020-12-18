const fss = require('fs')
const fs = fss.promises

const tar = require('tar')
const os = require('os')
const path = require('path')
const unzipper = require('unzipper')
const url = require('url')
const { http } = require('follow-redirects')

const thisPath = path.resolve(__dirname)
const root = path.resolve(thisPath, '..', '..', '..', '..', '..')
const distPath = path.resolve(root, 'dist', 'bin')
const buildInfoPath = path.resolve(root, 'dist', 'build.json')

async function get_build_config() {
    let exists = fss.existsSync(buildInfoPath)
    if (exists) {
        let configFile = await fs.readFile(buildInfoPath)
        return JSON.parse(configFile)
    }
}

async function get_target_url(): Promise<string> {
    const config = await get_build_config()
    const target_platform = config.target
    console.log('webpack target ' + target_platform)
    const version = '0.1.2-rc.18'
    const base_url = `https://github.com/enso-org/enso-staging/releases/download/enso-${version}/enso-project-manager-${version}`
    switch (target_platform) {
        case 'linux':
            return `${base_url}-linux-amd64.tar.gz`
        case 'macos':
            return `${base_url}-macos-amd64.tar.gz`
        case 'win':
            return `${base_url}-windows-amd64.zip`
        default:
            throw 'UnsupportedPlatform: ' + target_platform
    }
}

// TODO[MM] remove duplicate version of this method
function project_manager_path() {
    let base_path = path.join(distPath, 'enso', 'bin')
    const target_platform = os.platform()
    switch (target_platform) {
        case 'linux':
            return path.join(base_path, 'project-manager')
        case 'darwin':
            return path.join(base_path, 'project-manager')
        case 'win32':
            return path.join(base_path, 'project-manager.exe')
        default:
            throw 'UnsupportedPlatform: ' + target_platform
    }
}

function decompress_project_manager(source_file_path, target_folder) {
    let decompressor
    if (source_file_path.toString().endsWith('.zip')) {
        decompressor = unzipper.Extract({ path: target_folder })
    } else {
        decompressor = tar.x({
            strip: 1,
            C: target_folder,
        })
    }
    fss.createReadStream(source_file_path)
        .pipe(decompressor)
        .on('finish', () => {
            const bin_path = project_manager_path()
            fss.chmodSync(bin_path, '744')
        })
}

async function download_file_http(
    file_url: string,
    overwrite: boolean
): Promise<void> {
    const file_name = url.parse(file_url).pathname.split('/').pop()
    const file_path = path.resolve(distPath, file_name)

    if (fss.existsSync(file_path) && !overwrite) {
        console.log(
            `The ${file_path} file exists. Project manager executable will not be regenerated.`
        )
        return
    }

    await fs.mkdir(distPath, { recursive: true })

    const parsed = url.parse(file_url)
    const options = {
        host: parsed.host,
        port: 80,
        path: parsed.pathname,
    }

    const target_file = fss.createWriteStream(file_path)
    http.get(options, (res) => {
        res.on('data', (data) => {
            target_file.write(data)
        }).on('end', () => {
            target_file.end()
            console.log(file_url + ' downloaded to ' + file_path)
            decompress_project_manager(file_path, distPath)
        })
    })
}

async function main() {
    let file_url = await get_target_url()
    await download_file_http(file_url, true)
}

main().then((r) => console.log(r))
