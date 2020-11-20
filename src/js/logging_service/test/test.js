const assert = require('assert');
const axios = require('axios').default
const fs = require('fs')
const mockFs = require('mock-fs');

const { startServer } = require('../server')


describe('Logging Server', function () {

    let server

    beforeEach(function (done) {
        mockFs({
            'node_modules': mockFs.load('node_modules')
        })
        const port = 20060
        server = startServer(port)
        server.on('listening', done)
    })

    afterEach(function () {
        server.close()
        mockFs.restore()
    })

    it('should write the body of valid requests to a file', async function () {
        const message = 'This is a crash report.'
        await axios.post(`http://localhost:${server.address().port}/`,
            message,
            { headers: { 'content-type': 'text/plain' }})
        const log_files = fs.readdirSync('log/')
        assert.strictEqual(log_files.length, 1)
        assert.strictEqual(fs.readFileSync(`log/${log_files[0]}`).toString(), message)
    })
})