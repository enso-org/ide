/// Package release utilities. Especially, utilities to load `CHANGELOG.md`, extract the newest
/// entry, and use it to generate package version and description.

const fss    = require('fs')
const path   = require('path')
const paths  = require('./paths')
const semver = require('semver')



// =================
// === Constants ===
// =================

const CHANGELOG_FILE_NAME = 'CHANGELOG.md'
const CHANGELOG_FILE      = path.join(paths.root,CHANGELOG_FILE_NAME)



// ======================
// === ChangelogEntry ===
// ======================

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



// =================
// === Changelog ===
// =================

class Changelog {
    constructor() {
        this.entries = changelogEntries()
    }

    newestEntry() {
        return this.entries[0]
    }

    currentVersion() {
        return this.newestEntry().version
    }
}

function changelogSections() {
    let text   = '\n' + fss.readFileSync(CHANGELOG_FILE,"utf8")
    let chunks = text.split(/\r?\n# /)
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

function changelog() {
    return new Changelog
}

function currentVersion() {
    return changelog().currentVersion()
}



// ===============
// === Exports ===
// ===============

module.exports = {changelog,currentVersion}
