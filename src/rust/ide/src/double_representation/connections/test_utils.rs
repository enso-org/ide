
use crate::prelude::*;

use data::text::Index;
use data::text::Size;
use data::text::Span;

use regex::Match;
use regex::Regex;
use regex::Replacer;
use regex::Captures;
use std::ops::Range;

/// Test case for testing identifier resolution for nodes.
#[derive(Clone,Debug,Default)]
pub struct Case {
    /// The code: the text of the block line that is considered to be a node of a graph.
    pub code       : String,
    /// List of spans in the code where the identifiers are introduced into the graph's scope.
    pub introduced : Vec<Span>,
    /// List of spans in the code where the identifiers from the graph's scope are used.
    pub used       : Vec<Span>,
}

impl Case {
    pub fn from_markdown(marked_code:impl Str) -> Case {
        // https://regex101.com/r/pboF8O/
        let regexp = Regex::new(r"«(?P<introduced>[^»]*)»|»(?P<used>[^«]*)«").unwrap();


        let mut replacer = MarkdownReplacer::default();
        let code         = regexp.replace_all(marked_code.as_ref(), replacer.by_ref()).into();
        Case {
            code,
            introduced : replacer.introduced,
            used       : replacer.used,
        }
    }

    pub fn used_names(&self) -> Vec<String> {
        self.used.iter().map(|span| {
            self.code[span.index.value .. span.end().value].into()
        }).collect()
    }
}

#[derive(Debug,Default)]
struct MarkdownReplacer {
    markdown_consumed : usize,
    introduced        : Vec<Span>,
    used              : Vec<Span>,
}
impl MarkdownReplacer {
    fn to_output_index(&self, i:usize) -> usize {
        assert!(self.markdown_consumed <= i);
        i - self.markdown_consumed
    }
    fn consume_marker(&mut self) {
        self.markdown_consumed += '«'.len_utf8();
    }
    fn consume_marked(&mut self, capture:&Match) -> Span {
        println!("Consuming marked: {:?}\nState: {:?}", capture,self);
        self.consume_marker();
        let start = self.to_output_index(capture.start());
        let end   = self.to_output_index(capture.end());
        self.consume_marker();
        (start .. end).into()
    }
}
impl Replacer for MarkdownReplacer {
    fn replace_append(&mut self, captures: &Captures, dst: &mut String) {
        println!("Replacing match: {:?}", captures);
        if let Some(introduced) = captures.name("introduced") {
            let span = self.consume_marked(&introduced);
            self.introduced.push(span)
        } else if let Some(used) = captures.name("used") {
            let span = self.consume_marked(&used);
            self.used.push(span)
        } else {
            panic!("Unexpected capture: expected named `introduced` or `used`.")
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;


    # [test]
    fn aaaa() {
        let code = "«sum» = »a« + »b«";
        let case = Case::from_markdown(code);
        println!("{:?}",case);

    }
}