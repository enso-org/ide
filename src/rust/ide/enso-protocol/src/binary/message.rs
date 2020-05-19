//! Module defining types representing messages being sent between client and server.

use crate::prelude::*;

use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::OutboundMessage;

use flatbuffers::FlatBufferBuilder;
use json_rpc::Transport;
use crate::binary::serialization;

use crate::language_server::Path as LSPath;



// ===============
// === Aliases ===
// ===============

/// An owning representation of the message received from a server.
pub type MessageFromServerOwned = Message<FromServerPayloadOwned>;

/// An non-owning representation of the message to be sent to the server.
pub type MessageToServerRef<'a> = Message<ToServerPayload<'a>>;



// =============
// === Types ===
// =============

/// Identifies the visualization.
#[allow(missing_docs)]
#[derive(Clone,Debug,Copy,PartialEq)]
pub struct VisualisationContext {
    pub visualization_id : Uuid,
    pub context_id       : Uuid,
    pub expression_id    : Uuid,
}



// ================
// === Payloads ===
// ================

#[allow(missing_docs)]
#[derive(Clone,Debug,PartialEq)]
pub enum ToServerPayloadOwned {
    InitSession {client_id:Uuid},
    WriteFile   {path:LSPath, contents:Vec<u8>},
    ReadFile    {path:LSPath}
}

#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub enum FromServerPayloadOwned {
    Error {code:i32, message:String},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:Vec<u8>},
    FileContentsReply   {contents:Vec<u8>},
}

#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub enum ToServerPayload<'a> {
    InitSession {client_id:Uuid},
    WriteFile   {path:&'a LSPath, contents:&'a[u8]},
    ReadFile    {path:&'a LSPath}
}

#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub enum FromServerPayload<'a> {
    Error {code:i32, message:&'a str},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:&'a [u8]},
    FileContentsReply {contents:&'a [u8]},
}



// ===============
// === Message ===
// ===============

/// Common message envelope for binary protocol.
///
/// `T` should represent the payload.
#[derive(Clone,Debug)]
pub struct Message<T> {
    /// Each message bears unique id.
    pub message_id     : Uuid,
    /// When sending reply, server sets this to the request's `message_id`.
    pub correlation_id : Option<Uuid>,
    #[allow(missing_docs)]
    pub payload        : T,
}

impl<T> Message<T> {
    /// Wraps the given payload into a message envelope. Generates a unique ID for the message.
    pub fn new(payload:T) -> Message<T> {
        Message {
            message_id     : Uuid::new_v4(),
            correlation_id : None,
            payload,
        }
    }
}

impl<'a> crate::handler::IsRequest for MessageToServerRef<'a> {
    type Id = Uuid;

    fn send(&self, transport:&mut dyn Transport) -> FallibleResult<()> {
        self.with_serialized(|data| transport.send_binary(data))
    }

    fn id(&self) -> Self::Id {
        self.message_id
    }
}

impl Message<FromServerPayloadOwned> {
    /// Deserializes a message from server from a binary blob.
    pub fn deserialize_owned(data:&[u8]) -> FallibleResult<Self> {
        let message = flatbuffers::get_root::<OutboundMessage>(data);
        let payload = FromServerPayloadOwned::deserialize_owned(&message)?;

        let enso = message.correlationId();
        let uuid = enso.map(Uuid::from);
        println!("From {:?} we got {:?}", enso, uuid);
        Ok(Message {
            message_id     : message.messageId().into(),
            correlation_id : message.correlationId().map(|id| id.into()),
            payload
        })
    }
}

/// Entity that can be serialized into a binary blob using our FlatBuffer schema.
pub trait Serialize {
    /// Stores the entity into the builder and calls `finish` on it.
    fn write(&self, builder:&mut FlatBufferBuilder);

    /// Returns `finish`ed builder with the serialized entity.
    fn serialize(&self) -> FlatBufferBuilder {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        self.write(&mut builder);
        builder
    }

    /// Calls the given function with the binary blob with the serialized entity.
    fn with_serialized<R>(&self, f:impl FnOnce(&[u8]) -> R) -> R {
        let buffer = self.serialize();
        f(buffer.finished_data())
    }
}

impl<T: serialization::Serializable> Serialize for Message<T> {
    fn write(&self, builder:&mut FlatBufferBuilder) {
        self.payload.write_message(builder,self.correlation_id,self.message_id)
    }
}
