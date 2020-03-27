//! Crumbs for AST. Crumb identifies children node location in AST node. The access should be
//! possible in a constant time.

use crate::prelude::*;

use crate::known;
use crate::Shape;
use utils::fail::FallibleResult;



// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[fail(display = "The crumb refers to line by index that is out of bounds.")]
#[derive(Debug,Fail,Clone,Copy)]
pub struct LineIndexOutOfBounds;

#[allow(missing_docs)]
#[derive(Debug,Fail,Clone)]
#[fail(display = "The line designated by crumb {:?} does not contain any AST. Context AST was {}.",
crumb,repr)]
pub struct LineDoesNotContainAst {
    repr  : String,
    crumb : Crumb,
}

impl LineDoesNotContainAst {
    /// Creates a new instance of error about missing AST in the designated line.
    pub fn new(repr:impl HasRepr, crumb:impl Into<Crumb>) -> LineDoesNotContainAst {
        let repr = repr.repr();
        let crumb = crumb.into();
        LineDoesNotContainAst {repr,crumb}
    }
}

#[derive(Debug,Display,Fail,Clone,Copy)]
struct MismatchedCrumbType;



// =============
// === Crumb ===
// =============

// === Ast ===

/// Sequence of `Crumb`s describing traversal path through AST.
pub type Crumbs = Vec<Crumb>;

/// Crumb identifies location of child AST in an AST node. Allows for a single step AST traversal.
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
#[allow(missing_docs)]
pub enum Crumb {
    Block(BlockCrumb),
    Module(ModuleCrumb),
    Infix(InfixCrumb),
    Prefix(PrefixCrumb),
    SectionLeft(SectionLeftCrumb),
    SectionRight(SectionRightCrumb),
    SectionSides(SectionSidesCrumb),
    Import(ImportCrumb),
    Mixfix(MixfixCrumb),
    Group(GroupCrumb),
    Def(DefCrumb)
}


// === Block ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub enum BlockCrumb {
    /// The first non-empty line in block.
    HeadLine,
    /// Index in the sequence of "rest of" lines (not counting the HeadLine).
    TailLine {tail_index:usize},
}


// === Module ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub struct ModuleCrumb {pub line_index:usize}


// === Import ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub struct ImportCrumb {pub path_index:usize}


// === Mixfix ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub enum MixfixCrumb {
    Name {index:usize},
    Args {index:usize}
}


// === Group ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub struct GroupCrumb;


// === Def ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub enum DefCrumb {
    Name,
    Args {index:usize},
    Body
}


// === Infix ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub enum InfixCrumb {
    LeftOperand,
    Operator,
    RightOperand,
}


// === Prefix ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub enum PrefixCrumb {
    Func,
    Arg
}


// === SectionLeft ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub enum SectionLeftCrumb {
    Arg,
    Opr
}


// === SectionRight ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub enum SectionRightCrumb {
    Opr,
    Arg
}


// === SectionSides ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub struct SectionSidesCrumb;


// === Conversion Traits ===

macro_rules! from_crumb {
    ($id:ident, $crumb_id:ident) => {
        impl From<$crumb_id> for Crumb {
            fn from(crumb:$crumb_id) -> Self {
                Crumb::$id(crumb)
            }
        }

        impl From<&$crumb_id> for Crumb {
            fn from(crumb:&$crumb_id) -> Self {
                Crumb::$id(crumb.clone())
            }
        }
    }
}

from_crumb!{Block,BlockCrumb}
from_crumb!{Module,ModuleCrumb}
from_crumb!{Infix,InfixCrumb}
from_crumb!{Prefix,PrefixCrumb}
from_crumb!{SectionLeft,SectionLeftCrumb}
from_crumb!{SectionRight,SectionRightCrumb}
from_crumb!{SectionSides,SectionSidesCrumb}



// =================
// === Crumbable ===
// =================

