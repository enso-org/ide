const assert = require('assert');
const axios = require('axios').default
const fs = require('fs')
const mockFs = require('mock-fs');
const path = require('path');

const { startServer } = require('../server')


describe('Logging Server', function () {

    let server
    let serverUrl

    const dummyMessage = '<Dummy message>'
    const goodConfig = {
        headers: {
            'Content-Type': 'text/plain',
            'Origin': 'http://localhost/'
        }
    }
    const wrongOriginConfig = {
        headers: {
            'Content-Type': 'text/plain',
            'Origin': 'http://attacker/'
        }
    }
    const wrongContentTypeConfig = {
        headers: {
            'Content-Type': 'image/jpeg',
            'Origin': 'http://localhost/'
        }
    }

    beforeEach(function (done) {
        // For some reason, we load a file from this package directory at runtime
        const rawBodyDir = path.dirname(require.resolve('raw-body'))
        mockFs({
            [rawBodyDir]: mockFs.load(rawBodyDir)
        })
        const port = 20060
        server = startServer(port)
        server.on('listening', function () {
            serverUrl = `http://localhost:${server.address().port}/`
            done()
        })
    })

    afterEach(function () {
        server.close()
        mockFs.restore()
    })

    it('should write the body of valid requests to a file', async function () {
        await axios.post(serverUrl, dummyMessage, goodConfig)
        const log_files = fs.readdirSync('log/')
        assert.strictEqual(log_files.length, 1)
        assert.strictEqual(fs.readFileSync(`log/${log_files[0]}`).toString(), dummyMessage)
    })

    it('should reject requests from origins other than localhost', async function () {
        let req = axios.post(serverUrl, '', wrongOriginConfig)
        await assert.rejects(req, 'Error: Request failed with status code 403')
        assert(!fs.existsSync('log/'))
    })

    it('should keep running', async function () {
        await Promise.allSettled([
            axios.post(serverUrl, '', goodConfig),
            axios.post(serverUrl, '', wrongOriginConfig),
            axios.post(serverUrl, '', wrongContentTypeConfig)
        ])

        await axios.post(serverUrl, dummyMessage, goodConfig)
        const log_files = fs.readdirSync('log/')
        assert(log_files.some(file =>
            fs.readFileSync(`log/${file}`).toString() === dummyMessage))
    })
})