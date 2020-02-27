
use crate::prelude::*;

use ast::*;
use parser::api::IsParser;

mod known_operators {
    pub const ASSIGNMENT : &str = "=";
}

#[derive(Clone,Copy,Debug)]
pub enum BlockKind {
    Module,
    Definition,
}

pub fn is_identifier(ast:&Ast) -> bool {
    match ast.shape() {
        Shape::Var          {..} => true,
        Shape::Cons         {..} => true,
        Shape::SectionSides {..} => true,
        Shape::Opr          {..} => true,
        _                        => false,
    }
}

pub fn identifier_name(ast:&Ast) -> Option<String> {
    is_identifier(ast).then_with(|| ast.repr())
}

///// An ast that can serve as an identifier.
//pub enum Identifier {
//    Var(known::Var),
//    Cons(known::Cons),
//    Sides(known::SectionSides),
//}
//
//impl Identifier {
//    pub fn try_new(ast:Ast) -> Option<Identifier> {
//        if let Ok(var) = known::Var::try_from(&ast) {
//            Some(Identifier::Var(var))
//        } else if let Ok(cons) = known::Cons::try_from(&ast) {
//            Some(Identifier::Cons(cons))
//        } else if let Ok(sides) = known::SectionSides::try_from(&ast) {
//            Some(Identifier::Sides(sides))
//        } else {
//            None
//        }
//    }
//}
//
//impl HasRepr for Identifier {
//    fn write_repr(&self, mut target:&mut String) {
//        match self {
//            Identifier::Var(var) => var.write_repr(&mut target),
//            Identifier::Cons(cons) => cons.write_repr(&mut target),
//            Identifier::Sides(sides) => sides.write_repr(&mut target),
//        }
//    }
//}

#[derive(Clone,Debug)]
pub struct DefinitionName {
    /// Used when definition is an extension method. Then it stores the segments
    /// of the extended target type path.
    pub extended_target : Vec<String>,
    /// Name of the function itself.
    pub name : String,
}


impl DefinitionName {
    pub fn from_ast(ast:&Ast) -> Option<DefinitionName> {
        let name = identifier_name(ast)?;
        Some(DefinitionName {
            extended_target: default(),
            name,
        })
    }
}

#[derive(Clone,Debug)]
/// Result of flattening a sequence of prefix applications.
pub struct FlattenedPrefix {
    /// The function (initial application target)
    pub func : Ast,
    /// Subsequent arguments applied over the function.
    pub args : Vec<Ast>
}

impl FlattenedPrefix {
    /// Translates calls like `a b c` that generate nested prefix chain like
    /// App(App(a,b),c) into flat list where first element is the function and
    /// then arguments are placed: `{func:a, args:[b,c]}`.
    pub fn new(ast:&known::Prefix) -> FlattenedPrefix {
        fn run(ast:&known::Prefix, mut acc: &mut Vec<Ast>) {
            match known::Prefix::try_from(&ast.func) {
                Ok(lhs_app) => run(&lhs_app, &mut acc),
                _           => acc.push(ast.func.clone()),
            }
            acc.push(ast.arg.clone())
        }

        let mut parts = Vec::new();
        run(ast,&mut parts);

        let func = parts.remove(0);
        let args = parts; // remaining parts are args
        FlattenedPrefix {func,args}
    }

    /// As new but if the AST is not a prefix, interprets is a function with an
    /// empty arguments list.
    pub fn new_non_strict(ast:&Ast) -> FlattenedPrefix {
        if let Ok(ref prefix) = known::Prefix::try_from(ast) {
            Self::new(prefix)
        } else {
            let func = ast.clone();
            let args = Vec::new();
            FlattenedPrefix {func,args}
        }
    }
}

pub enum Assoc {Left,Right}

pub fn is_applicative(operator:&str) -> bool {
    let pattern = "<?[+*$]>?";//.r
    false
}

pub fn char_assoc(c:char) -> i32 {
    match c {
        '=' => -1,
        ',' => -1,
        '>' => -1,
        '<' =>  1,
        _   =>  0,
    }
}

