//! Provides data wrappers for our analytics api. This is intended to ensure we are conscious of
//! whether we are sending public or private data. No private data should be logged at the moment.
//!
//! Note: this is meant to be a little bit un-ergonomic to ensure the data has been vetted by the
//! API user and allow the reader of the code to see the intent behind the data.

/// Wrapper struct for data that can be made public and has no privacy implications.
#[derive(Clone,Copy,Debug)]
pub struct Public<'a> {
    pub data: &'a str
}

impl<'a> Public<'a>{
    pub fn new(data: &'a str) -> Self {
        Public {data}
    }
}
