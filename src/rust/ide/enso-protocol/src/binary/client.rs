//! Module defines LS binary protocol client `API` and its two implementation: `Client` and
//! `MockClient`.

use crate::prelude::*;
use crate::new_handler::{HandlerHandle, Disposition};
use crate::binary::payload::{FromServerOwned, ToServerPayload};
use json_rpc::error::RpcError;
use json_rpc::{TransportEvent, Transport};
use crate::common::error::{UnexpectedTextMessage, UnexpectedMessage};
use crate::binary::message::{MessageFromServerOwned, Message};

use crate::language_server::types as ls;

use mockall::mock;

/// Identifies the visualization.
#[allow(missing_docs)]
#[derive(Clone,Debug,Copy)]
pub struct VisualisationContext {
    pub visualization_id : Uuid,
    pub context_id       : Uuid,
    pub expression_id    : Uuid,
}

/// The notifications that binary protocol client may receive.
#[derive(Clone,Debug)]
pub enum Notification {
    /// A new data has been sent for a visualization.
    VisualizationUpdate {
        /// Identifies the specific visualization.
        context:VisualisationContext,
        /// Data to be passed to the visualization.
        data:Vec<u8>
    },
}

/// The Engine Services Language Server Binary Protocol Client API.
pub trait API {
    /// Initializes the protocol. Must be called exactly once before making any other calls.
    fn init(&self, client_id:Uuid) -> LocalBoxFuture<FallibleResult<()>>;
    /// Writes binary data to the file.
    fn write_file(&self, path:&ls::Path, contents:&[u8]) -> LocalBoxFuture<FallibleResult<()>>;
    /// Retrieves the file contents as a binary data.
    fn read_file(&self, path:&ls::Path) -> LocalBoxFuture<FallibleResult<Vec<u8>>>;
}



// ==============
// === Client ===
// ==============

/// The client for Engine Services Language Server Binary Protocol.
#[derive(Clone,Derivative)]
#[derivative(Debug)]
pub struct Client {
    handler : HandlerHandle<Uuid,FromServerOwned,Notification>,
    logger  : Logger,
}

impl Client {
    /// Helper function that fails if the received message represents a remote error.
    fn expect_success(result:FromServerOwned) -> FallibleResult<()> {
        if let FromServerOwned::Success {} = result {
            Ok(())
        } else {
            Err(RpcError::MismatchedResponseType.into())
        }
    }

    /// Helper function that does early processing of the peer's message and decides how it shall
    /// be handled.
    fn processor(logger:Logger) -> impl FnMut(TransportEvent) -> Disposition<Uuid,FromServerOwned,Notification> + 'static {
        move |event:TransportEvent| {
            let binary_data = match event {
                TransportEvent::BinaryMessage(data) =>
                    data,
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

    /// Creates a new client from the given transport to the Language Server Data Endpoint.
    ///
    /// Before client is functional:
    /// * `runner` must be scheduled for execution;
    /// * `init` must be called or it needs to be wrapped into `Connection`.
    pub fn new(transport:impl Transport + 'static) -> Client {
        let logger = Logger::new("binary-protocol");
        let processor = Self::processor(logger.clone_ref());
        Client {
            logger  : logger.clone_ref(),
            handler : HandlerHandle::new(transport,logger,processor),
        }
    }

    /// Starts a new request, described by the given payload.
    /// Function `f`
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

    /// A `runner`. Its execution must be scheduled for `Client` to be able to complete requests and
    /// emit events.
    pub fn runner(&self) -> impl Future<Output = ()> {
        self.handler.runner()
    }
}

impl API for Client {
    fn init(&self, client_id:Uuid) -> LocalBoxFuture<FallibleResult<()>> {
        info!(self.logger,"Initializing binary connection as {client_id}");
        let payload = ToServerPayload::InitSession {client_id};
        self.open(payload,Self::expect_success)
    }

    fn write_file(&self, path:&ls::Path, contents:&[u8]) -> LocalBoxFuture<FallibleResult<()>> {
        info!(self.logger,"Writing file {path} with {contents:?}");
        let payload = ToServerPayload::WriteFile {path,contents};
        self.open(payload,Self::expect_success)
    }

    fn read_file(&self, path:&ls::Path) -> LocalBoxFuture<FallibleResult<Vec<u8>>> {
        info!(self.logger,"Reading file {path}");
        let payload = ToServerPayload::ReadFile {path};
        self.open(payload, move |result| {
            if let FromServerOwned::FileContentsReply {contents} = result {
                Ok(contents)
            } else {
                Err(RpcError::MismatchedResponseType.into())
            }
        })
    }
}



// ==================
// === MockClient ===
// ==================

mock!{
    pub Client {
        fn init_ready(&self, client_id:Uuid) -> FallibleResult<()>;
        fn write_file_ready(&self, path:&ls::Path, contents:&[u8]) -> FallibleResult<()>;
        fn read_file_ready(&self, path:&ls::Path) -> FallibleResult<Vec<u8>>;
    }
}

impl API for MockClient {
    fn init(&self, client_id:Uuid) -> LocalBoxFuture<FallibleResult<()>> {
        futures::future::ready(self.init_ready(client_id)).boxed_local()
    }
    fn write_file(&self, path:&ls::Path, contents:&[u8]) -> LocalBoxFuture<FallibleResult<()>> {
        futures::future::ready(self.write_file_ready(path,contents)).boxed_local()
    }
    fn read_file(&self, path:&ls::Path) -> LocalBoxFuture<FallibleResult<Vec<u8>>>{
        futures::future::ready(self.read_file_ready(path)).boxed_local()
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use json_rpc::test_util::transport::mock::MockTransport;
    use futures::task::LocalSpawnExt;
    use chrono::format::Item::Fixed;



    // ===============
    // === Fixture ===
    // ===============

    struct ClientFixture {
        transport : MockTransport,
        client    : Client,
        executor  : futures::executor::LocalPool,
    }

    impl ClientFixture {
        fn new() -> ClientFixture {
            let transport = MockTransport::new();
            let client    = Client::new(transport.clone());
            let executor  = futures::executor::LocalPool::new();
            executor.spawner().spawn_local(client.runner()).unwrap();
            ClientFixture {transport,client,executor}
        }
    }

    #[test]
    fn test_init() {
        let mut fixture = ClientFixture::new();

        let client_id = Uuid::new_v4();

        //let init_fut = fixture.client.init(client_id);
        // fixture.executor.spawner().spawn_local(async move {
        //     assert!(init_fut.await.is_ok());
        // });

        println!("got msg {:?}", fixture.transport.expect_binary_message());


    }
}


