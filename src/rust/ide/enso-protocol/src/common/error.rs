use crate::prelude::*;

/// When trying to parse a line, not a single line was produced.
#[derive(Debug,Fail,Clone,Copy)]
#[fail(display = "No active request by id {}", _0)]
pub struct NoSuchRequest<Id:Sync + Send + Debug + Display + 'static>(pub Id);

#[derive(Debug,Fail,Clone,Copy)]
#[fail(display = "Received text message when expecting only binary ones.")]
pub struct UnexpectedTextMessage;

#[derive(Debug,Fail,Clone)]
#[fail(display = "Failed to deserialize the received message. {}", _0)]
pub struct DeserializationError(pub String);

#[derive(Debug,Fail,Clone)]
#[fail(display = "Received a message that is neither a response nor a notification")]
pub struct UnexpectedMessage;