/// Interface for items that allow getting/setting stored Ast located by arbitrary `Crumb`.
pub trait Crumbable {
    /// Specific `Crumb` type used by `Self` to locate child Asts.
    type Crumb : Into<Crumb>;

    /// Retrieves `Ast` under the crumb.
    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast>;

    /// Sets `Ast` under the crumb, returns updated entity.
    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> where Self:Sized;

    /// Iterates all valid crumbs available for `self`.
    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a>;

    /// Iterates pairs (crumb,child_ast) for `self`.
    fn enumerate<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::Crumb,&'a Ast)> + 'a> {
        let indices = self.iter_subcrumbs();
        let iter = indices.map(move |crumb| {
            // NOTE Safe if this module is correct - children crumbs are always accessible.
            let child = self.get(&crumb).unwrap();
            (crumb,child)
        });
        Box::new(iter)
    }
}

impl Crumbable for crate::SectionLeft<Ast> {
    type Crumb = SectionLeftCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        let ret = match crumb {
            SectionLeftCrumb::Arg => &self.arg,
            SectionLeftCrumb::Opr => &self.opr
        };
        Ok(ret)
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut ret = self.clone();
        let target  = match crumb {
            SectionLeftCrumb::Arg => &mut ret.arg,
            SectionLeftCrumb::Opr => &mut ret.opr
        };
        *target = new_ast;
        Ok(ret)
    }

    fn iter_subcrumbs(&self) -> Box<dyn Iterator<Item = Self::Crumb>> {
        const CHILDREN: [SectionLeftCrumb; 2] = [SectionLeftCrumb::Arg, SectionLeftCrumb::Opr];
        Box::new(CHILDREN.iter().copied())
    }
}

impl Crumbable for crate::SectionRight<Ast> {
    type Crumb = SectionRightCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        let ret = match crumb {
            SectionRightCrumb::Arg => &self.arg,
            SectionRightCrumb::Opr => &self.opr
        };
        Ok(ret)
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut ret = self.clone();
        let target  = match crumb {
            SectionRightCrumb::Arg => &mut ret.arg,
            SectionRightCrumb::Opr => &mut ret.opr
        };
        *target = new_ast;
        Ok(ret)
    }

    fn iter_subcrumbs(&self) -> Box<dyn Iterator<Item = Self::Crumb>> {
        const CHILDREN: [SectionRightCrumb; 2] = [SectionRightCrumb::Arg, SectionRightCrumb::Opr];
        Box::new(CHILDREN.iter().copied())
    }
}

impl Crumbable for crate::SectionSides<Ast> {
    type Crumb = SectionSidesCrumb;

    fn get(&self, _crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        Ok(&self.opr)
    }

    fn set(&self, _crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut ret = self.clone();
        ret.opr = new_ast;
        Ok(ret)
    }

    fn iter_subcrumbs(&self) -> Box<dyn Iterator<Item = Self::Crumb>> {
        const CHILDREN: [SectionSidesCrumb; 1] = [SectionSidesCrumb];
        Box::new(CHILDREN.iter().copied())
    }
}

impl Crumbable for crate::Prefix<Ast> {
    type Crumb = PrefixCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        let ret = match crumb {
            PrefixCrumb::Func => &self.func,
            PrefixCrumb::Arg  => &self.arg
        };
        Ok(ret)
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut ret = self.clone();
        let target  = match crumb {
            PrefixCrumb::Func => &mut ret.func,
            PrefixCrumb::Arg  => &mut ret.arg
        };
        *target = new_ast;
        Ok(ret)
    }

    fn iter_subcrumbs(&self) -> Box<dyn Iterator<Item = Self::Crumb>> {
        const CHILDREN: [PrefixCrumb; 2] = [PrefixCrumb::Func, PrefixCrumb::Arg];
        Box::new(CHILDREN.iter().copied())
    }
}

