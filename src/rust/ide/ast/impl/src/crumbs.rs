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


// === Infix ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub enum InfixCrumb {
    LeftOperand,
    Operator,
    RightOperand,
}

// === Conversion Traits ===

impl From<BlockCrumb> for Crumb {
    fn from(crumb: BlockCrumb) -> Self {
        Crumb::Block(crumb)
    }
}

impl From<&BlockCrumb> for Crumb {
    fn from(crumb: &BlockCrumb) -> Self {
        Crumb::Block(crumb.clone())
    }
}

impl From<ModuleCrumb> for Crumb {
    fn from(crumb: ModuleCrumb) -> Self {
        Crumb::Module(crumb)
    }
}
impl From<&ModuleCrumb> for Crumb {
    fn from(crumb: &ModuleCrumb) -> Self {
        Crumb::Module(crumb.clone())
    }
}

impl From<InfixCrumb> for Crumb {
    fn from(crumb: InfixCrumb) -> Self {
        Crumb::Infix(crumb)
    }
}

impl From<&InfixCrumb> for Crumb {
    fn from(crumb: &InfixCrumb) -> Self {
        Crumb::Infix(crumb.clone())
    }
}



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
            (Shape::Block(shape), Crumb::Block(crumb))  => shape.get(crumb),
            (Shape::Module(shape),Crumb::Module(crumb)) => shape.get(crumb),
            (Shape::Infix(shape), Crumb::Infix(crumb))  => shape.get(crumb),
            _                                           => Err(MismatchedCrumbType.into()),
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        match (self,crumb) {
            (Shape::Block(shape),  Crumb::Block(crumb))  => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Module(shape), Crumb::Module(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Infix(shape),  Crumb::Infix(crumb))  => Ok(shape.set(crumb,new_ast)?.into()),
            _                                            => Err(MismatchedCrumbType.into()),
        }
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        match self {
            Shape::Block(shape)  => Box::new(shape.iter_subcrumbs().map(Crumb::Block)),
            Shape::Module(shape) => Box::new(shape.iter_subcrumbs().map(Crumb::Module)),
            Shape::Infix(shape)  => Box::new(shape.iter_subcrumbs().map(Crumb::Infix)),
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

