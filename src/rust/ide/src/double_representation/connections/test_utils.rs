
use crate::prelude::*;

use data::text::Span;

use regex::Captures;
use regex::Match;
use regex::Regex;
use regex::Replacer;



// ============
// === Case ===
// ============

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
    /// Constructs a test case using a markdown. Input should be text representation of the node's
    /// AST in which all identifiers introduced into the graph's scope are marked like `«foo»`, and
    /// all identifiers used from graph's scope are marked like `»sum«`.
    pub fn from_markdown(marked_code:impl Str) -> Case {

        // Regexp that matches either «sth» or »sth« into a group names `introduced` or `used`,
        // respectively. See: https://regex101.com/r/pboF8O/2 for detailed explanation.
        let regex        = format!(r"«(?P<{}>[^»]*)»|»(?P<{}>[^«]*)«",INTRODUCED,USED);
        // As this is test utils, we don't try nicely handling failure nor reusing the compiled
        // regexp between calls to save some cycles.
        let regex        = Regex::new(&regex).unwrap();
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



// ========================
// === MarkdownReplacer ===
// ========================

/// We want to recognize two kinds of marked identifiers: ones introduced into the graph's scope and
/// ones used from the graph's scope.
#[derive(Clone,Copy,Debug,Display)]
enum Kind { Introduced, Used }

/// Name of the pattern group matching introduced identifier.
const INTRODUCED:&str="introduced";

/// Name of the pattern group matching used identifier.
const USED:&str="used";

#[derive(Debug,Default)]
struct MarkdownReplacer {
    markdown_bytes_consumed : usize,
    introduced              : Vec<Span>,
    used                    : Vec<Span>,
}
impl MarkdownReplacer {
    fn to_output_index(&self, i:usize) -> usize {
        assert!(self.markdown_bytes_consumed <= i);
        i - self.markdown_bytes_consumed
    }
    fn consume_marker(&mut self) {
        self.markdown_bytes_consumed += '«'.len_utf8();
    }
    fn consume_marked(&mut self, capture:&Match) -> Span {
        self.consume_marker();
        let start = self.to_output_index(capture.start());
        let end   = self.to_output_index(capture.end());
        self.consume_marker();
        (start .. end).into()
    }
}

impl Replacer for MarkdownReplacer {
    fn replace_append(&mut self, captures: &Captures, dst: &mut String) {
        let (kind,matched) = if let Some(introduced) = captures.name("introduced") {
            (Kind::Introduced,introduced)
        } else if let Some(used) = captures.name("used") {
            (Kind::Used,used)
        } else {
            panic!("Unexpected capture: expected named `introduced` or `used`.")
        };

        let span    = self.consume_marked(&matched);
        let out_vec = match kind {
            Kind::Introduced => &mut self.introduced,
            Kind::Used       => &mut self.used,
        };
        out_vec.push(span);
        dst.push_str(matched.as_str());
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_markdown_to_test_case() {
        let code = "«sum» = »a« + »b«";
        let case = Case::from_markdown(code);
        assert_eq!(case.code, "sum = a + b");
        assert_eq!(case.introduced.len(), 1);
        assert_eq!(case.introduced[0], 0..3); // sum

        assert_eq!(case.used.len(), 2);
        assert_eq!(case.used[0], 6..7);   // a
        assert_eq!(case.used[1], 10..11); // b
    }
}