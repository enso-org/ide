use crate::prelude::*;

use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary as generated;
use generated::Path;
use generated::PathArgs;
use generated::OutboundMessage;
//use generated::OutboundMessageArgs;
use generated::InboundPayload;
use generated::OutboundPayload;
use generated::InboundMessage;
use generated::InboundMessageArgs;
use generated::ReadFileCommand;
use generated::ReadFileCommandArgs;
use generated::InitSessionCommand;
use generated::InitSessionCommandArgs;
use generated::WriteFileCommand;
use generated::WriteFileCommandArgs;
use generated::EnsoUUID;

use crate::language_server::types::Path as LSPath;

use flatbuffers::FlatBufferBuilder;
use flatbuffers::UnionWIPOffset;
use flatbuffers::WIPOffset;
use crate::common::error::DeserializationError;
use crate::binary::client::VisualisationContext;
use crate::binary::message::Message;

pub fn serialize_path<'a>(path:&LSPath, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Path<'a>> {
    let root_id      = path.root_id.into();
    let segment_refs = path.segments.iter().map(|s| s.as_str()).collect_vec();
    let segments     = builder.create_vector_of_strings(&segment_refs);
    Path::create(builder, &PathArgs {
        rootId   : Some(&root_id),
        segments : Some(segments),
    })
}

pub fn deserialize_path<'a>(path:&Path<'a>) -> Result<LSPath,DeserializationError> {
    let missing_root_id = || DeserializationError("Missing root ID".to_string());
    let root_id         = Uuid::from(path.rootId().ok_or_else(missing_root_id)?);
    let segments        = match path.segments() {
        Some(segments) => {
            (0..segments.len()).map(|ix| segments.get(ix).to_string()).collect_vec()
        }
        None => Vec::new(),
    };
    Ok(LSPath {root_id,segments})
}

pub fn deserialize_required_path<'a>(path:&Option<Path<'a>>) -> Result<LSPath,DeserializationError> {
    let missing_required_field = || DeserializationError("Missing a required field".to_string());
    deserialize_path(&path.ok_or_else(missing_required_field)?)
}

/// Payloads that can be serialized and sent as a message to server.
pub trait SerializableToServer {
    /// Writes the message into a buffer and finishes it.
    fn write_message(&self, builder:&mut FlatBufferBuilder, message_id:Uuid, correlation_id:Option<Uuid>) {
        let payload_type   = self.payload_type();
        let payload        = Some(self.write_payload(builder));
        let correlation_id2 = correlation_id.map(EnsoUUID::from);
        println!("Sending message id: {:?}, generated from {}", EnsoUUID::from(message_id), message_id);
        let message        = InboundMessage::create(builder, &InboundMessageArgs {
            correlationId : correlation_id2.as_ref(),
            messageId     : Some(&message_id.into()),
            payload_type,
            payload,
        });
        builder.finish(message,None);
    }

    /// Writes just the payload into the buffer.
    fn write_payload(&self, builder:&mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset>;

    /// Returns enumeration describing variant of this payload.
    fn payload_type(&self) -> InboundPayload;
}

#[derive(Clone,Debug)]
pub enum FromServerOwned {
    Error {code:i32, message:String},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:Vec<u8>},
    FileContentsReply   {contents:Vec<u8>},
}

impl FromServerOwned {
    pub fn deserialize_owned(message:&OutboundMessage) -> Result<Self,DeserializationError> {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                Ok(FromServerOwned::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default().to_string(),
                })
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                Ok(FromServerOwned::FileContentsReply {
                    contents: Vec::from(payload.contents().unwrap_or_default())
                })
            }
            OutboundPayload::SUCCESS => Ok(FromServerOwned::Success {}),
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                Ok(FromServerOwned::VisualizationUpdate {
                    data: Vec::from(payload.data()),
                    context: VisualisationContext {
                        context_id: context.contextId().into(),
                        expression_id: context.expressionId().into(),
                        visualization_id: context.visualisationId().into(),
                    }
                })
            }
            OutboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}

