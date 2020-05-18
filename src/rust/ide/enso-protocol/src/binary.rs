//! Crate containing the Engine Services binary protocol interface.

pub mod connection;
pub mod message;
pub mod uuid;

#[allow(dead_code, unused_imports)]
use flatbuffers;

use crate::prelude::*;

use futures::{FutureExt, SinkExt};
use futures::StreamExt;
use futures::Stream;
use futures::channel::mpsc::unbounded;
use futures::channel::mpsc::UnboundedSender;
use futures::channel::oneshot;
use futures::future::BoxFuture;
use logger::*;

pub use crate::generated::binary_protocol_generated as binary_protocol;
use json_rpc::{Transport, TransportEvent};
use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::EnsoUUID;
use crate::language_server::Path as LSPath;
use binary_protocol::org::enso::languageserver::protocol::binary::*;
use flatbuffers::{FlatBufferBuilder, WIPOffset, UnionWIPOffset};
use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::InboundPayload::INIT_SESSION_CMD;
use std::future::Future;
use utils::fail::FallibleResult;
use json_rpc::error::{RpcError, HandlingError};
use futures::channel::oneshot::Canceled;
use std::pin::Pin;
use crate::new_handler::{HandlerHandle, Disposition};
use crate::binary::message::Message;

#[derive(Clone,Debug)]
pub struct VisualisationContext {
    pub visualization_id : Uuid,
    pub context_id       : Uuid,
    pub expression_id    : Uuid,
}

pub fn serialize_path<'a>(path:&LSPath, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Path<'a>> {
    let root_id      = path.root_id.into();
    let segment_refs = path.segments.iter().map(|s| s.as_str()).collect_vec();
    let segments     = builder.create_vector_of_strings(&segment_refs);
    Path::create(builder, &PathArgs {
        rootId   : Some(&root_id),
        segments : Some(segments),
    })
}

#[derive(Clone,Debug)]
pub enum FromServerOwned {
    Error {code:i32, message:String},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:Vec<u8>},
    FileContentsReply   {contents:Vec<u8>},
}

impl FromServerOwned {
    pub fn deserialize<'a>(message:&OutboundMessage<'a>) -> Self {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                FromServerOwned::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default().to_string(),
                }
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                FromServerOwned::FileContentsReply {
                    contents: Vec::from(payload.contents().unwrap_or_default())
                }
            }
            OutboundPayload::SUCCESS => FromServerOwned::Success {},
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                FromServerOwned::VisualizationUpdate {
                    data: Vec::from(payload.data()),
                    context: VisualisationContext {
                        context_id: context.contextId().into(),
                        expression_id: context.expressionId().into(),
                        visualization_id: context.visualisationId().into(),
                    }
                }
            }
            _ => todo!()
        }
    }
}

#[derive(Clone,Debug)]
pub enum FromServer<'a> {
    Error {code:i32, message:&'a str},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:&'a [u8]},
    FileContentsReply {contents:&'a [u8]},
}

#[derive(Clone,Debug)]
pub enum ToServerPayload<'a> {
    InitSession {client_id:Uuid},
    WriteFile   {path:&'a LSPath, contents:&'a[u8]},
    ReadFile    {path:&'a LSPath}
}

type MessageFromServerOwned = Message<FromServerOwned>;

type MessageToServer<'a> = Message<ToServerPayload<'a>>;

impl<'a> super::new_handler::MessageToServer for MessageToServer<'a> {
    type Id = Uuid;

    fn send(&self, transport:&mut dyn Transport) -> FallibleResult<()> {
        self.with_serialized(|data| transport.send_binary(data))
    }

    fn id(&self) -> Self::Id {
        self.message_id
    }
}

pub trait IsPayloadToServer {
    type PayloadType;

