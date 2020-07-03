//! Utilities for dealing with `Stream` values in test code.

use crate::prelude::*;

use futures::Stream;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

/// Extensions to the `Stream` trait allowing manual control of the execution by subsequent
/// polling.
pub trait StreamTestExt<S:?Sized + Stream> {
    /// Access the underlying `Stream` in its pinned form, so that it can be `poll`ed.
    fn get_pinned_stream(&mut self) -> Pin<&mut S>;

    /// Polls the stream, performing any available work. If a new value is ready, returns it.
    ///
    /// The stream polled with this method might not be properly waken later.
    /// Once stream finishes, this method must not be called again (unless stream is fused).
    fn manual_poll_next(&mut self) -> Poll<Option<S::Item>> {
        let mut ctx = Context::from_waker(futures::task::noop_waker_ref());
        self.get_pinned_stream().poll_next(&mut ctx)
    }

    /// Asserts that stream has next value ready and returns it.
    ///
    /// Same caveats apply as for `test_poll_next`.
    fn expect_next(&mut self) -> S::Item {
        match self.manual_poll_next() {
            Poll::Pending           => panic!("Stream has no next item available yet."),
            Poll::Ready(Some(item)) => item,
            Poll::Ready(None)       => panic!("Stream ended instead of yielding an expected value.")
        }
    }

    /// Asserts that stream has terminated.
    ///
    /// Same caveats apply as for `test_poll_next`.
    fn expect_terminated(&mut self) {
        match self.manual_poll_next() {
            Poll::Ready(None) => {}
            _                 => panic!("Stream has not terminated."),
        }
    }

    /// Asserts that the next value in the stream is not ready yet.
    ///
    /// Same caveats apply as for `test_poll_next`.
    fn expect_pending(&mut self)
    where S::Item:Debug {
        match self.manual_poll_next() {
            Poll::Pending           => {}
            Poll::Ready(Some(item)) =>
                panic!("There should be no value ready, yet the stream yielded {:?}",item),
            Poll::Ready(None) =>
                panic!("Stream has terminated, while it should be waiting for the next value."),
        }
    }
}

impl<P,S> StreamTestExt<S> for Pin<P>
where P : Unpin  + DerefMut<Target=S>,
      S : ?Sized + Stream {
    fn get_pinned_stream(&mut self) -> Pin<&mut S> {
        self.as_mut()
    }
}
