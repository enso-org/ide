Enso protocol consists of pair server and client. They communicate by exchanging
messages, following JSON-RPC 2.0.

# Setup
Follow the guidelines of [Enso repository](https://github.com/luna/enso) to setup the project
. Once you have all the requirements configured, you are able to run the project manager service
 with the command bellow:
 
luna/enso$ `sbt -java-home $JAVA_HOME -J-Xss10M project-manager/run`
 
 Where `$JAVA_HOME` is the path where `graalvm-ce-java8-20.0.0` is located.

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
