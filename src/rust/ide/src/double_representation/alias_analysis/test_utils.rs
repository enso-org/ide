
use crate::prelude::*;

use crate::double_representation::alias_analysis::NormalizedName;
use crate::double_representation::alias_analysis::LocatedIdentifier;
use crate::double_representation::node::NodeInfo;

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
        let code         = regex.replace_all(marked_code.as_ref(), replacer.by_ref()).into();
        Case {
            code,
            introduced : replacer.introduced,
            used       : replacer.used,
        }
    }

    /// Lists names of the used or introduced identifiers
    pub fn names(&self, kind:Kind) -> Vec<String> {
        let spans = match kind {
            Kind::Introduced => &self.introduced,
            Kind::Used       => &self.used,
        };

        spans.iter().map(|span| {
            self.code[span.range()].into()
        }).collect()
    }
}



// ========================
// === MarkdownReplacer ===
// ========================

/// We want to recognize two kinds of marked identifiers: ones introduced into the graph's scope and
/// ones used from the graph's scope.
#[derive(Clone,Copy,Debug,Display)]
pub enum Kind { Introduced, Used }

/// Name of the pattern group matching introduced identifier.
const INTRODUCED:&str="introduced";

/// Name of the pattern group matching used identifier.
const USED:&str="used";

/// Replacer that is called with each marked token. Does the following:
/// * removes the markdown, i.e. replaces `»foo«` with `foo`;
/// * counts removed markdown bytes, so it is possible to translate between indices in marked and
///   unmarked code;
/// * accumulates spans of introduced and used identifiers.
#[derive(Debug,Default)]
struct MarkdownReplacer {
    markdown_bytes_consumed : usize,
    /// Indices in the unmarked code.
    introduced              : Vec<Span>,
    /// Indices in the unmarked code.
    used                    : Vec<Span>,
}

impl MarkdownReplacer {
    fn marked_to_unmarked_index(&self, i:usize) -> usize {
        assert!(self.markdown_bytes_consumed <= i);
        i - self.markdown_bytes_consumed
    }
    /// Increments the consumed marker bytes count by size of a single marker character.
    fn consume_marker(&mut self) {
        self.markdown_bytes_consumed += '«'.len_utf8();
    }
    /// Consumes opening and closing marker. Returns span of marked item in unmarked text indices.
    fn consume_marked(&mut self, capture:&Match) -> Span {
        self.consume_marker();
        let start = self.marked_to_unmarked_index(capture.start());
        let end   = self.marked_to_unmarked_index(capture.end());
        self.consume_marker();
        (start .. end).into()
    }
}

// Processes every single match for a marged entity.
impl Replacer for MarkdownReplacer {
    fn replace_append(&mut self, captures: &Captures, dst: &mut String) {
        let (kind,matched) = if let Some(introduced) = captures.name(INTRODUCED) {
            (Kind::Introduced,introduced)
        } else if let Some(used) = captures.name(USED) {
            (Kind::Used,used)
        } else {
            panic!("Unexpected capture: expected named capture `{}` or `{}`.",INTRODUCED,USED)
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



// =========================
// === IdentifierChecker ===
// =========================

#[derive(Clone,Copy,Debug,Display,PartialEq)]
enum IsValidated { No, Yes }

/// Helper test structure that requires that each given identifier is validated at least once.
/// Otherwise, it shall panic when dropped.
#[derive(Clone,Debug)]
pub struct IdentifierValidator(HashMap<NormalizedName, IsValidated>);

impl IdentifierValidator {
    /// Creates a new checker, with identifier set obtained from given node's representation
    /// spans.
    pub fn new(node:&NodeInfo,spans:&Vec<Span>) -> IdentifierValidator {
        let ast     = node.ast();
        let repr    = ast.repr();
        let mut map = HashMap::default();
        for span in spans {
            let name = NormalizedName::new(&repr[span.range()]);
            map.insert(name, IsValidated::No);
        }
        IdentifierValidator(map)
    }

    /// Marks given identifier as checked.
    pub fn validate_identifier(&mut self, name:&NormalizedName) {
        println!("Used: {}", name.name);
        let used = self.0.get_mut(&name).expect(&iformat!("unexpected identifier {name}"));
        *used = IsValidated::Yes;
    }

    /// Marks given sequence of identifiers as checked.
    pub fn validate_identifiers<'a>
    (&mut self, identifiers:impl IntoIterator<Item=&'a LocatedIdentifier>) {
        for identifier in identifiers {
            self.validate_identifier(&identifier.item)
        }
    }
}

/// Panics if there are remaining identifiers that were not checked.
impl Drop for IdentifierValidator {
    fn drop(&mut self) {
        println!("dropping usage map: {:?}", self);
        for elem in &self.0 {
            assert_eq!(elem.1, &IsValidated::Yes, "identifier `{}` was not validated)", elem.0)
        }
    }
}



// =============
// === Tests ===
// =============

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
        assert_eq!(&code[case.introduced[0].range()], "sum");

        assert_eq!(case.used.len(), 2);
        assert_eq!(case.used[0], 6..7);
        assert_eq!(&code[case.used[0].range()], "a");
        assert_eq!(case.used[1], 10..11);
        assert_eq!(&code[case.used[1].range()], "b");
    }
}