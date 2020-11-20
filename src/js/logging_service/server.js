const express = require('express')
const fs = require('fs')
const uuid = require('uuid')
const yargs = require('yargs')


module.exports = {
    startServer
  }


function main(argv) {
    startServer(parse_args(argv).port)
}


function parse_args(argv) {
    return yargs(argv)
        .option('port', {
            alias: 'p',
            description:
                'The number of the port that this server will listen on. ' +
                'If the the number is 0 then an arbitrary free port will be chosen.',
            type: 'number',
            default: 20060
        })
        .help()
        .alias('help', 'h')
        .argv
}


function startServer(port) {
    const app = express()
    app.use(express.text())

    app.post("/", async (req, res) => {
        if (typeof req.headers.origin === 'undefined' ||
                (new URL(req.headers.origin).hostname) !== 'localhost') {
            res.sendStatus(403)  // Forbidden
        } else if (typeof req.body !== 'string') {
            res.sendStatus(400)  // Bad request
        } else {
            try {
                await writeLog(req.body)
                console.log(`Saved log from origin ${req.headers.origin}`)
                res.sendStatus(204)  // No content (But request was successful)
            } catch (e) {
                console.error(
                    'Could not write log file:\n' +
                    e.message)
                res.sendStatus(500)  // Internal Server Error
            }
        }
    })

    const server = app.listen(port, 'localhost')
    server.on('listening', function () {
        console.log(`Logging service listening at port ${server.address().port}`)
    })
    return server
}


/**
 * Writes message to a new file in the log sub directory.
 * The file name is composed of the UTC time and date and a V4 UUID to guarantee uniqueness.
 */
async function writeLog(message) {
    const dir = 'log'
    const file = `${timestamp()}__${uuid.v4()}`
    await fs.promises.mkdir(dir, { recursive: true })
    await fs.promises.writeFile(`${dir}/${file}`, message)
}


/**
 * Returns the current UTC date and time in the format "yyy-MM-dd_HH:mm:ss.".
 */
function timestamp() {
    const d = new Date()

    const year = d.getUTCFullYear().toString()
    const month = d.getUTCMonth().toString().padStart(2, "0")
    const day = d.getUTCDate().toString().padStart(2, "0")

    const hour = d.getUTCHours().toString().padStart(2, "0")
    const minute = d.getUTCMinutes().toString().padStart(2, "0")
    const second = d.getUTCSeconds().toString().padStart(2, "0")

    return `${year}-${month}-${day}_${hour}:${minute}:${second}`
}


if (require.main === module) {
    const command_line_args = process.argv.slice(2)
    main(command_line_args)
}
