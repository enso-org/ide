//! Here are the common structures used by Controllers notifications (sent between controllers and
//! from controller to view).

use crate::prelude::*;



// =================
// === Publisher ===
// =================

/// A buffer size for notification publisher.
///
/// If Publisher buffer will be full, the thread sending next notification will be blocked until
/// all subscribers read message from buffer. We don't expect much traffic on file notifications,
/// therefore there is no need for setting big buffers.
const NOTIFICATION_BUFFER_SIZE : usize = 36;

/// A notification publisher which implements Debug, Default and CloneRef (which is same as
/// republishing for the same stream).
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Publisher<Message>(pub flo_stream::Publisher<Message>);

impl<Message:Clone> Default for Publisher<Message> {
    fn default() -> Self {
        Self(flo_stream::Publisher::new(NOTIFICATION_BUFFER_SIZE))
    }
}

impl<Message:'static> Debug for Publisher<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "notification::Publisher<{:?}>", std::any::TypeId::of::<Message>())
    }
}

impl<Message:Clone> CloneRef for Publisher<Message> {
    fn clone_ref(&self) -> Self {
        self.clone()
    }
}

impl<Message:Clone> Clone for Publisher<Message> {
    fn clone(&self) -> Self {
        let Self(inner) = self;
        Self(inner.republish())
    }
}
