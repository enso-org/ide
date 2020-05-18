
pub mod error;
pub mod event;
pub mod ongoing_calls;

pub trait IsConnection {
    type Client;
}
