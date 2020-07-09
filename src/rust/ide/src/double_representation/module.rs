//! Code for module-level double representation processing.

use crate::prelude::*;

use crate::double_representation::definition;
use crate::double_representation::definition::DefinitionProvider;

use ast::crumbs::{ChildAst, Located, ModuleCrumb};
use ast::known;
use enso_protocol::language_server;
use ast::macros::ImportInfo;

struct Info {
    ast:known::Module,
}

impl Info {
    pub fn iter_imports<'a>(&'a self) -> impl Iterator<Item=(ModuleCrumb,ImportInfo)> + 'a {
        let children = self.ast.shape().enumerate();
        children.filter_map(|(crumb,ast)| {
            ast::macros::ast_as_import(ast).map(|import| (crumb,import))
        })
    }

    pub fn lines(&self) -> &Vec<ast::BlockLine<Option<Ast>>> {
        &self.ast.shape().lines
    }

    pub fn non_empty_lines<'a>(&'a self) -> impl Iterator<Item=(usize,&'a Ast)> + 'a {
        self.lines().iter().enumerate().filter_map(|(index,line)| {
            line.elem.as_ref().and_then(|ast| Some((index,ast)))
        })
    }

    pub fn line(&self, index:usize) -> &ast::BlockLine<Option<Ast>> {
        &self.ast.shape().lines[index]
    }

    // pub fn leading_import_lines(&self) -> Range<usize> {
    //     // let non_empty_lines = self.lines().iter().enumerate().filter_map(|(index,line)| {
    //     //     line.elem.and_then(|ast| Some((index,ast)))
    //     // }).collect_vec();
    //
    //
    //     //
    //     // let (first_import,first_non_import) = || {
    //     //     let mut first_import = None;
    //     //     let mut first_non_import = None;
    //     //     for (index,ast) in &non_empty_lines {
    //     //         if ast::macros::is_import(ast) {
    //     //             first_import.get_or_insert(index)
    //     //         } else {
    //     //             first_non_import.get_or_insert(index)
    //     //         }
    //     //     }
    //     // }();
    //
    //     // let first_import = non_empty_lines.iter().find_map(|(index,ast)| {
    //     //     ast::macros::is_import(ast).then(index).copied()
    //     // });
    //     //
    //     // let first_non_import = non_empty_lines.iter().find_map(|(index,ast)| {
    //     //     ast::macros::is_import(ast).then(index).copied()
    //     // });
    //     //
    //     // let first_import_index = match first_non_empty {
    //     //     (index,ast) if ast::macros::if_import(ast) => index,
    //     //     _                                          => return 0..0,
    //     // };
    //     //
    //     // let last_import_index = {
    //     //     let mut index = first_import_index+1;
    //     //     while let Some(index,)
    //     // }
    //
    //
    //     // let mut index = 0;
    //     // while let Some()
    //
    //     // for index in 0..lines.len() {
    //     //     if let Some(line_ast) = &self.line(index).elem {
    //     //         if ast::macros::ast_as_import(line_ast).is_some() {
    //     //
    //     //         }
    //     //     }
    //     // }
    //     //
    //     //
    //     // let first_non_import = self.lines().iter().position(|line| {
    //     //     line.elem.and_then(ast::macros::ast_as_import).is_some()
    //     // });
    // }

    pub fn imports(&self) -> Vec<ImportInfo> {
        self.iter_imports().map(|(_,import)| import).collect()
    }



    pub fn remove_import(&mut self, segments:impl IntoIterator<Item:Into<String>>) -> Option<ImportInfo> {
        let searched_segments = segments.into_iter().map(|s| s.into()).collect_vec();
        let (crumb,import) = self.iter_imports().find(|(_,import)| {
            import.segments == searched_segments
        })?;
        self.remove_line(crumb.line_index);
        Some(import)
    }

    pub fn remove_line(&mut self, index:usize) {
        self.ast.update_shape(|shape| { shape.lines.remove(index); })
    }

    pub fn add_import(&mut self, segments:impl IntoIterator<Item:Into<String>>) -> usize {
        let previous_import = 5;
        for ((crumb1,import1),(crumb2,import2)) in self.iter_imports().tuples() {

        }
        4
    }

    pub fn add_line(&mut self, index:usize, ast:Option<Ast>) {
        // TODO
        //self.ast.update_shape(|shape| { shape.lines.remove(index); })
    }
}
//
// struct ModuleHeader {
//     first_import : usize,
//     last_import  : usize,
// }
//
// impl ModuleHeader {
//     fn new(lines:&[(usize,Ast)]) -> Option<ModuleHeader> {
//         let mut first_import = None;
//         let mut last_import = None;
//
//         for (index,ast) in lines {
//             if ast::macros::is_import(ast) {
//                 first_import.get_or_insert(*index);
//                 last_import = Some(*index);
//             } else {
//                 break;
//             }
//         }
//
//         first_import.and_then(|first_import| {
//             last_import.map(|last_import| ModuleHeader {first_import,last_import})
//         })
//     }
// }
//
// fn leading_imports(lines:&[(usize,Ast)]) -> Vec<(usize,Ast)> {
//     let mut first_import = None;
//     let mut last_import = None;
//
//     for (index,ast) in lines {
//         if ast::macros::is_import(ast) {
//             first_import.get_or_insert(*index);
//             last_import = Some(*index);
//         } else {
//             break;
//         }
//     }
//
//     first_import.and_then(|first_import| {
//         last_import.map(|last_import| ModuleHeader {first_import,last_import})
//     })
// }

// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[derive(Fail,Clone,Debug)]
#[fail(display="Cannot find method with pointer {:?}.",_0)]
pub struct CannotFindMethod(language_server::MethodPointer);

#[allow(missing_docs)]
#[derive(Copy,Fail,Clone,Debug)]
#[fail(display="Encountered an empty definition ID. It must contain at least one crumb.")]
pub struct EmptyDefinitionId;



// ========================
// === Module Utilities ===
// ========================

/// Looks up graph in the module.
pub fn get_definition
(ast:&known::Module, id:&definition::Id) -> FallibleResult<definition::DefinitionInfo> {
    Ok(locate(ast, id)?.item)
}

/// Traverses the module's definition tree following the given Id crumbs, looking up the definition.
pub fn locate
(ast:&known::Module, id:&definition::Id) -> FallibleResult<definition::ChildDefinition> {
    let mut crumbs_iter = id.crumbs.iter();
    // Not exactly regular - we need special case for the first crumb as it is not a definition nor
    // a children. After this we can go just from one definition to another.
    let first_crumb = crumbs_iter.next().ok_or(EmptyDefinitionId)?;
    let mut child = ast.def_iter().find_by_name(&first_crumb)?;
    for crumb in crumbs_iter {
        child = definition::resolve_single_name(child,crumb)?;
    }
    Ok(child)
}

/// Get a definition ID that points to a method matching given pointer.
///
/// The module is assumed to be in the file identified by the `method.file` (for the purpose of
/// desugaring implicit extensions methods for modules).
pub fn lookup_method
(ast:&known::Module, method:&language_server::MethodPointer) -> FallibleResult<definition::Id> {
    let module_path = model::module::Path::from_file_path(method.file.clone())?;
    let explicitly_extends_looked_type = method.defined_on_type == module_path.module_name();

    for child in ast.def_iter() {
        let child_name : &definition::DefinitionName = &child.name.item;
        let name_matches = child_name.name.item == method.name;
        let type_matches = match child_name.extended_target.as_slice() {
            []         => explicitly_extends_looked_type,
            [typename] => typename.item == method.defined_on_type,
            _          => child_name.explicitly_extends_type(&method.defined_on_type),
        };
        if name_matches && type_matches {
            return Ok(definition::Id::new_single_crumb(child_name.clone()))
        }
    }

    Err(CannotFindMethod(method.clone()).into())
}

impl DefinitionProvider for known::Module {
    fn indent(&self) -> usize { 0 }

    fn scope_kind(&self) -> definition::ScopeKind { definition::ScopeKind::Root }

    fn enumerate_asts<'a>(&'a self) -> Box<dyn Iterator<Item = ChildAst<'a>>+'a> {
        self.ast().children()
    }
}



// ============
// === Test ===
// ============

#[cfg(test)]
mod tests {
    use super::*;

    use crate::double_representation::definition::DefinitionName;

    use enso_protocol::language_server::MethodPointer;
    use enso_protocol::language_server::Path;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn implicit_method_resolution() {
        let parser = parser::Parser::new_or_panic();
        let foo_method = MethodPointer {
            defined_on_type : "Main".into(),
            file            : Path::new(default(),&["src","Main.enso"]),
            name            : "foo".into(),
        };

        let expect_find = |code,expected:definition::Id| {
            let module = parser.parse_module(code,default()).unwrap();
            let result = lookup_method(&module,&foo_method);
            assert_eq!(result.unwrap().to_string(),expected.to_string());

            // TODO [mwu]
            //  We should be able to use `assert_eq!(result.unwrap(),expected);`
            //  But we can't, because definition::Id uses located fields and crumbs won't match.
            //  Eventually we'll likely need to split definition names into located and unlocated
            //  ones. Definition ID should not require any location info.
        };

        let expect_not_found = |code| {
            let module = parser.parse_module(code,default()).unwrap();
            lookup_method(&module,&foo_method).expect_err("expected method not found");
        };

        // Implicit module extension method.
        let id = definition::Id::new_plain_name("foo");
        expect_find("foo a b = a + b", id);
        // Explicit module extension method
        let id = definition::Id::new_single_crumb(DefinitionName::new_method("Main","foo"));
        expect_find("Main.foo a b = a + b", id);

        expect_not_found("bar a b = a + b");
    }
}
