//! TODO TODO

use crate::prelude::*;

use ast::*;

use ast::opr;
use ast::prefix;

/// Describes the kind of code block (scope).
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum ScopeKind {
    /// Module scope is a file's top-level block.
    Root,
    /// Any other block, e.g. introduced as body of some definition.
    NonRoot,
}



// ==================
// === Assignment ===
// ==================

/// Checks if given Ast is an assignment operator identifier.
pub fn is_assignment_opr(ast:&Ast) -> bool {
    let opr_opt = known::Opr::try_from(ast);
    opr_opt.map(|opr| opr.name == opr::predefined::ASSIGNMENT).unwrap_or(false)
}

/// If given Ast is an assignment operator, returns it as Some known::Infix.
pub fn to_assignment(ast:&Ast) -> Option<known::Infix> {
    let infix = known::Infix::try_from(ast).ok()?;
    is_assignment_opr(&infix.opr).then(infix)
}



// ==================
// === Identifier ===
// ==================

/// Checks if given Ast node can be used to represent identifier (that is e.g.
/// used to name a defined function).
pub fn is_identifier(ast:&Ast) -> bool {
    match ast.shape() {
        Shape::Var          {..} => true,
        Shape::Cons         {..} => true,
        Shape::SectionSides {..} => true,
        Shape::Opr          {..} => true,
        _                        => false,
    }
}

/// Retrieves the identifier's name, if the Ast node is an identifier. Otherwise, returns None.
pub fn identifier_name(ast:&Ast) -> Option<String> {
    is_identifier(ast).then_with(|| ast.repr())
}



// ======================
// === DefinitionName ===
// ======================

/// Structure representing definition name. If this is an extension method, extended type is
/// also included.
#[derive(Clone,Debug)]
pub struct DefinitionName {
    /// Used when definition is an extension method. Then it stores the segments
    /// of the extended target type path.
    pub extended_target : Vec<String>,
    /// Name of the function itself.
    pub name : String,
}

impl DefinitionName {
    /// Tries describing given Ast piece as a definition name. Typically, passed Ast
    /// should be the binding's left-hand side.
    ///
    /// Returns `None` if is not name-like entity.
    pub fn from_ast(ast:&Ast) -> Option<DefinitionName> {
        let accessor_chain = opr::Chain::try_new_of(ast,opr::predefined::ACCESS);
        let (extended_target,name) = match accessor_chain {
            Some(accessor_chain) => {
                let mut args = vec![identifier_name(&accessor_chain.target?)?];
                for arg in accessor_chain.args.iter() {
                    let arg_ast = arg.1.as_ref()?;
                    args.push(identifier_name(arg_ast)?)
                }
                let name = args.pop()?;
                (args,name)
            }
            None => {
                (Vec::new(), identifier_name(ast)?)
            }
        };
        Some(DefinitionName {extended_target,name})
    }
}

impl ToString for DefinitionName {
    fn to_string(&self) -> String {
        let mut pieces = self.extended_target.iter().map(|s| s.as_str()).collect_vec();
        pieces.push(&self.name);
        pieces.join(opr::predefined::ACCESS)
    }
}



// ======================
// === DefinitionInfo ===
// ======================

/// Information about definition binding.
#[derive(Clone,Debug)]
pub struct DefinitionInfo {
    /// The whole definition. It is an Infix shape with `=` operator. Its left-hand side is
    /// an App.
    pub ast: known::Infix,
    /// Name of this definition. Includes typename, if this is an extension method.
    pub name: DefinitionName,
    /// Arguments for this definition. Does not include any implicit ones (e.g. no `this`).
    pub args: Vec<Ast>,
}

/// Tries to interpret given `Ast` as a function definition. `Ast` is expected to represent the
/// whole line of the program.
pub fn get_definition_info(ast:&Ast, kind:ScopeKind) -> Option<DefinitionInfo> {
    let ast  = to_assignment(ast)?;
    let lhs  = prefix::Chain::new_non_strict(&ast.larg);
    let name = DefinitionName::from_ast(&lhs.func)?;
    let args = lhs.args;
    let ret  = DefinitionInfo {ast,name,args};

    // Note [Scope Differences]
    if kind == ScopeKind::NonRoot {
        // 1. Not an extension method but setter.
        if !ret.name.extended_target.is_empty() {
            None?
        }
        // 2. No args -- this is a node, not a definition.
        if ret.args.is_empty() {
            None?;
        }
    };

    Some(ret)
}

// Note [Scope Differences]
// ========================
// When we are in definition scope (as opposed to global scope) certain patterns should not be
// considered to be function definitions. These are:
// 1. Expressions like "Int.x = …". In module, they'd be treated as extension methods. In
//    definition scope they are treated as accessor setters.
// 2. Expression like "foo = 5". In module, this is treated as method definition (with implicit
//    this parameter). In definition, this is just a node (evaluated expression).

/// List all definition in the given block.
pub fn get_definition_infos
(lines:&Vec<BlockLine<Option<Ast>>>,kind:ScopeKind) -> Vec<DefinitionInfo> {
    let opt_defs = lines.iter().map(|line| -> Option<DefinitionInfo> {
        let ast = line.elem.as_ref()?;
        get_definition_info(ast,kind)
    });
    opt_defs.flatten().collect()
}

/// Returns information for all definition defined in the module's root scope.
pub fn list_definitions_in_module_block(module:&Module<Ast>) -> Vec<DefinitionInfo> {
    get_definition_infos(&module.lines,ScopeKind::Root)
}

/// Returns information for all definition defined in the (non-root) block's scope.
pub fn list_definitions_in_definition_block(block:&Block<Ast>) -> Vec<DefinitionInfo> {
    get_definition_infos(&block.lines,ScopeKind::NonRoot)
}

#[test]
fn list_definition_test() {
    // TODO [mwu]
    //  Due to parser bug, extension methods defining operators cannot be currently
    //  correctly recognized. When it is fixed, the following should be also supported
    //  and covered in test: `Int.+ a = _` and `Int.+ = _`.
    //  Issue link: https://github.com/luna/enso/issues/565


    let definition_lines = vec!{
        "main = _",
        "Foo.Bar.foo = _",
        "Foo.Bar.baz a b = _",
        "+ = _",
        "bar = _",
        "Baz = _",
        "add a b = _",
        "* a b = _",
    };

    let mut parser  = parser::Parser::new_or_panic();

    let program     = definition_lines.join("\n");
    let ast         = parser.parse(program.into(),default()).unwrap();
    let module      = &known::Module::try_from(&ast).unwrap();
    let definitions = list_definitions_in_module_block(module);
    println!("{:?}", definitions.iter().map(|d| d.name.to_string()).collect_vec());
}