    fn write_message(&self, builder:&mut FlatBufferBuilder, message_id:Uuid, correlation_id:Option<Uuid>);
    fn write_payload(&self, builder:&mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset>;
    fn payload_type(&self) -> Self::PayloadType;
}

impl<'a> IsPayloadToServer for ToServerPayload<'a> {
    type PayloadType = InboundPayload;

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

/// When trying to parse a line, not a single line was produced.
#[derive(Debug,Fail,Clone,Copy)]
#[fail(display = "No active request by id {}", _0)]
pub struct NoSuchRequest<Id : Sync + Send + Debug + Display + 'static>(pub Id);


/// Event emitted by the `Handler<N>`.
#[derive(Debug)]
pub enum Event<N> {
    /// Transport has been closed.
    Closed,
    /// Error occurred.
    Error(failure::Error),
    /// Notification received.
    Notification(N),
}

#[derive(Clone,Debug)]
pub enum Notification {
    VisualizationUpdate {context:VisualisationContext, data:Vec<u8>},
}

#[derive(Clone,Derivative)]
#[derivative(Debug)]
pub struct Client {
    handler : super::new_handler::HandlerHandle<Uuid,FromServerOwned,Notification>,
    logger  : Logger,
}

pub fn expect_success(result:FromServerOwned) -> FallibleResult<()> {
    match result {
        FromServerOwned::Success {} => Ok(()),
        _ => Err(RpcError::MismatchedResponseType.into()),
    }
}

impl Client {
    pub fn processor(logger:Logger) -> impl FnMut(TransportEvent) -> Disposition<Uuid,FromServerOwned,Notification> + 'static {
        move |event:TransportEvent| {
            if let TransportEvent::BinaryMessage(data) = event {
                let message = MessageFromServerOwned::deserialize_owned(&data);
                info!(logger, "Received binary message {message:?}");
                match message.payload {
                    FromServerOwned::VisualizationUpdate { context, data } =>
                        Disposition::notify(Notification::VisualizationUpdate { data, context }),
                    _ => {
                        if let Some(id) = message.correlation_id {
                            let reply = message.payload;
                            Disposition::HandleReply {id,reply}
                        } else {
                            // Not a known notification and yet not a response to our request.
                            Disposition::Ignore
                        }
                    }
                }
            } else {
                // Not the kind of events we are interested in.
                Disposition::Ignore
            }
        }
    }

    pub fn new(mut transport:impl Transport + 'static) -> Client {
        let logger = Logger::new("binary-protocol");
        let processor = Self::processor(logger.clone_ref());
        Client {
            logger          : logger.clone_ref(),
            handler         : HandlerHandle::new(transport,logger,processor),
        }
    }


    pub fn open<'a,F,R>(&self, payload:ToServerPayload<'a>, f:F) -> Pin<Box<dyn Future<Output=FallibleResult<R>> + 'static>>
    where F : FnOnce(FromServerOwned) -> FallibleResult<R>,
          R : 'static,
          F : 'static, {
        let message = Message::new(payload);
        let id = message.message_id;

        let logger = self.logger.clone_ref();
        let completer = move |reply| {
            info!(logger,"Processing reply to request {id}: {reply:?}");
            if let FromServerOwned::Error {code,message} = reply {
                let error = RpcError::new_remote_error(code.into(), message);
                Err(error.into())
            } else {
                f(reply)
            }
        };

        let fut = self.handler.open(&message,completer);
        Box::pin(fut)
    }

    pub fn init(&self, client_id:Uuid) -> impl Future<Output = FallibleResult<()>> {
        info!(self.logger,"Initializing binary connection as {client_id}");
        let payload = ToServerPayload::InitSession {client_id};
        self.open(payload,expect_success)
    }

    pub fn write_file(&self, path:&LSPath, contents:&[u8]) -> impl Future<Output = FallibleResult<()>> {
        info!(self.logger,"Writing file {path} with {contents:?}");
        let payload = ToServerPayload::WriteFile {path,contents};
        self.open(payload,expect_success)
    }

    pub fn read_file(&self, path:&LSPath) -> impl Future<Output = FallibleResult<Vec<u8>>> {
        info!(self.logger,"Reading file {path}");
        let payload = ToServerPayload::ReadFile {path};
        self.open(payload, move |result| {
            match result {
                FromServerOwned::FileContentsReply {contents} =>
                    Ok(contents),
                _ =>
                    Err(RpcError::MismatchedResponseType.into()),
            }
        })
    }

    pub fn runner(&self) -> impl Future<Output = ()> {
        self.handler.runner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test_configure;
    wasm_bindgen_test_configure!(run_in_browser);


    #[test]
    fn uuid_round_trips() {
        //let uuid = Uuid::new_v4();
        let uuid = Uuid::parse_str("6de39f7b-df3a-4a3c-84eb-5eaf96ddbac2").unwrap();
        println!("uuid bytes: {:?}", uuid.as_bytes());
        println!("initial uuid: {:?}", uuid);
        let enso = EnsoUUID::from(uuid);
        println!("enso-uuid: {:?}", enso);

        let uuid2 = Uuid::from(enso);
        println!("restored uuid: {:?}", uuid2);

        let enso_uuid = EnsoUUID::from(uuid);
        println!("uuid bytes: {:?}", enso_uuid.leastSigBits().to_le_bytes());

        assert_eq!(uuid,Uuid::from(EnsoUUID::from(uuid)));
    }


    #[wasm_bindgen_test::wasm_bindgen_test(async)]
    #[allow(dead_code)]
    async fn first_real_test() {
        ensogl_system_web::set_stdout();



        assert!(false);
    }
}