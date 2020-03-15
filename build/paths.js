const path  = require('path')



// =============
// === Paths ===
// =============

let paths  = {}

paths.root           = path.dirname(__dirname)

paths.script         = {}
paths.script.start   = path.join(paths.root,'run')
paths.script.root    = path.join(paths.root,'build')
paths.script.run     = path.join(paths.script.root,'run')

paths.dist           = {}
paths.dist.root      = path.join(paths.root,'dist')
paths.dist.client    = path.join(paths.dist.root,'client')
paths.dist.content   = path.join(paths.dist.root,'content')
paths.dist.init      = path.join(paths.dist.root,'init')
paths.dist.buildInfo = path.join(paths.dist.root,'build.json')

paths.js             = {}
paths.js.root        = path.join(paths.root,'src','js')

paths.rust           = {}
paths.rust.root      = path.join(paths.root,'src','rust')
paths.rust.wasmDist  = path.join(paths.dist.root,'wasm')


module.exports = paths
