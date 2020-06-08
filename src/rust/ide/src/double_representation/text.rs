//! A module with functions used to support working with text representation of the language.

use crate::prelude::*;

use ast::IdMap;
use data::text::Size;
use data::text::Span;

// Returns the length of the trailing whitespace
fn trailing_spaces(code:&str) -> usize {
    code.bytes().rev().take_while(|c| *c == ' ' as u8).count()
}



// ================
// === Text API ===
// ================

pub fn apply_code_change_to_id_map2(id_map:&mut IdMap, change:&data::text::TextChange, code:&str) {
    apply_code_change_to_id_map(id_map,&change.replaced_span(),&change.inserted,code)
}

/// Update IdMap to reflect the recent code change.
pub fn apply_code_change_to_id_map(id_map:&mut IdMap, removed:&Span, inserted:&str, code:&str) {
    // TODO [ao]
    // It's a really minimalistic algorithm, just extends/shrinks span without caring about
    // code structure.
    let vector = &mut id_map.vec;
    let inserted_len  = Size::new(inserted.chars().count());
    // Remove all entries covered by the removed span.
    vector.drain_filter(|(span,_)| removed.contains_span(&span));

    let logger = Logger::new("IdMapUpdate");


    let to_last_non_white = inserted.char_indices()
        .rfind(|(_,c)| !c.is_whitespace())
        .map_or_else(|| 0,|(i,_)| i+1);

    let to_first_non_space = inserted.char_indices()
        .find(|(_,c)| *c != ' ')
        .map_or_else(|| 0,|(i,_)| i);

    debug!(logger,"Removed: {removed}, inserted length: {inserted_len}");
    debug!(logger,"To last non white character {to_last_non_white}; inserting {inserted}");
    for (span, _id) in vector {
        let initial_span = *span;
        if span.index >= removed.end() {
            debug!(logger,"After");
            // Entry starts after edited region — will be simply shifted.
            span.index += inserted_len;
            span.index -= removed.size;
        } else if span.index >= removed.index {
            // symbol zaczyna sie wewnatrz edycji
            debug!(logger, "Trailing overlap");
            // Entry starts in the middle of the region — will be resized.
            let removed_chars = removed.end() - span.index;
            span.index = removed.index + inserted_len;
            span.size -= removed_chars;
        } else if span.end() >= removed.index {
            debug!(logger,"Leading overlap");
            // Entry ends before the edit starts.
            let removed_chars = (span.end() - removed.index).min(removed.size);
            span.size -= removed_chars;
            span.size += Size::new(to_last_non_white);

            // If after edit the symbol would be left with trailing whitespace, we trim it.
            if removed_chars.value > 0 && to_last_non_white == 0 {
                let new_code = &code[*span];
                span.size -= Size::new(trailing_spaces(new_code));
            }
        } else {
            // If there are only spaces between current AST symbol and insertion, extend the symbol.
            // This is for cases like line with `foo ` being changed into `foo j`.
            let between     = &code[Span::from(span.end() .. removed.index)];
            let only_spaces = between.chars().all(|c| c == ' ');
            if only_spaces {
                span.size += Size::new(between.len());
                span.size += Size::new(to_last_non_white);
            }
        }

        debug!(logger, "Processing for id {_id}: {initial_span} ->\t{span}. Code: {&code[initial_span]}");
    }
}

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

    /// For each Ast node record the id or lack of such.
    ///
    /// The order is same as used by `iter_recursive` method.
    fn record_ids(ast:impl Into<Ast>) -> Vec<Option<Id>> {
        ast.into().iter_recursive().map(|ast| ast.id).collect()
    }

    fn assert_same_ids(mut ids:&[Option<Id>], ast:impl Into<Ast>) {
        let ast = ast.into();
        ast.iter_recursive().for_each(|ast| {
            let expected_id = ids[0];
            ids = &ids[1..];
            assert_eq!(ast.id, expected_id, "id mismatch at ast {}: {:?}",ast,ast);
        });
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

    fn edit_keeps_node_ids(parser:&Parser, code:&str, change:&TextChange) {

        let ast1       = parser.parse_module(code,default()).unwrap();
        let mut id_map = ast1.id_map();

        apply_code_change_to_id_map2(&mut id_map, change, code);
        let code2 = change.applied(code);

        println!("Old code:\n```\n{}\n```",code);
        println!("New code:\n```\n{}\n```",code2);


        let ast2 = parser.parse_module(&code2,id_map.clone()).unwrap();
        assert_same_node_ids(&ast1,&ast2);
    }


    fn print_position(ast:&Ast) {
        ast::traverse_with_span(ast,|span,ast| {
            println!(" * {} => {}",span,ast);
        });
    }

    #[test]
    fn applying_code_changes_to_id_map3() {
        let parser = Parser::new_or_panic();

        let mut code = "main = \n    foo \n    bar";
        let change = TextChange::insert(Index::new(15),"\n".to_string());
        edit_keeps_node_ids(&parser,&code,&change);

        let mut code = "main = \n    foo \n    bar";
        let change = TextChange::insert(Index::new(16),"j".to_string());
        edit_keeps_node_ids(&parser,&code,&change);

        let mut code = "main = \n    foo j\n    bar";
        let change = TextChange::delete(Index::new(16)..Index::new(17));
        edit_keeps_node_ids(&parser,&code,&change);

        let mut code = "main = \n    foo \n    bar";
        let change = TextChange::insert(Index::new(15)," j".to_string());
        edit_keeps_node_ids(&parser,&code,&change);

        let mut code = "main = \n    foo \n    bar";
        let change = TextChange::insert(Index::new(15),"j".to_string());
        edit_keeps_node_ids(&parser,&code,&change);
    }

    #[test]
    fn applying_code_changes_to_id_map2() {
        let parser = Parser::new_or_panic();
        let mut code = "main = \n    foo\n    bar".to_string();
        let ast = parser.parse_module(&code,default()).unwrap();
        let ids = record_ids(ast.ast());
        println!("{:#?}",ids);
        let mut id_map = ast.id_map();
        let ast = parser.parse_module(&code,id_map.clone()).unwrap();
        assert_same_ids(&ids,ast);

        let change = TextChange::insert(Index::new(15),"\n".to_string());
        apply_code_change_to_id_map2(&mut id_map,&change,&code);
        change.apply(&mut code);
        println!("New code: {}",code);

        let ast = parser.parse_module(&code,id_map.clone()).unwrap();

        println!("New AST positions");
        ast::traverse_with_span(&ast,|span,ast| {
            println!(" * {} => {}",span,ast);
        });

        assert_same_ids(&ids,ast.ast());
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

        apply_code_change_to_id_map(&mut id_map, &Span::new(Index::new(6),Size::new(4)), "a test","");
        let expected = IdMap::new(vec!
            [ (Span::new(Index::new(0) , Size::new(3)), uuid1)
            , (Span::new(Index::new(5) , Size::new(7)), uuid2)
            , (Span::new(Index::new(12), Size::new(1)), uuid4)
            , (Span::new(Index::new(15), Size::new(2)), uuid5)
            ]);
        assert_eq!(expected, id_map);

        apply_code_change_to_id_map(&mut id_map, &Span::new(Index::new(12), Size::new(2)), "x","");
        let expected = IdMap::new(vec!
            [ (Span::new(Index::new(0) , Size::new(3)), uuid1)
            , (Span::new(Index::new(5) , Size::new(8)), uuid2)
            , (Span::new(Index::new(14), Size::new(2)), uuid5)
            ]);
        assert_eq!(expected, id_map);
    }
}
