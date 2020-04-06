
use crate::prelude::*;

use data::text::Index;
use data::text::Size;
use data::text::Span;

#[derive(Clone,Debug,PartialEq,Eq)]
struct Markable {
    opener : char,
    closer : char,
}

const INTRODUCED : Markable     = Markable {opener:'«',  closer:'»'};
const USED       : Markable     = Markable {opener:'»',  closer:'«'};

#[derive(Clone,Debug,Default)]
pub struct Case {
    pub introduced : Vec<Span>,
    pub used       : Vec<Span>,
    pub code       : String,
}

#[derive(Clone,Debug,Default)]
struct MarkdownParser {
    result  : Case,
    current : Option<OngoingMatch>,
    index   : usize,
}

impl MarkdownParser {
    fn new() -> MarkdownParser {
        default()
    }

    fn is_in(&self, markable:Markable) -> bool {
        self.current.contains_if(|current_match| current_match.markable == markable)
    }

    fn open(&mut self, markable:Markable) {
        self.current = Some(OngoingMatch::new(self.index, markable))
    }

    fn close(&mut self) {
        if let Some(current_match) = self.current.take() {
            let span = Span::from_indices(Index::new(current_match.begin), Index::new(self.index));
            match current_match.markable {
                INTRODUCED => self.result.introduced.push(span),
                USED       => self.result.used.push(span),
                _          => {}
            }
        }
    }

    fn parse(&mut self, input:impl Str) {
        for c in input.as_ref().chars() {
            let unopened = self.current.is_none();
            if      unopened && c == INTRODUCED.opener { self.open(INTRODUCED) }
            else if unopened && c == USED.opener       { self.open(USED)       }
            else if self.is_in(INTRODUCED) && c == INTRODUCED.closer { self.close() }
            else if self.is_in(USED)       && c == USED.closer       { self.close() }
            else {
                self.result.code.push(c);
                self.index += 1;
            }
        }
    }
}

impl Case {
    pub fn parse(code:impl Str) -> Case {
        let mut parser = MarkdownParser::new();
        parser.parse(code);
        parser.result
    }

    fn used_names(&self) -> Vec<String> {
        self.used.iter().map(|span| {
            self.code[span.index.value .. span.end().value].into()
        }).collect()
    }
}

#[derive(Clone,Debug)]
struct OngoingMatch {
    begin    : usize,
    markable : Markable,
}

impl OngoingMatch {
    pub fn new(begin:usize, markable:Markable) -> OngoingMatch {
        OngoingMatch {begin,markable}
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    use regex::Match;
    use regex::Regex;
    use regex::Replacer;
    use regex::Captures;
    use std::ops::Range;

    #[derive(Default)]
    struct MarkdownReplacer {
        markdown_consumed : usize,
        introduced        : Vec<Range<usize>>,
        used              : Vec<Range<usize>>,
    }
    impl MarkdownReplacer {
        fn to_output_index(&self, i:usize) -> usize {
            assert!(self.markdown_consumed < i);
            i - self.markdown_consumed
        }
        fn consume_marker(&mut self) {
            self.markdown_consumed += '«'.len_utf8();
        }
        fn consume_marked(&mut self, capture:&Match) -> Range<usize> {
            self.consume_marker();
            let start = self.to_output_index(capture.start());
            let end   = self.to_output_index(capture.end());
            self.consume_marker();
            start .. end
        }
    }
    impl Replacer for MarkdownReplacer {
        fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
            if let Some(introduced) = caps.name("introduced") {
                let span = self.consume_marked(&introduced);
                self.introduced.push(span)
            } else if let Some(used) = caps.name("used") {
                let span = self.consume_marked(&used);
                self.used.push(span)
            } else {
                panic!("Unexpected capture: expected named `introduced` or `used`.")
            }
        }
    }

    # [test]
    fn aaaa() {
        // https://regex101.com/r/pboF8O/
        let regexp = r"«(?P<introduced>[^»]*)»|»(?P<used>[^«]*)«";

        let aa = Regex::new(regexp).unwrap();
        let code = "«sum» = »a« + »b«";

        let out = aa.replace_all(code, MarkdownReplacer::default());
        println!("{}",out);

    }
}