impl Crumbable for crate::Infix<Ast> {
    type Crumb = InfixCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        let ret = match crumb {
            InfixCrumb::LeftOperand  => &self.larg,
            InfixCrumb::Operator     => &self.opr ,
            InfixCrumb::RightOperand => &self.rarg,
        };
        Ok(ret)
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut ret = self.clone();
        let target  = match crumb {
            InfixCrumb::LeftOperand  => &mut ret.larg,
            InfixCrumb::Operator     => &mut ret.opr ,
            InfixCrumb::RightOperand => &mut ret.rarg,
        };
        *target = new_ast;
        Ok(ret)
    }

    fn iter_subcrumbs(&self) -> Box<dyn Iterator<Item = Self::Crumb>> {
        const CHILDREN: [InfixCrumb; 3] = [InfixCrumb::LeftOperand, InfixCrumb::Operator, InfixCrumb::RightOperand];
        Box::new(CHILDREN.iter().copied())
    }
}

impl Crumbable for crate::Module<Ast> {
    type Crumb = ModuleCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        let line = self.lines.get(crumb.line_index).ok_or(LineIndexOutOfBounds)?;
        line.elem.as_ref().ok_or(LineDoesNotContainAst::new(self,crumb).into())
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut module = self.clone();
        let line = module.lines.get_mut(crumb.line_index).ok_or(LineIndexOutOfBounds)?;
        line.elem.replace(new_ast);
        Ok(module)
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        let indices = non_empty_line_indices(self.lines.iter());
        let crumbs  = indices.map(|line_index| ModuleCrumb {line_index});
        Box::new(crumbs)
    }
}

impl Crumbable for crate::Import<Ast> {
    type Crumb = ImportCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        Ok(self.path.get(crumb.index).ok_or(LineIndexOutOfBounds)?)
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut module = self.clone();
        let line = module.lines.get_mut(crumb.line_index).ok_or(LineIndexOutOfBounds)?;
        line.elem.replace(new_ast);
        Ok(module)
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        let indices = non_empty_line_indices(self.lines.iter());
        let crumbs  = indices.map(|line_index| ModuleCrumb {line_index});
        Box::new(crumbs)
    }
}

impl Crumbable for crate::Block<Ast> {
    type Crumb = BlockCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        match crumb {
            BlockCrumb::HeadLine => Ok(&self.first_line.elem),
            BlockCrumb::TailLine {tail_index} => {
                let line = self.lines.get(*tail_index).ok_or(LineIndexOutOfBounds)?;
                line.elem.as_ref().ok_or(LineDoesNotContainAst::new(self,crumb).into())
            }
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut block = self.clone();
        match crumb {
            BlockCrumb::HeadLine              => block.first_line.elem = new_ast,
            BlockCrumb::TailLine {tail_index} => {
                let line = block.lines.get_mut(*tail_index).ok_or(LineIndexOutOfBounds)?;
                line.elem.replace(new_ast);
            }
        }
        Ok(block)
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        let first_line        = std::iter::once(BlockCrumb::HeadLine);
        let tail_line_indices = non_empty_line_indices(self.lines.iter());
        let tail_lines        = tail_line_indices.map(|tail_index| {
            BlockCrumb::TailLine {tail_index}
        });
        Box::new(first_line.chain(tail_lines))
    }
}

