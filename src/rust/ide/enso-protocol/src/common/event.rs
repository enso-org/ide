
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
