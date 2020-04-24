Enso protocol consists of pair server and client. They communicate by exchanging
messages, following JSON-RPC 2.0.

# Setup
Follow the guidelines of [Enso repository](https://github.com/luna/enso) to setup the project
. Once you have all the requirements configured, you are able to run the project manager service
 with the command bellow:
 
luna/enso$ `sbt -java-home $JAVA_HOME -J-Xss10M project-manager/run`
 
 Where `$JAVA_HOME` is the path where `graalvm-ce-java8-20.0.0` is located.

# Protocol
An up-to-date and complete list of possible operations can be found in the [enso protocol specification document](https://github.com/luna/enso/blob/master/doc/language-server/specification/enso-protocol.md).
