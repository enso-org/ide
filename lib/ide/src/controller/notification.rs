//! Here are the common structures used by Controllers notifications (sent between controllers and
//! from controller to view).

use crate::prelude::*;



// ===============
// === Commons ===
// ===============

/// A buffer size for notification publisher.
///
/// If Publisher buffer will be full, the thread sending next notification will be blocked until
/// all subscribers read message from buffer. We don't expect much traffic on file notifications,
/// therefore there is no need for setting big buffers.
const NOTIFICATION_BUFFER_SIZE : usize = 36;

/// A macro generating newtype for notification publisher which implements Debug and Default.
///
/// For message Msg you can write
/// ```rust
/// publisher_newtype(MsgPub,Msg);
/// ```
/// which generate
/// ```rust
/// /// A publisher newtype which implements Debug and Default.
///  #[derive(Shrinkwrap)]
///  #[shrinkwrap(mutable)]
///  pub struct MsgPub(pub flo_stream::Publisher<Msg>);
/// ```
/// along with some basic implementation of Default and Debug.
macro_rules! publisher_newtype {
    ($name:ident, $message:ty) => {
        /// A publisher newtype which implements Debug and Default.
        #[derive(Shrinkwrap)]
        #[shrinkwrap(mutable)]
        pub struct $name(pub flo_stream::Publisher<$message>);

        impl Default for $name {
            fn default() -> Self {
                Self(flo_stream::Publisher::new(NOTIFICATION_BUFFER_SIZE))
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "{:?}", std::any::TypeId::of::<Self>())
            }
        }
    }
}


// =====================================
// === Double Representation Changes ===
// =====================================

// === Text ===

/// A notification about changes of text representation or plain text file content.
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum TextChanged {
    /// A notification indicating that the whole content was changed and should be fully reloaded.
    Entirely,
}

publisher_newtype!(TextChangedPublisher,TextChanged);

// === Graph ===

/// A notification about changes of graph representation of a module.
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum GraphChanged {
    /// A notification indicating that the whole graph was changed and should be fully reloaded.
    Entirely,
}

publisher_newtype!(GraphChangedPublisher,GraphChanged);
