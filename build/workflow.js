/// GitHub workflow generator. We are using custom generator in order to re-use sub-steps. This
/// may not be needed anymore after this feature request will be resolved by GitHub:
/// https://github.community/t/support-for-yaml-anchors/16128

const fss     = require('fs')
const path    = require('path')
const paths   = require('./paths')
const release = require('./release')
const yaml    = require('js-yaml')



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



// ==================
// === Validators ===
// ==================

let assertVersionUnstable = {
    name: "Assert Verstion Unstable",
    run: "node ./run assert-version-unstable --skip-version-validation",
}

let assertVersionStable = {
    name: "Assert Verstion Stable",
    run: "node ./run assert-version-stable --skip-version-validation",
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
    "continue-on-error": true,
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

function uploadBinArtifactsFor(name,sys,ext,os) {
    return {
        name: `Upload Artifacts (${name}, ${ext})`,
        uses: "actions/upload-artifact@v1",
        with: {
           name: `enso-${os}-${release.currentVersion()}.${ext}`,
           path: `dist/client/enso-${os}-${release.currentVersion()}.${ext}`
        },
        if: `matrix.os == '${sys}-latest'`
    }
}

uploadBinArtifactsForMacOS   = uploadBinArtifactsFor('Linux','ubuntu','AppImage','linux')
uploadBinArtifactsForWindows = uploadBinArtifactsFor('Windows','windows','exe','win')
uploadBinArtifactsForLinux   = uploadBinArtifactsFor('macOS','macos','dmg','mac')

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
    name: 'Read changelog info',
    id: 'changelog',
    run: `
        node ./run ci-gen --skip-version-validation
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
        files:      "artifacts/**/enso-*",
        name:       "Enso ${{fromJson(steps.changelog.outputs.content).version}}",
        tag_name:   "v${{fromJson(steps.changelog.outputs.content).version}}",
        body:       "${{fromJson(steps.changelog.outputs.content).body}}",
        prerelease: "${{fromJson(steps.changelog.outputs.content).prerelease}}",
    },
}



// ===================
// === CDN Release ===
// ===================

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

function uploadToCDN(...names) {
    let actions = []
    for (let name of names) {
        let action = {
            name: `Upload '${name}' to CDN`,
            shell: "bash",
            run: `
                aws s3 cp ./artifacts/content/assets/${name}
                s3://ensocdn/ide/${release.currentVersion()}/${name} --profile
                s3-upload --acl public-read --content-encoding gzip
                `
        }
        actions.push(action)
    }
    return actions
}



// ================
// === Workflow ===
// ================

let releaseCondition = `github.ref == 'refs/heads/unstable' || github.ref == 'refs/heads/stable'`
let buildCondition   = `github.ref == 'refs/heads/main' || github.ref == 'refs/heads/develop' || ${releaseCondition}`

let workflow = {
    name : "GUI CI",
    on: ['push'],
    jobs: {
        assert_version_unstable: job_on_macos("Unstable Version Assertion", [
            assertVersionUnstable
        ],{if:`github.ref == 'refs/heads/unstable'`}),
        assert_version_stable: job_on_macos("Stable Version Assertion", [
            assertVersionStable
        ],{if:`github.ref == 'refs/heads/stable'`}),
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
        ],{if:buildCondition}),
        release_to_github: job_on_macos("GitHub Release", [
            downloadArtifacts,
            getCurrentReleaseChangelogInfo,
            uploadGitHubRelease,
        ],{ if:releaseCondition,
            needs:["assert_version_unstable","assert_version_stable","lint","test","wasm-test","build"]
        }),
// FIXME: Commented until CDN gets fixed
//        release_to_cdn: job_on_macos("CDN Release", [
//            downloadArtifacts,
//            prepareAwsSessionCDN,
//            uploadToCDN('index.js.gz','style.css','ide.wasm','wasm_imports.js.gz'),
//        ],{ if:releaseCondition,
//            needs:["assert_version_unstable","assert_version_stable","lint","test","wasm-test","build"]
//        }),
    }
}

let header = `
# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
# DO NOT CHANGE THIS FILE. IT WAS GENERATED FROM 'workflows.js'. READ DOCS THERE TO LEARN MORE.
# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
`

/// Generates a new GitHub workflow file (in .github/workflow/...).
function generate() {
    let workflow_script = header + '\n' + yaml.dump(workflow,{noRefs:true})
    fss.writeFileSync(path.join(paths.github.workflows,'gui-ci.yml'),workflow_script)
}



// ===============
// === Exports ===
// ===============

module.exports = {generate}
