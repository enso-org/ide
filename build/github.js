const { Octokit } = require("@octokit/core")

const organization = 'enso-org'
const engineRepo   = 'enso'
const token = process.env.GITHUB_TOKEN
const octokit = new Octokit({ auth: token })

function determineRepositoryName() {
    const fallback = "ide"
    const fallbackMessage =
          "Could not determine the repository name, falling back to the default."
    const fullName = process.env.GITHUB_REPOSITORY;
    if (!fullName) {
        console.log(fallbackMessage)
        return fallback
    }

    const prefix = organization + "/"
    if (fullName.startsWith(prefix)) {
        return fullName.substring(prefix.length)
    } else {
        console.log(fallbackMessage)
        return fallback
    }
}

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
