//! Crate containing the Engine Services binary protocol interface.

pub mod connection;
pub mod message;
pub mod payload;
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
use futures::future::LocalBoxFuture;
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
use crate::binary::message::{Message, MessageFromServerOwned};
use crate::binary::payload::{FromServerOwned, ToServerPayload};

use payload::VisualisationContext;
use crate::common::error::{UnexpectedTextMessage, UnexpectedMessage};


#[derive(Clone,Debug)]
pub enum Notification {
    VisualizationUpdate {context:VisualisationContext, data:Vec<u8>},
}

pub trait API {
    fn init(&self, client_id:Uuid) -> LocalBoxFuture<FallibleResult<()>>;
    fn write_file(&self, path:&LSPath, contents:&[u8]) -> LocalBoxFuture<FallibleResult<()>>;
    fn read_file(&self, path:&LSPath) -> LocalBoxFuture<FallibleResult<Vec<u8>>>;
}

use mockall::mock;

mock!{
    Client {
        fn init_ready(&self, client_id:Uuid) -> FallibleResult<()>;
        fn write_file_ready(&self, path:&LSPath, contents:&[u8]) -> FallibleResult<()>;
        fn read_file_ready(&self, path:&LSPath) -> FallibleResult<Vec<u8>>;
    }
}

impl API for MockClient {
    fn init(&self, client_id:Uuid) -> LocalBoxFuture<FallibleResult<()>> {
        futures::future::ready(self.init_ready(client_id)).boxed_local()
    }
    fn write_file(&self, path:&LSPath, contents:&[u8]) -> LocalBoxFuture<FallibleResult<()>> {
        futures::future::ready(self.write_file_ready(path,contents)).boxed_local()
    }
    fn read_file(&self, path:&LSPath) -> LocalBoxFuture<FallibleResult<Vec<u8>>>{
        futures::future::ready(self.read_file_ready(path)).boxed_local()
    }
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
            let binary_data = match event {
                TransportEvent::BinaryMessage(data) => data,
                _ =>
                    return Disposition::error(UnexpectedTextMessage),
            };
            let message = match MessageFromServerOwned::deserialize_owned(&binary_data) {
                Ok(message) => message,
                Err(e)      => return Disposition::error(e),
            };
            info!(logger, "Received binary message {message:?}");
            match message.payload {
                FromServerOwned::VisualizationUpdate {context,data} =>
                    Disposition::notify(Notification::VisualizationUpdate {data,context}),
                _ => {
                    if let Some(id) = message.correlation_id {
                        let reply = message.payload;
                        Disposition::HandleReply {id,reply}
                    } else {
                        // Not a known notification and yet not a response to our request.
                        Disposition::error(UnexpectedMessage)
                    }
                }
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


    pub fn open<F,R>(&self, payload:ToServerPayload, f:F) -> LocalBoxFuture<FallibleResult<R>>
    where F : FnOnce(FromServerOwned) -> FallibleResult<R>,
          R : 'static,
          F : 'static, {
        let message = Message::new_to_server(payload);
        let id = message.message_id;

        let logger = self.logger.clone_ref();
        let completer = move |reply| {
            info!(logger,"Completing request {id} with a reply: {reply:?}");
            if let FromServerOwned::Error {code,message} = reply {
                let error = RpcError::new_remote_error(code.into(), message);
                Err(error.into())
            } else {
                f(reply)
            }
        };

        let fut = self.handler.make_request(&message, completer);
        Box::pin(fut)
    }

    pub fn runner(&self) -> impl Future<Output = ()> {
        self.handler.runner()
    }
}

impl API for Client {
    fn init(&self, client_id:Uuid) -> LocalBoxFuture<FallibleResult<()>> {
        info!(self.logger,"Initializing binary connection as {client_id}");
        let payload = ToServerPayload::InitSession {client_id};
        self.open(payload,expect_success)
    }

    fn write_file(&self, path:&LSPath, contents:&[u8]) -> LocalBoxFuture<FallibleResult<()>> {
        info!(self.logger,"Writing file {path} with {contents:?}");
        let payload = ToServerPayload::WriteFile {path,contents};
        self.open(payload,expect_success)
    }

    fn read_file(&self, path:&LSPath) -> LocalBoxFuture<FallibleResult<Vec<u8>>> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test_configure;
    wasm_bindgen_test_configure!(run_in_browser);


    #[wasm_bindgen_test::wasm_bindgen_test(async)]
    #[allow(dead_code)]
    async fn first_real_test() {
        ensogl_system_web::set_stdout();



        assert!(false);
    }
}