impl<'a> SerializableToServer for ToServerPayload<'a> {
    fn write_payload(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            ToServerPayload::InitSession {client_id} => {
                InitSessionCommand::create(builder, &InitSessionCommandArgs {
                    identifier : Some(&client_id.into())
                }).as_union_value()
            }
            ToServerPayload::WriteFile {path,contents} => {
                let path     = serialize_path(path,builder);
                let contents = builder.create_vector(contents);
                WriteFileCommand::create(builder, &WriteFileCommandArgs {
                    path : Some(path),
                    contents : Some(contents),
                }).as_union_value()
            }
            ToServerPayload::ReadFile {path} => {
                let path = serialize_path(path,builder);
                ReadFileCommand::create(builder, &ReadFileCommandArgs {
                    path : Some(path)
                }).as_union_value()
            }
        }
    }

    fn payload_type(&self) -> InboundPayload {
        match self {
            ToServerPayload::InitSession {..} => InboundPayload::INIT_SESSION_CMD,
            ToServerPayload::WriteFile   {..} => InboundPayload::WRITE_FILE_CMD,
            ToServerPayload::ReadFile    {..} => InboundPayload::READ_FILE_CMD,
        }
    }
}

#[derive(Clone,Debug)]
pub enum FromServerRef<'a> {
    Error {code:i32, message:&'a str},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:&'a [u8]},
    FileContentsReply {contents:&'a [u8]},
}


impl<'a> FromServerRef<'a> {
    pub fn deserialize(message:&OutboundMessage<'a>) -> Result<Self,DeserializationError> {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                Ok(FromServerRef::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default(),
                })
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                Ok(FromServerRef::FileContentsReply {
                    contents: payload.contents().unwrap_or_default()
                })
            }
            OutboundPayload::SUCCESS => Ok(FromServerRef::Success {}),
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                Ok(FromServerRef::VisualizationUpdate {
                    data: payload.data(),
                    context: VisualisationContext {
                        context_id: context.contextId().into(),
                        expression_id: context.expressionId().into(),
                        visualization_id: context.visualisationId().into(),
                    }
                })
            }
            OutboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}

/// Payloads that can be serialized and sent as a message to server.
pub trait DeserializableToServer : Sized {
    /// Writes the message into a buffer and finishes it.
    fn read_message(data:&[u8]) -> Result<Message<Self>,DeserializationError> {
        let message = flatbuffers::get_root::<InboundMessage>(data);
        let payload = Self::from_message(&message)?;
        Ok(Message {
            message_id : message.messageId().into(),
            correlation_id : message.correlationId().map(|id| id.into()),
            payload
        })
    }

    fn from_message(message:&InboundMessage) -> Result<Self,DeserializationError>;
}



#[derive(Clone,Debug)]
pub enum ToServerPayload<'a> {
    InitSession {client_id:Uuid},
    WriteFile   {path:&'a LSPath, contents:&'a[u8]},
    ReadFile    {path:&'a LSPath}
}

impl<'a> SerializableToServer for ToServerPayload<'a> {
    fn write_payload(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            ToServerPayload::InitSession {client_id} => {
                InitSessionCommand::create(builder, &InitSessionCommandArgs {
                    identifier : Some(&client_id.into())
                }).as_union_value()
            }
            ToServerPayload::WriteFile {path,contents} => {
                let path     = serialize_path(path,builder);
                let contents = builder.create_vector(contents);
                WriteFileCommand::create(builder, &WriteFileCommandArgs {
                    path : Some(path),
                    contents : Some(contents),
                }).as_union_value()
            }
            ToServerPayload::ReadFile {path} => {
                let path = serialize_path(path,builder);
                ReadFileCommand::create(builder, &ReadFileCommandArgs {
                    path : Some(path)
                }).as_union_value()
            }
        }
    }

    fn payload_type(&self) -> InboundPayload {
        match self {
            ToServerPayload::InitSession {..} => InboundPayload::INIT_SESSION_CMD,
            ToServerPayload::WriteFile   {..} => InboundPayload::WRITE_FILE_CMD,
            ToServerPayload::ReadFile    {..} => InboundPayload::READ_FILE_CMD,
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub enum ToServerPayloadOwned {
    InitSession {client_id:Uuid},
    WriteFile   {path:LSPath, contents:Vec<u8>},
    ReadFile    {path:LSPath}
}

impl DeserializableToServer for ToServerPayloadOwned {
    fn from_message(message: &InboundMessage) -> Result<Self,DeserializationError> {
        let missing_required_field = DeserializationError("missing a required field".into());
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
                    path: deserialize_path(&payload.path().ok_or(missing_required_field)?)?,
                    contents: Vec::from(payload.contents().unwrap_or_default())
                })
            }
            InboundPayload::READ_FILE_CMD => {
                let payload = message.payload_as_write_file_cmd().unwrap();
                Ok(ToServerPayloadOwned::ReadFile {
                    path: deserialize_path(&payload.path().ok_or(missing_required_field)?)?,
                })
            }
            InboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}


