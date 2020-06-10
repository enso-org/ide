//! A module with functions used to support working with text representation of the language.

use crate::prelude::*;

use ast::IdMap;
use data::text::Size;
use data::text::Span;



// ================
// === Text API ===
// ================

/// Update IdMap to reflect the recent code change.
pub fn apply_code_change_to_id_map(id_map:&mut IdMap, change:&data::text::TextChange, code:&str) {
    let removed   = change.replaced_span();
    let inserted  = change.inserted.as_str();
    let new_code  = change.applied(code);
    let non_white = |c:char| !c.is_whitespace();

    println!("Old code:\n```\n{}\n```",code);
    println!("New code:\n```\n{}\n```",new_code);

    let vector = &mut id_map.vec;
    let inserted_size = Size::new(inserted.chars().count());
    // Remove all entries covered by the removed span.
    vector.drain_filter(|(span,_)| removed.contains_span(&span));

    let logger = Logger::new("IdMapUpdate");

    // If the edited section ends up being the trailing part of AST node, how many bytes should be
    // trimmed from the id. Precalculated, as is constant in the loop below.
    let to_trim_back = {
        let last_non_white           = inserted.rfind(non_white);
        let inserted_len             = || inserted.len();
        let length_to_last_non_white = |index| inserted.len() - index - 1;
        Size::new(last_non_white.map_or_else(inserted_len,length_to_last_non_white))
    };
    // As above but for the front side.
    let to_trim_front = {
        let first_non_white = inserted.find(non_white);
        let ret             = first_non_white.unwrap_or(inserted.len());
        Size::new(ret)
    };

    // In case of collisions (when, after resizing spans, multiple ids for the same span are
    // present), the mappings from this map will be preferred over other ones.
    //
    // This is needed for edits like: `foo f` => `foo` — the earlier `foo` in `foo f` also has a
    // id map entry, however we want it to be consistently shadowed by the id from the whole App
    // expression.
    let mut preferred : HashMap<Span,ast::Id> = default();

    debug!(logger,"Removed: {removed}, inserted length: {inserted_size}");
    debug!(logger,"To trim back {to_trim_back}; inserting {inserted}");
    for (span, id) in vector.iter_mut() {
        let mut trim_front = false;
        let mut trim_back  = false;
        let initial_span   = *span;
        if span.index > removed.end() {
            debug!(logger,"After");
            // AST node starts after edited region — it will be simply shifted.
            let code_between = &code[Span::from(removed.end() .. span.index)];
            span.move_left(removed.size);
            span.move_right(inserted_size);

            // If there are only spaces between current AST symbol and insertion, extend the symbol.
            // This is for cases like line with `foo ` being changed into `foo j`.
            debug!(logger,"Between: `{code_between}`");
            if all_spaces(code_between) {
                debug!(logger,"Will extend front");
                span.extend_left(inserted_size);
                span.extend_left(Size::from(code_between));
                trim_front = true;
            }
        } else if span.index >= removed.index {
            // AST node starts inside the edited region. It doesn't end strictly inside it.
            debug!(logger, "Trailing overlap");
            span.set_left(removed.index);
            span.extend_right(inserted_size);
            trim_front = true;
        } else if span.end() >= removed.index {
            debug!(logger,"Leading overlap");
            // AST node ends before the edit starts.
            span.set_right(removed.index);
            span.extend_right(inserted_size);
            trim_back = true;
        } else {
            // If there are only spaces between current AST symbol and insertion, extend the symbol.
            // This is for cases like line with `foo ` being changed into `foo j`.
            let between = &code[Span::from(span.end() .. removed.index)];
            if all_spaces(between) {
                debug!(logger,"Will extend ");
                span.size += Size::new(between.len()) + inserted_size;
                trim_back = true;
            }
        }

        if trim_front {
            span.index += to_trim_front;
            span.size -= to_trim_front;
            debug!(logger,"Will trim front {to_trim_front}");
        }

        if trim_back {
            span.size -= to_trim_back;

            let new_repr = &new_code[*span];
            // Trim trailing spaces
            let space_count = spaces_size(new_repr.chars().rev());
            debug!(logger,"Will trim back {to_trim_back} and {space_count} spaces");
            debug!(logger,"The would-be code: {new_repr}");
            span.size -= Size::new(space_count);
        }

        // If we edited front or end of an AST node, its extended (or shrunk) span will be
        // preferred.
        if trim_front || trim_back {
            preferred.insert(*span,*id);
        }

        debug!(logger, "Processing for id {id}: {initial_span} ->\t{span}.\n\
        Code: `{&code[initial_span]}` => `{&new_code[*span]}`");
    }

    // If non-preferred entry collides with the preferred one, remove the former.
    vector.drain_filter(|(span,id)| {
        preferred.get(span).map(|preferred_id| id != preferred_id).unwrap_or(false)
    });
}



