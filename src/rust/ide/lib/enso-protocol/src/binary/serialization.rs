//! Code for converting between FlatBuffer-generated wrappers and our representation of the protocol
//! messages and their parts.

use crate::prelude::*;

use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::*;

use crate::common::error::DeserializationError;
use crate::binary::message;
use crate::binary::message::Message;
use crate::binary::message::MessageToServer;
use crate::binary::message::MessageFromServer;
use crate::binary::message::FromServerPayloadOwned;
use crate::binary::message::FromServerPayload;
use crate::binary::message::ToServerPayload;
use crate::binary::message::ToServerPayloadOwned;
use crate::language_server::types::Path as LSPath;

use flatbuffers::FlatBufferBuilder;
use flatbuffers::UnionWIPOffset;
use flatbuffers::WIPOffset;



// =================
// === constants ===
// =================

/// The initial buffer size used when serializing binary message.
/// Should be large enough to fit most of the messages we send, while staying possibly small.
pub const INITIAL_BUFFER_SIZE:usize = 256;



// ==========================
// === SerializableObject ===
// ==========================

// === Trait ===

/// All entities that can be serialized to the FlatBuffers and represented as offsets.
/// That includes tables and vectors, but not primitives, structs nor unions.
///
/// Supports both serialization and deserialization.
trait SerializableDeserializableObject<'a> : Sized {
    /// The FlatBuffer's generated type for this type representation.
    type Out : Sized;

    /// Writes this table to the buffer and returns its handle.
    fn serialize(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out>;

    /// Instantiates Self and reads the data from the FlatBuffers representation.
    fn deserialize(fbs:Self::Out) -> Result<Self,DeserializationError>;

    /// Instantiates Self and reads the data from the optional FlatBuffers representation.
    /// Will fail always if the representation is not present.
    fn deserialize_required_opt(fbs:Option<Self::Out>) -> Result<Self, DeserializationError>{
        let missing_expected = || DeserializationError("Missing expected field".to_string());
        Self::deserialize(fbs.ok_or_else(missing_expected)?)
    }
}


// === impl Vec<String> ===

impl<'a> SerializableDeserializableObject<'a> for Vec<String> {
    type Out = flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<&'a str>>;

    fn serialize(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out> {
        let strs = self.iter().map(|s| s.as_str()).collect_vec();
        builder.create_vector_of_strings(&strs)
    }

    fn deserialize(fbs: Self::Out) -> Result<Self, DeserializationError> {
        let indices = 0..fbs.len();
        Ok(indices.map(|ix| fbs.get(ix).to_string()).collect())
    }
}


// === impl VisualisationContext ===

impl<'a> SerializableDeserializableObject<'a> for message::VisualisationContext {
    type Out = VisualisationContext<'a>;
    fn serialize(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out> {
        VisualisationContext::create(builder, &VisualisationContextArgs {
            visualisationId : Some(&self.visualization_id.into()),
            expressionId    : Some(&self.expression_id.into()),
            contextId       : Some(&self.context_id.into()),
        })
    }

    fn deserialize(fbs:Self::Out) -> Result<Self,DeserializationError> {
        Ok(message::VisualisationContext {
            context_id       : fbs.contextId().into(),
            visualization_id : fbs.visualisationId().into(),
            expression_id    : fbs.expressionId().into(),
        })
    }
}


// === impl language server's Path ===

impl<'a> SerializableDeserializableObject<'a> for LSPath {
    type Out = Path<'a>;
    fn serialize(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out> {
        let root_id      = self.root_id.into();
        let segments     = Vec::serialize(&self.segments, builder);
        Path::create(builder, &PathArgs {
            rootId   : Some(&root_id),
            segments : Some(segments),
        })
    }

    fn deserialize(fbs:Self::Out) -> Result<Self,DeserializationError> {
        let missing_root_id = || DeserializationError("Missing root ID".to_string());
        let root_id         = Uuid::from(fbs.rootId().ok_or_else(missing_root_id)?);
        let segments        = Vec::deserialize_required_opt(fbs.segments())?;
        Ok(LSPath {root_id,segments})
    }
}



// =========================
// === SerializableUnion ===
// =========================

/// Traits for serialization of our types that flatbuffers schema represents as unions.
pub trait SerializableUnion : Sized {
    /// Type of the FlatBuffers-generated enumeration with the variant index.
    type EnumType;

    /// Write the enumeration to the builder and return the handle.
    fn serialize(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset>;

    /// Obtain the index of the active variant.
    fn active_variant(&self) -> Self::EnumType;
}

impl<'a> SerializableUnion for ToServerPayload<'a> {
    type EnumType = InboundPayload;
    fn serialize(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            ToServerPayload::InitSession {client_id} => {
                InitSessionCommand::create(builder, &InitSessionCommandArgs {
                    identifier : Some(&client_id.into())
                }).as_union_value()
            }
            ToServerPayload::WriteFile {path,contents} => {
                let path     = path.serialize(builder);
                let contents = builder.create_vector(contents);
                WriteFileCommand::create(builder, &WriteFileCommandArgs {
                    path     : Some(path),
                    contents : Some(contents),
                }).as_union_value()
            }
            ToServerPayload::ReadFile {path} => {
                let path = path.serialize(builder);
                ReadFileCommand::create(builder, &ReadFileCommandArgs {
                    path : Some(path)
                }).as_union_value()
            }
        }
    }

