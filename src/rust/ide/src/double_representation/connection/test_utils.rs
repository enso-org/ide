use crate::prelude::*;

use crate::double_representation::connection;

use regex::Captures;
use regex::Match;
use regex::Regex;
use regex::Replacer;



// =============
// === Regex ===
// =============

/// Matches constructs like `«id:foo»` or `»0:sum«`.
/// See TODO for details
const REGEX:&str = r"«([^:]*):([^»]*)»|»([^:]*):([^«]*)«";

/// Index of the group with a source endpoint id.
const SRC_ID   : usize = 1;

/// Index of the group with a source endpoint body.
const SRC_BODY : usize = 2;

/// Index of the group with a destination endpoint id.
const DST_ID   : usize = 3;

/// Index of the group with a destination endpoint body.
const DST_BODY : usize = 4;



// ============
// === Case ===
// ============

/// Test case for testing identifier resolution for nodes.
/// Can be expressed using markdown notation, see `from_markdown` method.
#[derive(Clone,Debug,Default)]
pub struct Case {
    /// The code: the text of the block line that is considered to be a node of a graph.
    /// Any markers are already removed.
    pub code:String,
    pub expected_connections:Vec<(Range<usize>,Range<usize>)>,
}

impl Case {
    /// Constructs a test case using a markdown. Input should be text representation of the node's
    /// AST in which all identifiers introduced into the graph's scope are marked like `«foo»`, and
    /// all identifiers used from graph's scope are marked like `»sum«`.
    pub fn from_markdown(marked_code:impl Str) -> Case {
        // As this is test utils, we don't try nicely handling failure nor reusing the compiled
        // regexp between calls to save some cycles.
        let regex        = Regex::new(REGEX).unwrap();
        let mut replacer = MarkdownReplacer::default();
        let code         = regex.replace_all(marked_code.as_ref(), replacer.by_ref()).into();
        println!("Code:\n===\n{}\n===", code);
        println!("Info: {:?}", replacer);

        let MarkdownReplacer{source,destination,..} = replacer; // decompose
        let connections = destination.into_iter().map(|(name,dst)| {
            let err = || iformat!{"missing src for destination {name}"};
            let src = source.get(&name).expect(&err()).clone();
            (src,dst)
        }).collect_vec();
        Case {code,expected_connections:connections}
    }
}



// ========================
// === MarkdownReplacer ===
// ========================

/// We want to recognize two kinds of marked identifiers: ones introduced into the graph's scope and
/// ones used from the graph's scope.
#[derive(Clone,Copy,Debug,Display)]
pub enum Kind {Source,Destination}

/// Replacer that is called with each marked token. Does the following:
/// * removes the markdown, i.e. replaces `»2:foo«` with `foo`;
/// * counts removed markdown bytes, so it is possible to translate between indices in marked and
///   unmarked code;
/// * stores spans representing identifiers usage for connection source and destination endpoints.
#[derive(Debug,Default)]
struct MarkdownReplacer {
    markdown_bytes_consumed : usize,
    source      : HashMap<String,Range<usize>>,
    destination : HashMap<String,Range<usize>>,
}

impl MarkdownReplacer {
    fn marked_to_unmarked_index(&self, i:usize) -> usize {
        assert!(self.markdown_bytes_consumed <= i);
        i - self.markdown_bytes_consumed
    }

    fn push(&mut self, kind:Kind, id:impl Str, capture:&Match) {
        let start   = self.marked_to_unmarked_index(capture.start());
        let end     = self.marked_to_unmarked_index(capture.end());
        let mut vec = match kind {
            Kind::Source      => &mut self.source,
            Kind::Destination => &mut self.destination,
        };
        println!("pushed {}..{}",start,end);
        vec.insert(id.into(),start..end);
    }
}

// Processes every single match for a marked entity.
impl Replacer for MarkdownReplacer {
    fn replace_append(&mut self, captures: &Captures, dst: &mut String) {
        let whole_match          = captures.get(0).expect("Capture 0 should always be present.");
        let Marked {kind,id,body} = Marked::new(captures);
        let bytes_to_body        = body.start() - whole_match.start();
        let bytes_after_body     = whole_match.end() - body.end();
        self.markdown_bytes_consumed += bytes_to_body;
        self.push(kind,id.as_str(),&body);
        self.markdown_bytes_consumed += bytes_after_body;
        dst.push_str(body.as_str());
    }
}



// ==============
// === Marked ===
// ==============

/// Recognizes and splits into pieces captures like `«id:body»` or `»0:sum«`.
struct Marked<'a> {
    kind : Kind,
    id   : Match<'a>,
    body : Match<'a>,
}

impl<'a> Marked<'a> {
    fn new(captures:&'a Captures) -> Marked<'a> {
        let groups = |ix,ix2| captures.get(ix).into_iter().zip(captures.get(ix2)).next();
        if let (Some((id,body))) = groups(SRC_ID,SRC_BODY) {
            Marked {kind:Kind::Source,id,body}
        } else if let (Some((id,body))) = groups(DST_ID,DST_BODY) {
            Marked {kind:Kind::Destination,id,body}
        } else {
            panic!("Internal error: recheck regex behavior for input: {}", &captures[0])
        }
    }
}

#[test]
fn aaa() {
    let code = r"
«2:bar»
«1:sum» = »2:bar«";
    let case = Case::from_markdown(code);
    println!("{:?}",case);
}