impl Crumbable for Shape<Ast> {
    type Crumb = Crumb;
    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        match (self,crumb) {
            (Shape::Block(shape), Crumb::Block(crumb))               => shape.get(crumb),
            (Shape::Module(shape),Crumb::Module(crumb))              => shape.get(crumb),
            (Shape::Infix(shape), Crumb::Infix(crumb))               => shape.get(crumb),
            (Shape::Prefix(shape), Crumb::Prefix(crumb))             => shape.get(crumb),
            (Shape::SectionLeft(shape), Crumb::SectionLeft(crumb))   => shape.get(crumb),
            (Shape::SectionRight(shape), Crumb::SectionRight(crumb)) => shape.get(crumb),
            (Shape::SectionSides(shape), Crumb::SectionSides(crumb)) => shape.get(crumb),
            (Shape::Import(shape), Crumb::Import(crumb))             => shape.get(crumb),
            (Shape::Mixfix(shape), Crumb::Mixfix(crumb))             => shape.get(crumb),
            (Shape::Group(shape), Crumb::Group(crumb))               => shape.get(crumb),
            (Shape::Def(shape), Crumb::Def(crumb))                   => shape.get(crumb),
            _                                           => Err(MismatchedCrumbType.into()),
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        match (self,crumb) {
            (Shape::Block(shape),  Crumb::Block(crumb))  => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Module(shape), Crumb::Module(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Infix(shape),  Crumb::Infix(crumb))  => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Prefix(shape), Crumb::Prefix(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Import(shape), Crumb::Import(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Mixfix(shape), Crumb::Mixfix(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Group(shape), Crumb::Group(crumb))   => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Def(shape), Crumb::Def(crumb))       => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::SectionLeft(shape), Crumb::SectionLeft(crumb)) => {
                Ok(shape.set(crumb,new_ast)?.into())
            },
            (Shape::SectionRight(shape), Crumb::SectionRight(crumb)) => {
                Ok(shape.set(crumb,new_ast)?.into())
            },
            (Shape::SectionSides(shape), Crumb::SectionSides(crumb)) => {
                Ok(shape.set(crumb,new_ast)?.into())
            },
            _                                            => Err(MismatchedCrumbType.into()),
        }
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        match self {
            Shape::Block(shape)       => Box::new(shape.iter_subcrumbs().map(Crumb::Block)),
            Shape::Module(shape)      => Box::new(shape.iter_subcrumbs().map(Crumb::Module)),
            Shape::Infix(shape)       => Box::new(shape.iter_subcrumbs().map(Crumb::Infix)),
            Shape::Prefix(shape)      => Box::new(shape.iter_subcrumbs().map(Crumb::Prefix)),
            Shape::SectionLeft(shape) => Box::new(shape.iter_subcrumbs().map(Crumb::SectionLeft)),
            Shape::Import(shape)      => Box::new(shape.iter_subcrumbs().map(Crumb::Import)),
            Shape::Mixfix(shape)      => Box::new(shape.iter_subcrumbs().map(Crumb::Mixfix)),
            Shape::Group(shape)       => Box::new(shape.iter_subcrumbs().map(Crumb::Group)),
            Shape::Def(shape)         => Box::new(shape.iter_subcrumbs().map(Crumb::Def)),
            Shape::SectionRight(shape) => {
                Box::new(shape.iter_subcrumbs().map(Crumb::SectionRight))
            },
            Shape::SectionSides(shape) => {
                Box::new(shape.iter_subcrumbs().map(Crumb::SectionSides))
            },
            _                    => Box::new(std::iter::empty()),
        }
    }
}

/// Just delegates the implementation to shape.
impl Crumbable for Ast {
    type Crumb = Crumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        self.shape().get(crumb)
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let new_shape = self.shape().set(crumb,new_ast)?;
        Ok(self.with_shape(new_shape))

    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        self.shape().iter_subcrumbs()
    }
}

/// Just delegates to Ast.
impl<T,E> Crumbable for known::KnownAst<T>
where for<'t> &'t Shape<Ast> : TryInto<&'t T, Error=E>,
      E                      : failure::Fail {
    type Crumb = Crumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> { self.ast().get(crumb) }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let new_ast = self.ast().set(crumb,new_ast)?;
        let ret = known::KnownAst::try_new(new_ast)?;
        Ok(ret)
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        self.ast().iter_subcrumbs()
    }
}



// ===========================
// === Recursive Traversal ===
// ===========================

