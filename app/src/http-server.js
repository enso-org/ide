import * as fs    from 'fs'
import * as mime  from 'mime-types'
import * as path  from 'path'
import * as union from 'union'

export class HttpServer {
    constructor(serve_dir, port) {
        this.serve_dir = serve_dir;
        this.port      = port;
        let server     = this;
        this.server    = union.createServer({
            before: [
                function (request, response) {
                    server.process(request.url, response);
                }
            ]
        });

        this.server.listen(this.port);
    }

    process(resource, response) {
        resource   = `${this.serve_dir}${resource}`;
        let server = this;
        fs.readFile(resource, function (err,data) {
            if (err) {
                server.process("/index.html", response);
            } else {
                let contentType   = mime.contentType(path.extname(resource))
                let contentLength = data.length;
                response.setHeader('Content-Type'  , contentType);
                response.setHeader('Content-Length', contentLength);
                response.writeHead(200);
                response.end(data);
            }
        });
    }
}
