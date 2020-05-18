
use crate::prelude::*;

use crate::binary::payload::FromServerOwned;
use crate::binary::payload::IsPayloadToServer;
use crate::binary::payload::ToServerPayload;
use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::OutboundMessage;

use flatbuffers::FlatBufferBuilder;
use json_rpc::Transport;


/// Common message envelope for binary protocol.
///
/// `T` should represent the payload.
#[derive(Clone,Debug)]
pub struct Message<T> {
    pub message_id     : Uuid,
    pub correlation_id : Option<Uuid>,
    pub payload        : T,
}

impl<T> Message<T> {
    pub fn new(payload:T) -> Message<T> {
        Message {
            message_id     : Uuid::new_v4(),
            correlation_id : None,
            payload,
        }
    }
}

/// When payload supports serialization, we can serialize the whole message.
impl<T: IsPayloadToServer> Message<T> {
    pub fn write_message(&self, builder:&mut FlatBufferBuilder) {
        self.payload.write_message(builder,self.message_id,self.correlation_id)
    }

    pub fn build_buffer(&self) -> FlatBufferBuilder {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        self.write_message(&mut builder);
        builder
    }

    pub fn with_serialized<R>(&self, f:impl FnOnce(&[u8]) -> R) -> R {
        let buffer = self.build_buffer();
        let data = buffer.finished_data();
        f(data)
    }
}

pub type MessageFromServerOwned = Message<FromServerOwned>;

pub type MessageToServer<'a> = Message<ToServerPayload<'a>>;

impl<'a> crate::new_handler::MessageToServer for MessageToServer<'a> {
    type Id = Uuid;

    fn send(&self, transport:&mut dyn Transport) -> FallibleResult<()> {
        self.with_serialized(|data| transport.send_binary(data))
    }

    fn id(&self) -> Self::Id {
        self.message_id
    }
}

impl Message<FromServerOwned> {
    pub fn deserialize_owned(data:&[u8]) -> FallibleResult<Self> {
        let message = flatbuffers::get_root::<OutboundMessage>(data);
        let payload = FromServerOwned::deserialize_owned(&message)?;

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

pub trait Serialize {
    fn write(&self, builder:&mut FlatBufferBuilder);

    fn serialize(&self) -> FlatBufferBuilder {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        self.write(&mut builder);
        builder
    }

    fn with_serialized<R>(&self, f:impl FnOnce(&[u8]) -> R) -> R {
        let buffer = self.serialize();
        f(buffer.finished_data())
    }
}

impl<T: IsPayloadToServer> Serialize for Message<T> {
    fn write(&self, builder:&mut FlatBufferBuilder) {
        self.payload.write_message(builder,self.message_id,self.correlation_id)
    }
}