/// Interface for recursive AST traversal using `Crumb` sequence.
///
/// Intended for `Ast` and `Ast`-like types, like `KnownAst`.
pub trait TraversableAst {
    /// Returns rewritten AST where child AST under location designated by `crumbs` is updated.
    ///
    /// Works recursively.
    fn set_traversing(&self, crumbs:&[Crumb], new_ast:Ast) -> FallibleResult<Self>
    where Self:Sized {
        let ast         = self.ast_ref();
        let updated_ast = if let Some(first_crumb) = crumbs.first() {
            let child = ast.get(first_crumb)?;
            let updated_child = child.set_traversing(&crumbs[1..], new_ast)?;
            ast.set(first_crumb,updated_child)
        } else {
            Ok(new_ast)
        };
        Self::from_ast(updated_ast?)
    }

    /// Recursively traverses AST to retrieve AST node located by given crumbs sequence.
    fn get_traversing<'a>(&'a self, crumbs:&[Crumb]) -> FallibleResult<&'a Ast> {
        let ast = self.ast_ref();
        if let Some(first_crumb) = crumbs.first() {
            let child = ast.get(first_crumb)?;
            child.get_traversing(&crumbs[1..])
        } else {
            Ok(ast)
        }
    }

    /// Access this node's AST.
    fn ast_ref(&self) -> &Ast;

    /// Wrap Ast into Self.
    fn from_ast(ast:Ast) -> FallibleResult<Self> where Self:Sized;
}

impl TraversableAst for Ast {
    fn ast_ref(&self) -> &Ast { self }

    fn from_ast(ast:Ast) -> FallibleResult<Self> { Ok(ast) }
}

impl<T,E> TraversableAst for known::KnownAst<T>
where for<'t> &'t Shape<Ast> : TryInto<&'t T, Error=E>,
      E                      : failure::Fail {
    fn ast_ref(&self) -> &Ast { self.ast() }

    fn from_ast(ast:Ast) -> FallibleResult<Self> { Ok(ast.try_into()?) }
}



// ===============
// === Utility ===
// ===============

/// Iterates over indices of non-empty lines in a line sequence.
pub fn non_empty_line_indices<'a, T:'a>
(iter:impl Iterator<Item = &'a crate::BlockLine<Option<T>>> + 'a)
 -> impl Iterator<Item=usize> + 'a {
    iter.enumerate().filter_map(|(line_index,line)| {
        line.elem.as_ref().map(|_| line_index)
    })
}



// ===============
// === Located ===
// ===============

/// Item which location is identified by `Crumbs`.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct Located<T> {
    /// Crumbs from containing parent.
    pub crumbs : Crumbs,
    /// The sub-item representation.
    #[shrinkwrap(main_field)]
    pub item   : T
}

impl<T> Located<T> {
    /// Creates a new located item.
    pub fn new(crumbs:Crumbs, item:T) -> Located<T> {
        Located {crumbs,item}
    }

    /// Creates a new item in a root location (empty crumbs list).
    pub fn new_root(item:T) -> Located<T> {
        let crumbs = default();
        Located {crumbs,item}
    }

    /// Creates a new item in a root location (single crumb location).
    pub fn new_direct_child(crumb:impl Into<Crumb>, item:T) -> Located<T> {
        let crumbs = vec![crumb.into()];
        Located {crumbs,item}
    }

    /// Uses given function to map over the item.
    pub fn map<U>(self, f:impl FnOnce(T) -> U) -> Located<U> {
        Located::new(self.crumbs, f(self.item))
    }

    /// Replaces the item, while pushing given crumbs on top of already present ones.
    pub fn into_descendant<U>(self, crumbs:Crumbs, item:U) -> Located<U> {
        let mut ret = self.map(|_| item);
        ret.crumbs.extend(crumbs);
        ret
    }

    /// Maps into child, concatenating this crumbs and child crumbs.
    pub fn push_descendant<U>(self, child:Located<U>) -> Located<U> {
        self.into_descendant(child.crumbs,child.item)
    }
}

