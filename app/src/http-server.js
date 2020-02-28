import * as fs    from 'fs'
import * as mime  from 'mime-types'
import * as path  from 'path'
import * as union from 'union'

class HttpServer {
    constructor(cfg) {
        this.cfg    = cfg
        this.server = union.createServer({
            before: [
                function (request, response) {
                    this.process(request.url, response)
                }.bind(this)
            ]
        })
        this.server.listen(this.cfg.port)
        console.log(`Server started. Listening on port ${this.port}.`)
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
    return new HttpServer(...args)
}