    fn active_variant(&self) -> Self::EnumType {
        match self {
            ToServerPayload::InitSession {..} => InboundPayload::INIT_SESSION_CMD,
            ToServerPayload::WriteFile   {..} => InboundPayload::WRITE_FILE_CMD,
            ToServerPayload::ReadFile    {..} => InboundPayload::READ_FILE_CMD,
        }
    }
}

impl SerializableUnion for ToServerPayloadOwned {
    type EnumType = InboundPayload;
    fn serialize(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            ToServerPayloadOwned::InitSession {client_id} => {
                InitSessionCommand::create(builder, &InitSessionCommandArgs {
                    identifier : Some(&client_id.into())
                }).as_union_value()
            }
            ToServerPayloadOwned::WriteFile {path,contents} => {
                let path     = path.serialize(builder);
                let contents = builder.create_vector(contents);
                WriteFileCommand::create(builder, &WriteFileCommandArgs {
                    path     : Some(path),
                    contents : Some(contents),
                }).as_union_value()
            }
            ToServerPayloadOwned::ReadFile {path} => {
                let path = path.serialize(builder);
                ReadFileCommand::create(builder, &ReadFileCommandArgs {
                    path : Some(path)
                }).as_union_value()
            }
        }
    }

    fn active_variant(&self) -> Self::EnumType {
        match self {
            ToServerPayloadOwned::InitSession {..} => InboundPayload::INIT_SESSION_CMD,
            ToServerPayloadOwned::WriteFile   {..} => InboundPayload::WRITE_FILE_CMD,
            ToServerPayloadOwned::ReadFile    {..} => InboundPayload::READ_FILE_CMD,
        }
    }
}

impl SerializableUnion for FromServerPayloadOwned {
    type EnumType = OutboundPayload;

    fn serialize(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            FromServerPayloadOwned::Success {} => {
                Success::create(builder, &SuccessArgs {}).as_union_value()
            }
            FromServerPayloadOwned::Error {code,message} => {
                let message = builder.create_string(&message);
                Error::create(builder, &ErrorArgs {
                    code    : *code,
                    message : Some(message),
                }).as_union_value()
            }
            FromServerPayloadOwned::FileContentsReply {contents} => {
                let contents = builder.create_vector(&contents);
                FileContentsReply::create(builder, &FileContentsReplyArgs {
                    contents : Some(contents)
                }).as_union_value()
            }
            FromServerPayloadOwned::VisualizationUpdate {data,context} => {
                let data    = builder.create_vector(&data);
                let context = context.serialize(builder);
                VisualisationUpdate::create(builder, &VisualisationUpdateArgs {
                    data                 : Some(data),
                    visualisationContext : Some(context),
                }).as_union_value()
            }
        }
    }

    fn active_variant(&self) -> Self::EnumType {
        match self {
            FromServerPayloadOwned::Error {..}               => OutboundPayload::ERROR,
            FromServerPayloadOwned::Success {..}             => OutboundPayload::SUCCESS,
            FromServerPayloadOwned::FileContentsReply {..}   => OutboundPayload::FILE_CONTENTS_REPLY,
            FromServerPayloadOwned::VisualizationUpdate {..} => OutboundPayload::VISUALISATION_UPDATE,
        }
    }
}



// ================================
// === DeserializableUnionField ===
// ================================

/// Unfortunately the FlatBuffers generated code includes union accessors in the parent type, so
/// we cannot generalize union field deserialization apart from the parent type.
///
/// `ParentType` should be a FlatBuffer-generated type that contains this union field.
pub trait DeserializableUnionField<'a, ParentType:'a> : Sized {
    /// Constructs deserialized representation from the value containing this union field.
    fn deserialize(owner:ParentType) -> Result<Self,DeserializationError>;
}

impl<'a> DeserializableUnionField<'a, OutboundMessage<'a>> for FromServerPayload<'a> {
    fn deserialize(message:OutboundMessage<'a>) -> Result<Self, DeserializationError> {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                Ok(FromServerPayload::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default(),
                })
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                Ok(FromServerPayload::FileContentsReply {
                    contents: payload.contents().unwrap_or_default()
                })
            }
            OutboundPayload::SUCCESS => Ok(FromServerPayload::Success {}),
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                Ok(FromServerPayload::VisualizationUpdate {
                    data: payload.data(),
                    context: message::VisualisationContext::deserialize(context)?,
                })
            }
            OutboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}