/// Reference to AST stored under some known crumbs path.
pub type ChildAst<'a> = Located<&'a Ast>;



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use crate::HasRepr;

    use utils::test::ExpectTuple;

    #[test]
    fn infix_crumb() -> FallibleResult<()> {
        let infix = Ast::infix_var("foo","+","bar");
        let get   = |infix_crumb| {
            let crumb = Crumb::Infix(infix_crumb);
            infix.get(&crumb)
        };
        let set   = |infix_crumb, ast| {
            let crumb = Crumb::Infix(infix_crumb);
            infix.set(&crumb,ast)
        };
        let baz   = Ast::var("baz");
        let times = Ast::opr("*");


        assert_eq!(infix.repr(), "foo + bar");

        assert_eq!(get(InfixCrumb::LeftOperand)?.repr(),  "foo");
        assert_eq!(get(InfixCrumb::Operator)?.repr(),     "+");
        assert_eq!(get(InfixCrumb::RightOperand)?.repr(), "bar");

        assert_eq!(set(InfixCrumb::LeftOperand, baz.clone())?.repr(), "baz + bar");
        assert_eq!(set(InfixCrumb::Operator, times.clone())?.repr(), "foo * bar");
        assert_eq!(set(InfixCrumb::RightOperand, baz.clone())?.repr(), "foo + baz");

        Ok(())
    }

    #[test]
    fn prefix_crumb() -> FallibleResult<()> {
        let prefix = Ast::prefix(Ast::var("func"), Ast::var("arg"));
        let get   = |prefix_crumb| {
            let crumb = Crumb::Prefix(prefix_crumb);
            prefix.get(&crumb)
        };
        let set   = |prefix_crumb, ast| {
            let crumb = Crumb::Prefix(prefix_crumb);
            prefix.set(&crumb,ast)
        };
        let foo = Ast::var("foo");
        let x   = Ast::var("x");

        assert_eq!(prefix.repr(), "func arg");

        assert_eq!(get(PrefixCrumb::Func)?.repr(), "func");
        assert_eq!(get(PrefixCrumb::Arg)?.repr(),  "arg");

        assert_eq!(set(PrefixCrumb::Func, foo.clone())?.repr(), "foo arg");
        assert_eq!(set(PrefixCrumb::Arg,  x.clone())?.repr(), "func x");

        Ok(())
    }

    #[test]
    fn section_left_crumb() -> FallibleResult<()> {
        let app = Ast::section_left(Ast::var("foo"), Ast::var("bar"));
        let get   = |app_crumb| {
            let crumb = Crumb::SectionLeft(app_crumb);
            app.get(&crumb)
        };
        let set   = |app_crumb, ast| {
            let crumb = Crumb::SectionLeft(app_crumb);
            app.set(&crumb,ast)
        };
        let arg = Ast::var("arg");
        let opr = Ast::var("opr");

        assert_eq!(app.repr(), "foo bar");

        assert_eq!(get(SectionLeftCrumb::Arg)?.repr(), "foo");
        assert_eq!(get(SectionLeftCrumb::Opr)?.repr(), "bar");

        assert_eq!(set(SectionLeftCrumb::Arg, arg.clone())?.repr(), "arg bar");
        assert_eq!(set(SectionLeftCrumb::Opr, opr.clone())?.repr(), "foo opr");

        Ok(())
    }

    #[test]
    fn section_right_crumb() -> FallibleResult<()> {
        let app = Ast::section_right(Ast::var("foo"), Ast::var("bar"));
        let get   = |app_crumb| {
            let crumb = Crumb::SectionRight(app_crumb);
            app.get(&crumb)
        };
        let set   = |app_crumb, ast| {
            let crumb = Crumb::SectionRight(app_crumb);
            app.set(&crumb,ast)
        };
        let arg = Ast::var("arg");
        let opr = Ast::var("opr");

        assert_eq!(app.repr(), "foo bar");

        assert_eq!(get(SectionRightCrumb::Opr)?.repr(), "foo");
        assert_eq!(get(SectionRightCrumb::Arg)?.repr(), "bar");

        assert_eq!(set(SectionRightCrumb::Opr, opr.clone())?.repr(), "opr bar");
        assert_eq!(set(SectionRightCrumb::Arg, arg.clone())?.repr(), "foo arg");

        Ok(())
    }

    #[test]
    fn section_sides_crumb() -> FallibleResult<()> {
        let app = Ast::section_sides(Ast::var("foo"));
        let get   = |app_crumb| {
            let crumb = Crumb::SectionSides(app_crumb);
            app.get(&crumb)
        };
        let set   = |app_crumb, ast| {
            let crumb = Crumb::SectionSides(app_crumb);
            app.set(&crumb,ast)
        };
        let opr = Ast::var("opr");

        assert_eq!(app.repr(), "foo");

        assert_eq!(get(SectionSidesCrumb)?.repr(), "foo");
        assert_eq!(set(SectionSidesCrumb, opr.clone())?.repr(), "opr");

        Ok(())
    }

    #[test]
    fn nested_infix() -> FallibleResult<()> {
        use InfixCrumb::*;

        let sum   = Ast::infix_var("foo", "+", "bar");
        let infix = Ast::infix(Ast::var("main"), "=", sum);
        assert_eq!(infix.repr(), "main = foo + bar");

        let set = |crumbs: &[InfixCrumb], ast| {
            let crumbs = crumbs.iter().map(|c| Crumb::Infix(*c)).collect_vec();
            infix.set_traversing(&crumbs, ast)
        };
        let get = |crumbs: &[InfixCrumb]| {
            let crumbs = crumbs.iter().map(|c| Crumb::Infix(*c)).collect_vec();
            infix.get_traversing(&crumbs)
        };

        assert_eq!(set(&[RightOperand,LeftOperand], Ast::var("baz"))?.repr(), "main = baz + bar");
        assert_eq!(set(&[LeftOperand], Ast::var("baz"))?.repr(), "baz = foo + bar");


        assert_eq!(get(&[Operator])?.repr(), "=");
        assert_eq!(get(&[RightOperand])?.repr(), "foo + bar");
        assert_eq!(get(&[RightOperand,LeftOperand])?.repr(), "foo");
        assert_eq!(get(&[RightOperand,RightOperand])?.repr(), "bar");
        Ok(())
    }


    #[test]
    fn iterate_infix() {
        let sum = crate::Infix::from_vars("foo", "+", "bar");
        let (larg,opr,rarg) = sum.iter_subcrumbs().expect_tuple();
        assert_eq!(larg, InfixCrumb::LeftOperand);
        assert_eq!(opr,  InfixCrumb::Operator);
        assert_eq!(rarg, InfixCrumb::RightOperand);
    }

    #[test]
    fn iterate_module() {
        let var = crate::Ast::var("foo");
        let lines = [
            Some(var.clone_ref()),
            None,
            Some(var.clone_ref()),
        ];
        let module = crate::Module::from_lines(&lines);
        assert_eq!(module.repr(), "foo\n\nfoo");

        let (line0,line2) = module.iter_subcrumbs().expect_tuple();
        assert_eq!(line0.line_index,0);
        assert_eq!(line2.line_index,2);
    }

    #[test]
    fn iterate_block() {
        let first_line = crate::Ast::var("foo");
        let lines      = [
            Some(crate::Ast::var("bar")),
            None,
            Some(crate::Ast::var("baz")),
        ];
        let block               = crate::Block::from_lines(&first_line,&lines);
        let (line0,line1,line3) = block.iter_subcrumbs().expect_tuple();
        assert_eq!(line0, BlockCrumb::HeadLine);
        assert_eq!(line1, BlockCrumb::TailLine {tail_index:0});
        assert_eq!(line3, BlockCrumb::TailLine {tail_index:2});
    }

    #[test]
    fn mismatched_crumb() {
        let sum        = Ast::infix_var("foo", "+", "bar");
        let crumb      = Crumb::Module(ModuleCrumb {line_index:0});
        let first_line = sum.get(&crumb);
        first_line.expect_err("Using module crumb on infix should fail");
    }
}