impl Assoc {
    pub fn of(operator:&str) -> Assoc {
        if is_applicative(operator) {
            Assoc::Left
        } else if operator.chars().map(char_assoc).sum() >= 0 {
            Assoc::Left
        } else {
            Assoc::Right
        }
    }
}





//pub fn flatten_prefix(ast:&known::Prefix) -> Vec<Ast> {
//    fn run(ast:&known::Prefix, mut acc: &mut Vec<Ast>) {
//        match known::Prefix::try_from(&ast.func) {
//            Ok(lhs_app) => run(&lhs_app, &mut acc),
//            _           => acc.push(ast.func.clone()),
//        }
//        acc.push(ast.arg.clone())
//    }
//
//    let mut ret = Vec::new();
//    run(ast,&mut ret);
//    ret
//}

/// Checks if given Ast is an assignment operator identifier.
pub fn is_assignment_opr(ast:&Ast) -> bool {
    let opr_opt = known::Opr::try_from(ast);
    opr_opt.map(|opr| opr.name == known_operators::ASSIGNMENT).unwrap_or(false)
}

pub fn to_assignment(ast:&Ast) -> Option<known::Infix> {
    let infix = known::Infix::try_from(ast).ok()?;
    is_assignment_opr(&infix.opr).then(infix)
}

/// Tries to interpret given `Ast` as a function definition. `Ast` is expected to represent the
/// whole line of the program.
pub fn to_definition(ast:&Ast) -> Option<DefinitionInfo> {
    let ast  = to_assignment(ast)?;
    let lhs  = FlattenedPrefix::new_non_strict(&ast.larg);
    let name = DefinitionName::from_ast(&lhs.func)?;
    let args = lhs.args;
    Some(DefinitionInfo {ast,name,args})
}




///// Tries to interpret given `Ast` as a function definition. `Ast` is expected to represent the
///// whole line of the program.
//pub fn to_definition_with_args(ast:&Ast) -> Option<DefinitionInfo> {
//    let ast         = to_assignment(ast)?;
//    let lhs_app     = known::Prefix::try_from(&ast.larg).ok()?;
//    let lhs_parts   = flatten_prefix(&lhs_app);
//    let first_piece = lhs_parts.first()?;
//    let name        = DefinitionName::from_ast(first_piece)?;
//    let args = {
//        let mut args = lhs_parts;
//        args.remove(0);
//        args
//    };
//    Some(DefinitionInfo {ast,name,args})
//}
//
///// Tries to interpret given `Ast` as a function definition. `Ast` is expected to represent the
///// whole line of the program.
//pub fn to_definition_without_args(ast:&Ast) -> Option<DefinitionInfo> {
//    let ast  = to_assignment(ast)?;
//    let name = DefinitionName::from_ast(&ast.larg)?;
//    let args = default();
//    Some(DefinitionInfo {ast,name,args})
//}

pub fn list_definitions_in_module_block(module:&Module<Ast>) -> Vec<DefinitionInfo> {
    let lines = module.lines.iter();
    let opt_defs = lines.map(|line| {
        let opt_ast_ref = line.elem.as_ref();
        println!("Line ast: {:?}\n\n", opt_ast_ref);
        opt_ast_ref.and_then(|ast| {
            to_definition(&ast)
        })
    });
    opt_defs.flatten().collect()
}

#[derive(Clone,Debug)]
pub struct DefinitionInfo {
    /// The whole definition. It is an Infix shape with `=` operator. Its left-hand side is
    /// an App.
    pub ast: known::Infix,
    pub name: DefinitionName,
    pub args: Vec<Ast>,
}

#[test]
fn list_definition_test() {
    let program = r"main = _
+ = _
Foo.foo = _
bar = _
Baz = _
add a b = _";

    let mut parser = parser::Parser::new_or_panic();
    let ast        = parser.parse(program.into(),default()).unwrap();
    let module     = &known::Module::try_from(&ast).unwrap();
    let definitions = list_definitions_in_module_block(module);

    println!("{:?}", definitions);


    println!("{:?}", definitions.iter().map(|d| d.name.clone()).collect_vec());
}