impl<'a> DeserializableUnionField<'a, InboundMessage<'a>> for ToServerPayloadOwned {
    fn deserialize(message:InboundMessage<'a>) -> Result<Self, DeserializationError> {
        match message.payload_type() {
            InboundPayload::INIT_SESSION_CMD => {
                let payload = message.payload_as_init_session_cmd().unwrap();
                Ok(ToServerPayloadOwned::InitSession {
                    client_id: payload.identifier().into()
                })
            }
            InboundPayload::WRITE_FILE_CMD => {
                let payload = message.payload_as_write_file_cmd().unwrap();

                Ok(ToServerPayloadOwned::WriteFile {
                    path: LSPath::deserialize_required_opt(payload.path())?,
                    contents: Vec::from(payload.contents().unwrap_or_default())
                })
            }
            InboundPayload::READ_FILE_CMD => {
                let payload = message.payload_as_read_file_cmd().unwrap();
                Ok(ToServerPayloadOwned::ReadFile {
                    path: LSPath::deserialize_required_opt(payload.path())?,
                })
            }
            InboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}

impl<'a> DeserializableUnionField<'a, OutboundMessage<'a>> for FromServerPayloadOwned {
    fn deserialize(message: OutboundMessage<'a>) -> Result<Self, DeserializationError> {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                Ok(FromServerPayloadOwned::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default().to_string(),
                })
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                Ok(FromServerPayloadOwned::FileContentsReply {
                    contents: Vec::from(payload.contents().unwrap_or_default())
                })
            }
            OutboundPayload::SUCCESS => Ok(FromServerPayloadOwned::Success {}),
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                Ok(FromServerPayloadOwned::VisualizationUpdate {
                    data: Vec::from(payload.data()),
                    context: message::VisualisationContext::deserialize(context)?,
                })
            }
            OutboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}



// ========================
// === SerializableRoot ===
// ========================

/// Representation of the value that can be written to FlatBuffer-serialized binary blob.
pub trait SerializableRoot {
    /// Stores the entity into the builder and calls `finish` on it.
    fn write(&self, builder:&mut FlatBufferBuilder);

    /// Returns `finish`ed builder with the serialized entity.
    fn serialize(&self) -> FlatBufferBuilder {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(INITIAL_BUFFER_SIZE);
        self.write(&mut builder);
        builder
    }

    /// Calls the given function with the binary blob with the serialized entity.
    fn with_serialized<R>(&self, f:impl FnOnce(&[u8]) -> R) -> R {
        let buffer = self.serialize();
        f(buffer.finished_data())
    }
}

impl<T> SerializableRoot for MessageToServer<T>
where T:SerializableUnion<EnumType=InboundPayload> {
    fn write(&self, builder:&mut FlatBufferBuilder) {
        let correlation_id = self.correlation_id.map(EnsoUUID::from);
        let message_id     = self.message_id.into();
        let payload_type   = self.payload.active_variant();
        let payload        = Some(self.payload.serialize(builder));
        let message        = InboundMessage::create(builder, &InboundMessageArgs {
            correlationId : correlation_id.as_ref(),
            messageId     : Some(&message_id),
            payload_type,
            payload,
        });
        builder.finish(message,None);
    }
}

impl<T> SerializableRoot for MessageFromServer<T>
where T : SerializableUnion<EnumType=OutboundPayload> {
    fn write(&self, builder:&mut FlatBufferBuilder) {
        let correlation_id = self.correlation_id.map(EnsoUUID::from);
        let message_id     = self.message_id.into();
        let payload_type   = self.payload.active_variant();
        let payload        = Some(self.payload.serialize(builder));
        let message        = OutboundMessage::create(builder, &OutboundMessageArgs {
            correlationId : correlation_id.as_ref(),
            messageId     : Some(&message_id),
            payload_type,
            payload,
        });
        builder.finish(message,None);
    }
}



// ==========================
// === DeserializableRoot ===
// ==========================

/// Representation of the value that can be read from FlatBuffer-serialized binary blob.
pub trait DeserializableRoot<'a> : Sized {
    /// Construct representation of the value from a binary blob in FlatBuffer format.
    fn deserialize(data:&'a [u8]) -> Result<Self,DeserializationError>;
}

impl<'a,T> DeserializableRoot<'a> for MessageToServer<T>
where T: DeserializableUnionField<'a,InboundMessage<'a>> {
    fn deserialize(data: &'a [u8]) -> Result<Self, DeserializationError> {
        let message = flatbuffers::get_root::<InboundMessage>(data);
        let payload = T::deserialize(message)?;
        Ok(MessageToServer(Message {
            message_id     : message.messageId().into(),
            correlation_id : message.correlationId().map(|id| id.into()),
            payload
        }))
    }
}

impl<'a,T> DeserializableRoot<'a> for MessageFromServer<T>
where T: DeserializableUnionField<'a,OutboundMessage<'a>> {
    fn deserialize(data: &'a [u8]) -> Result<Self, DeserializationError> {
        let message = flatbuffers::get_root::<OutboundMessage>(data);
        let payload = T::deserialize(message)?;
        Ok(MessageFromServer(Message {
            message_id     : message.messageId().into(),
            correlation_id : message.correlationId().map(|id| id.into()),
            payload
        }))
    }
}
