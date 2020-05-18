//! Crate containing the Engine Services binary protocol interface.

pub mod client;
pub mod connection;
pub mod message;
pub mod payload;
pub mod uuid;

// #[allow(dead_code, unused_imports)]
// use flatbuffers;

//use crate::prelude::*;

pub use client::API;
pub use client::Client;
pub use client::MockClient;
pub use connection::Connection;
//
// use futures::future::LocalBoxFuture;
// use logger::*;
//
// pub use crate::generated::binary_protocol_generated as binary_protocol;
// use json_rpc::{Transport, TransportEvent};
// use crate::language_server::Path as LSPath;
// use json_rpc::error::RpcError;
// use crate::new_handler::{HandlerHandle, Disposition};
// use crate::binary::message::{Message, MessageFromServerOwned};
// use crate::binary::payload::{FromServerOwned, ToServerPayload};
//
// use payload::VisualisationContext;
// use crate::common::error::{UnexpectedTextMessage, UnexpectedMessage};
//
//
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     use wasm_bindgen_test::wasm_bindgen_test_configure;
//     wasm_bindgen_test_configure!(run_in_browser);
//
//
//     #[wasm_bindgen_test::wasm_bindgen_test(async)]
//     #[allow(dead_code)]
//     async fn first_real_test() {
//         ensogl_system_web::set_stdout();
//
//
//
//         assert!(false);
//     }
// }