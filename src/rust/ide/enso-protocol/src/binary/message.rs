
use crate::prelude::*;

use crate::binary::FromServerOwned;
use crate::binary::IsPayloadToServer;
use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::OutboundMessage;

use flatbuffers::FlatBufferBuilder;



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

impl Message<FromServerOwned> {
    pub fn deserialize_owned(data:&[u8]) -> Self {
        let message = flatbuffers::get_root::<OutboundMessage>(data);
        let payload = FromServerOwned::deserialize(&message);

        let enso = message.correlationId();
        let uuid = enso.map(Uuid::from);
        println!("From {:?} we got {:?}", enso, uuid);
        Message {
            message_id     : message.messageId().into(),
            correlation_id : message.correlationId().map(|id| id.into()),
            payload
        }
    }
}
