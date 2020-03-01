import * as createServers from 'create-servers'
import * as fs            from 'fs'
import * as mime          from 'mime-types'
import * as path          from 'path'



// ==============
// === Server ===
// ==============

/// A simple server implementation.
///
/// Initially it was based on `union`, but later we migrated to `create-servers`. Read
/// this topic to learn why: https://github.com/http-party/http-server/issues/483
class Server {
    constructor(cfg) {
        let self    = this
        this.cfg    = cfg
        this.server = createServers({
            http: cfg.port,
            handler: function (request, response) {
                self.process(request.url, response)
            }
        },
        function (errs) {
            if (errs) { return console.log(errs.http) }
            console.log(`Server started. Listening on port ${cfg.port}.`)
        })
    }

    process(resource,response) {
        let resource_file = `${this.cfg.dir}${resource}`
        fs.readFile(resource_file, function (err,data) {
            if(err) {
                let fallback = this.cfg.fallback
                if(fallback) {
                    if(resource === fallback) {
                        console.error(`Fallback resource '${resource}' not found.`)
                    } else {
                        this.process(fallback,response)
                    }
                } else {
                    console.error(`Resource '${resource}' not found.`)
                }
            } else {
                let contentType   = mime.contentType(path.extname(resource_file))
                let contentLength = data.length
                response.setHeader('Content-Type'  , contentType)
                response.setHeader('Content-Length', contentLength)
                response.writeHead(200)
                response.end(data)
            }
        }.bind(this))
    }
}

export function create(...args) {
    return new Server(...args)
}
