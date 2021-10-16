const { Octokit } = require("@octokit/core")

const organization = 'enso-org'
const engineRepo   = 'enso'
const token = process.env.GITHUB_TOKEN
const octokit = new Octokit({ auth: token })

function isNightly(release) {
    const nightlyInfix = "Nightly"
    return release.name.indexOf(nightlyInfix) >= 0 && !release.draft
}

async function fetchAllReleases(repo) {
    const res = await octokit.request("GET /repos/{owner}/{repo}/releases", {
        owner: organization,
        repo: repo,
    })
    return res.data
}

async function fetchNightlies(repo) {
    const releases = await fetchAllReleases(repo)
    const nightlies = releases.filter(isNightly)
    return nightlies
}

async function fetchEngineNightlies() {
    return await fetchNightlies(engineRepo)
}

module.exports = { fetchEngineNightlies }
