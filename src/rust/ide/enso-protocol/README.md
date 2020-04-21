Enso protocol consists of pair server and client. They communicate by exchanging
messages, following JSON-RPC 2.0.

# Setup
Establish websocket connection. 

In future it is expected that some kind of authorization will be required by the
server. As of now, its details remain unspecified.

# General protocol
Remote calls made between the client and server follow [JSON-RPC 2.0
protocol](https://www.jsonrpc.org/specification).

There are two primary cases:
* RPC calls from client to server methods;
* Notifications sent from server to client.

All messages are text with JSON-encoded values.

The server accepts only method calls (request objects).

The server responds with call results and may send notifications.

# JSON Encoding
Struct values are serialized as map, e.g. `{ "field_name" : field_value }`.

Enum values are serialized as map `{ "variant_name" : variant_value }` or just
`"variant_name"` if there variants have no inner value.
Transitive enums (i.e. enums of enums) are flattened, no intermediate variant
names shall appear. 

`()` (unit value) is serialized as `null`.

`FileTime` value is serialized as a string compliant with RFC3339 / ISO8601 text
format, e.g. `"2020-01-07T21:25:26Z"`.

`Path` is serialized as JSON string value, e.g. `"./Main.luna"`.

`UUID` is serialzied as string using 8-4-4-4-12 format, e.g.
`"02723954-fbb0-4641-af53-cec0883f260a"`.

`u64` is an unsigned 64-bit integer value.

# Protocol
An up-to-date and complete list of possible operations can be found in the [enso protocol specification document](https://github.com/luna/enso/blob/master/doc/language-server/specification/enso-protocol.md).

## Examples

### Call to `exists` method
#### Request (call)
```json
{
    "jsonrpc" : "2.0",
    "id"      : 0,
    "method"  : "exists",
    "params"  : { "path" : "./Main.luna" }
}
```
#### Response
```json
{
    "jsonrpc" : "2.0",
    "id"      : 0,
    "result"  : true
}
```

### Filesystem Event Notification
#### Request (notification)
```json
{
    "jsonrpc" : "2.0",
    "method"  : "filesystemEvent",
    "params"  : { "path" : "./Main.luna", "kind" : "Modified" }
}
```

Notification requests gets no response.


### `Attributes` structure
`Attributes` value may be serialized to a following JSON:
```json
{
    "creationTime"     : "2020-01-07T21:25:26Z",
    "lastAccessTime"   : "2020-01-21T22:16:51.123994500+00:00",
    "lastModifiedTime" : "2020-01-07T21:25:26Z",
    "fileKind"         : "RegularFile",
    "sizeInBytes"      : 125125
}
```
