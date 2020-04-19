//! A module with all functions used to synchronize different representations of our language
//! module.

pub mod alias_analysis;
pub mod connection;
pub mod definition;
pub mod graph;
pub mod node;
pub mod text;



// ==============
// === Consts ===
// ==============

/// Indentation value from language specification:
///
/// Indentation: Indentation is four spaces, and all tabs are converted to 4 spaces. This is not
/// configurable on purpose.
///
/// Link: https://github.com/luna/enso/blob/master/doc/design/syntax/syntax.md#encoding
pub const INDENT : usize = 4;


#[cfg(test)]
pub mod test_utils {
    use crate::prelude::*;

    use regex::Captures;
    use regex::Match;

    /// Helper type for markdown-defined test cases with `regex` library.
    #[derive(Clone,Copy,Debug,Default)]
    pub struct MarkdownProcessor {
        markdown_bytes_consumed : usize,
    }

    impl MarkdownProcessor {
        /// Convert index from marked to unmarked code.
        fn marked_to_unmarked_index(&self, i:usize) -> usize {
            assert!(self.markdown_bytes_consumed <= i);
            i - self.markdown_bytes_consumed
        }

        /// Convert indices range from marked to unmarked code.
        fn marked_to_unmarked_range(&self, range:Range<usize>) -> Range<usize> {
            Range {
                start : self.marked_to_unmarked_index(range.start),
                end   : self.marked_to_unmarked_index(range.end),
            }
        }

        /// Assumes that given match is the part of capture that should be passed to the dst string.
        pub fn process_match
        (&mut self, captures:&Captures, body:&Match, dst:&mut String) -> Range<usize> {
            let whole_match      = captures.get(0).expect("Capture 0 should always be present.");
            let bytes_to_body    = body.start() - whole_match.start();
            let bytes_after_body = whole_match.end() - body.end();
            self.markdown_bytes_consumed += bytes_to_body;
            let ret = self.marked_to_unmarked_range(body.range());
            self.markdown_bytes_consumed += bytes_after_body;
            dst.push_str(body.as_str());
            ret
        }
    }
}