// ===============
// === Helpers ===
// ===============

/// Returns the byte length of leading space characters sequence.
fn spaces_size(itr:impl Iterator<Item=char>) -> usize {
    itr.take_while(|c| *c == ' ').fold(0, |acc, c| acc + c.len_utf8())
}

/// Checks if the given string slice contains only space charactesr.
fn all_spaces(text:&str) -> bool {
    text.chars().all(|c| c == ' ')
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    use ast::{IdMap, Id, HasIdMap};
    use data::text::{Index, TextChange};
    use data::text::Size;
    use data::text::Span;
    use uuid::Uuid;
    use parser::Parser;
    use enso_prelude::default;
    use crate::double_representation::test_utils::MarkdownProcessor;

    use regex::Captures;
    use regex::Replacer;


    struct Case {
        pub code   : String,
        pub change : TextChange,
    }

    impl Case {
        /// Markdown supports currently a single edit in the given code piece. It must be of form
        /// `«aa⎀bb»` which reads "replace `aa` with `bb`".
        fn from_markdown(marked_code:impl AsRef<str>) -> Case {
            let marked_code = marked_code.as_ref();
            let index_of = |c| marked_code.find(c);

            const START     : char = '«';
            const INSERTION : char = '⎀';
            const END       : char = '»';

            match (index_of(START),index_of(INSERTION),index_of(END)) {
                (Some(start),insertion,Some(end)) => {
                    assert!(start < end,"Markdown markers discovered in wrong order");
                    let erased_finish = insertion.unwrap_or(end);
                    let code          = {
                        let prefix = &marked_code[..start];
                        let erased = &marked_code[start + START.len_utf8() .. erased_finish];
                        let suffix = &marked_code[end   + END.  len_utf8() .. ];
                        String::from_iter([prefix,erased,suffix].iter().copied())
                    };

                    let inserted_code = insertion.map_or("", |insertion|
                        &marked_code[insertion + INSERTION.len_utf8()..end]
                    );
                    let removed_span = Range {
                        start : Index::new(start),
                        end   : Index::new(erased_finish - START.len_utf8()),
                    };
                    let change = TextChange::replace(removed_span,inserted_code.to_string());
                    Case {code,change}
                }
                _ => panic!("Invalid markdown in code: {}",marked_code),
            }
        }

        /// Code after applying the change
        fn resulting_code(&self) -> String {
            self.change.applied(&self.code)
        }


        fn assert_edit_keeps_node_ids(&self, parser:&Parser) {
            let ast1       = parser.parse_module(&self.code,default()).unwrap();
            let mut id_map = ast1.id_map();

            apply_code_change_to_id_map(&mut id_map,&self.change,&self.code);
            let code2 = self.resulting_code();

            let ast2 = parser.parse_module(&code2,id_map.clone()).unwrap();
            assert_same_node_ids(&ast1,&ast2);
        }
    }

    #[test]
    fn test_markdown() {
        let case = Case::from_markdown("foo«aa⎀bb»c");
        assert_eq!(case.code, "fooaac");
        assert_eq!(case.change.inserted, "bb");
        assert_eq!(case.change.replaced, Index::new(3)..Index::new(5));
        assert_eq!(case.resulting_code(), "foobbc");

        let case = Case::from_markdown("foo«aa»c");
        assert_eq!(case.code, "fooaac");
        assert_eq!(case.change.inserted, "");
        assert_eq!(case.change.replaced, Index::new(3)..Index::new(5));
        assert_eq!(case.resulting_code(), "fooc");
    }

    fn to_main(lines:impl IntoIterator<Item:AsRef<str>>) -> String {
        let mut ret = "main = ".to_string();
        for line in lines {
            ret.push_str(&format!("\n    {}", line.as_ref()))
        }
        ret
    }

    fn main_nodes(module:&ast::known::Module) -> Vec<Uuid> {
        use double_representation::definition::*;
        use double_representation::graph::GraphInfo;
        let id = Id::new_plain_name("main");
        let definition = traverse_for_definition(module,&id).unwrap();
        let graph = GraphInfo::from_definition(definition);
        let nodes = graph.nodes();
        nodes.into_iter().map(|node| node.id()).collect()
    }

    fn assert_same_node_ids(ast1:&ast::known::Module,ast2:&ast::known::Module) {
        let ids1 = main_nodes(ast1);
        let ids2 = main_nodes(ast2);
        println!("IDs1: {:?}", ids1);
        println!("IDs2: {:?}", ids2);
        assert_eq!(ids1,ids2);
    }

    #[test]
    fn applying_code_changes_to_id_map3() {
        let parser = Parser::new_or_panic();

        // All the cases describe edit to a middle line in three line main definition.
        let cases = [
            "a = «⎀f»foo",
            "a = «⎀ »foo",
            "a = «⎀f» foo",
            "a = foo«⎀ »",
            "a = foo«⎀\n»",
            "a = foo «⎀\n»",
            "a = foo «⎀j»",
            "a = foo «j»",
            "a = foo«⎀j»",

            // Same as above but not in an assignment form
            "«⎀f»foo",
            // "«⎀ »foo",  // Note: This would actually break the block (change of indent).
            // "«⎀f» foo", // Note: This would actually break the block (change of indent).
            "foo«⎀ »",
            "foo«⎀\n»",
            "foo «⎀\n»",
            "foo «⎀j»",
            "foo «j»",
            "foo«⎀j»",
        ];

        for case in cases.iter() {
            let all_nodes = ["previous",case,"next"];
            let main_def = to_main(all_nodes.iter());
            let case = Case::from_markdown(main_def);
            case.assert_edit_keeps_node_ids(&parser);
        }
    }

    #[test]
    fn applying_code_changes_to_id_map() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let uuid3 = Uuid::new_v4();
        let uuid4 = Uuid::new_v4();
        let uuid5 = Uuid::new_v4();
        let mut id_map = IdMap::new(vec!
            [ (Span::new(Index::new(0) , Size::new(3)), uuid1)
            , (Span::new(Index::new(5) , Size::new(2)), uuid2)
            , (Span::new(Index::new(7) , Size::new(2)), uuid3)
            , (Span::new(Index::new(9) , Size::new(2)), uuid4)
            , (Span::new(Index::new(13), Size::new(2)), uuid5)
            ]);
        let code = "foo  aa++bb  cc";

        let change = TextChange::replace(Index::new(6)..Index::new(10), "a test".to_string());
        apply_code_change_to_id_map(&mut id_map, &change, code);
        let expected = IdMap::new(vec!
            [ (Span::new(Index::new(0) , Size::new(3)), uuid1)
            , (Span::new(Index::new(5) , Size::new(7)), uuid2)
            , (Span::new(Index::new(12), Size::new(1)), uuid4)
            , (Span::new(Index::new(15), Size::new(2)), uuid5)
            ]);
        assert_eq!(expected, id_map);

        let code = "foo  aa++bb  cc";


        let change = TextChange::replace(Index::new(12)..Index::new(14), "x".to_string());
        apply_code_change_to_id_map(&mut id_map, &change,"");
        let expected = IdMap::new(vec!
            [ (Span::new(Index::new(0) , Size::new(3)), uuid1)
            , (Span::new(Index::new(5) , Size::new(8)), uuid2)
            , (Span::new(Index::new(14), Size::new(2)), uuid5)
            ]);
        assert_eq!(expected, id_map);
    }
}
