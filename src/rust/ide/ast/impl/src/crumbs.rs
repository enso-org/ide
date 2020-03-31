//! Crumbs for AST. Crumb identifies children node location in AST node. The access should be
//! possible in a constant time.

use crate::prelude::*;

use crate::{known, SegmentFmt, TextLine};
use crate::Shape;
use utils::fail::FallibleResult;



// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[fail(display = "The crumb refers to a {} which is not present.", _0)]
#[derive(Debug,Fail,Clone)]
pub struct NotPresent(String);

#[allow(missing_docs)]
#[fail(display = "The crumb refers to {} by index that is out of bounds.", _0)]
#[derive(Debug,Fail,Clone)]
pub struct IndexOutOfBounds(String);

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
    TextLineFmt(TextLineFmtCrumb),
    TextBlockFmt(TextBlockFmtCrumb),
    TextUnclosed(TextUnclosedCrumb),
    Prefix(PrefixCrumb),
    Infix(InfixCrumb),
    SectionLeft(SectionLeftCrumb),
    SectionRight(SectionRightCrumb),
    SectionSides(SectionSidesCrumb),
    Module(ModuleCrumb),
    Block(BlockCrumb),
    Import(ImportCrumb),
    Mixfix(MixfixCrumb),
    Group(GroupCrumb),
    Def(DefCrumb),
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
pub struct ImportCrumb {pub index:usize}


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


// === TextLineFmt ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub struct TextLineFmtCrumb {pub segment_index:usize}


// === TextBlockFmt ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub struct TextBlockFmtCrumb {
    pub text_line_index : usize,
    pub segment_index   : usize
}


// === TextUnclosed ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,PartialEq,Hash)]
pub struct TextUnclosedCrumb {
    pub text_line_crumb : TextLineFmtCrumb
}


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
from_crumb!{Import,ImportCrumb}
from_crumb!{Mixfix,MixfixCrumb}
from_crumb!{Group,GroupCrumb}
from_crumb!{Def,DefCrumb}
from_crumb!{TextLineFmt,TextLineFmtCrumb}
from_crumb!{TextBlockFmt,TextBlockFmtCrumb}
from_crumb!{TextUnclosed,TextUnclosedCrumb}


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
        const CHILDREN: [SectionRightCrumb; 2] = [SectionRightCrumb::Opr, SectionRightCrumb::Arg];
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
        let line = self.lines.get(crumb.line_index).ok_or(IndexOutOfBounds("line".into()))?;
        line.elem.as_ref().ok_or_else(|| LineDoesNotContainAst::new(self,crumb).into())
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut module = self.clone();
        let line = module.lines.get_mut(crumb.line_index).ok_or(IndexOutOfBounds("line".into()))?;
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
        self.path.get(crumb.index).ok_or_else(|| IndexOutOfBounds("path".into()).into())
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut import = self.clone();
        let path = import.path.get_mut(crumb.index).ok_or(IndexOutOfBounds("path".into()))?;
        *path = new_ast;
        Ok(import)
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        let indices = self.path.iter().enumerate().map(|(indices,_)| indices);
        let crumbs  = indices.map(|path_index| ImportCrumb { index: path_index });
        Box::new(crumbs)
    }
}

impl Crumbable for crate::Mixfix<Ast> {
    type Crumb = MixfixCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        match crumb {
            MixfixCrumb::Name {index} => {
                self.name.get(*index).ok_or_else(|| IndexOutOfBounds("name".into()).into())
            },
            MixfixCrumb::Args {index} => {
                self.args.get(*index).ok_or_else(|| IndexOutOfBounds("arg".into()).into())
            }
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut mixfix = self.clone();
        match crumb {
            MixfixCrumb::Name {index} => {
                *mixfix.name.get_mut(*index).ok_or_else(|| {
                    IndexOutOfBounds("name".into())
                })? = new_ast;
            },
            MixfixCrumb::Args {index} => {
                *mixfix.args.get_mut(*index).ok_or(IndexOutOfBounds("arg".into()))? = new_ast;
            }
        }
        Ok(mixfix)
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        let name_iter = self.name.iter().enumerate().map(|(index,_)|{
            MixfixCrumb::Name{index}
        });
        let args_iter = self.args.iter().enumerate().map(|(index,_)| MixfixCrumb::Args{index});
        Box::new(name_iter.chain(args_iter))
    }
}

impl Crumbable for crate::Def<Ast> {
    type Crumb = DefCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        match crumb {
            DefCrumb::Name         => Ok(&self.name),
            DefCrumb::Args {index} => self.args.get(*index).ok_or_else(|| {
                IndexOutOfBounds("arg".into()).into()
            }),
            DefCrumb::Body         => self.body.as_ref().ok_or_else(|| {
                NotPresent("body".into()).into()
            })
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut def = self.clone();
        match crumb {
            DefCrumb::Name         => def.name = new_ast,
            DefCrumb::Args {index} => {
                let arg = def.args.get_mut(*index).ok_or(IndexOutOfBounds("arg".into()))?;
                *arg = new_ast;
            },
            DefCrumb::Body         => def.body = Some(new_ast)
        }
        Ok(def)
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        let name_iter = std::iter::once(DefCrumb::Name);
        let args_iter = self.args.iter().enumerate().map(|(index,_)| DefCrumb::Args{index});
        let body_iter = self.body.iter().map(|_| DefCrumb::Body);
        Box::new(name_iter.chain(args_iter).chain(body_iter))
    }
}

impl Crumbable for crate::Group<Ast> {
    type Crumb = GroupCrumb;

    fn get(&self, _crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        Ok(self.body.as_ref().ok_or(NotPresent("body".into()))?)
    }

    fn set(&self, _crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut group = self.clone();
        group.body = Some(new_ast);
        Ok(group)
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        Box::new(self.body.iter().map(|_| GroupCrumb))
    }
}

impl Crumbable for crate::Block<Ast> {
    type Crumb = BlockCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        match crumb {
            BlockCrumb::HeadLine => Ok(&self.first_line.elem),
            BlockCrumb::TailLine {tail_index} => {
                let line = self.lines.get(*tail_index).ok_or(IndexOutOfBounds("line".into()))?;
                line.elem.as_ref().ok_or_else(|| LineDoesNotContainAst::new(self,crumb).into())
            }
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut block = self.clone();
        match crumb {
            BlockCrumb::HeadLine              => block.first_line.elem = new_ast,
            BlockCrumb::TailLine {tail_index} => {
                let line = block.lines.get_mut(*tail_index).ok_or(IndexOutOfBounds("line".into()))?;
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

impl Crumbable for crate::TextLineFmt<Ast> {
    type Crumb = TextLineFmtCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        let segment = self.text.get(crumb.segment_index);
        let segment = segment.ok_or(IndexOutOfBounds("text segment".into()))?;
        if let crate::SegmentFmt::SegmentExpr(expr) = segment {
            expr.value.as_ref().map(|ast| ast).ok_or_else(|| NotPresent("expression".into()).into())
        } else {
            Err(NotPresent("segment expression".into()).into())
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut text = self.clone();
        let segment = text.text.get_mut(crumb.segment_index);
        let segment = segment.ok_or(IndexOutOfBounds("text segment".into()))?;
        if let crate::SegmentFmt::SegmentExpr(expr) = segment {
            expr.value = Some(new_ast);
            Ok(text)
        } else {
            Err(NotPresent("segment expression".into()).into())
        }
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        Box::new(self.text.iter().enumerate().filter_map(|(segment_index,segment)| {
            if let crate::SegmentFmt::SegmentExpr(_) = segment {
                Some(TextLineFmtCrumb{segment_index})
            } else {
                None
            }
        }))
    }
}

impl Crumbable for crate::TextUnclosed<Ast> {
    type Crumb = TextUnclosedCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        if let crate::TextLine::TextLineFmt(text_line) = &self.line {
            text_line.get(&crumb.text_line_crumb)
        } else {
            Err(NotPresent("formatted text line".into()).into())
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut text = self.clone();
        if let crate::TextLine::TextLineFmt(text_line) = text.line {
            text.line = crate::TextLine::TextLineFmt(text_line.set(&crumb.text_line_crumb,new_ast)?);
            Ok(text)
        } else {
            Err(NotPresent("formatted text line".into()).into())
        }
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        if let TextLine::TextLineFmt(text_line) = &self.line {
            Box::new(text_line.iter_subcrumbs().map(|text_line_crumb| {
                TextUnclosedCrumb{text_line_crumb}
            }))
        } else {
            Box::new(std::iter::empty())
        }
    }
}

impl Crumbable for crate::TextBlockFmt<Ast> {
    type Crumb = TextBlockFmtCrumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        let line = self.text.get(crumb.text_line_index).ok_or(IndexOutOfBounds("line".into()))?;
        let segment = line.text.get(crumb.segment_index).ok_or(IndexOutOfBounds("segment".into()))?;
        if let crate::SegmentFmt::SegmentExpr(expr) = segment {
            expr.value.as_ref().map(|ast| ast).ok_or_else(|| {
                NotPresent("expression value".into()).into()
            })
        } else {
            Err(NotPresent("expression segment".into()).into())
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut text = self.clone();
        let line    = text.text.get_mut(crumb.text_line_index);
        let line    = line.ok_or(IndexOutOfBounds("line".into()))?;
        let segment = line.text.get_mut(crumb.segment_index);
        let segment = segment.ok_or(IndexOutOfBounds("segment".into()))?;
        if let SegmentFmt::SegmentExpr(expr) = segment {
            expr.value = Some(new_ast);
            Ok(text)
        } else {
            Err(NotPresent("expression segment".into()).into())
        }
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        Box::new(self.text.iter().enumerate().flat_map(|(text_line_index,line)| {
            line.text.iter().enumerate().filter(|(_,segment)| {
                if let SegmentFmt::SegmentExpr(_) = segment { true } else { false }
            }).map(move |(segment_index,_)| {
                TextBlockFmtCrumb{text_line_index,segment_index}
            })
        }))
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
            (Shape::TextLineFmt(shape), Crumb::TextLineFmt(crumb))   => shape.get(crumb),
            (Shape::TextBlockFmt(shape), Crumb::TextBlockFmt(crumb)) => shape.get(crumb),
            (Shape::TextUnclosed(shape), Crumb::TextUnclosed(crumb)) => shape.get(crumb),
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
            (Shape::Group(shape),  Crumb::Group(crumb))  => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::Def(shape),    Crumb::Def(crumb))    => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::TextLineFmt(shape), Crumb::TextLineFmt(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::TextBlockFmt(shape), Crumb::TextBlockFmt(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::TextUnclosed(shape), Crumb::TextUnclosed(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::SectionLeft(shape), Crumb::SectionLeft(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::SectionRight(shape), Crumb::SectionRight(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            (Shape::SectionSides(shape), Crumb::SectionSides(crumb)) => Ok(shape.set(crumb,new_ast)?.into()),
            _ => Err(MismatchedCrumbType.into()),
        }
    }

    fn iter_subcrumbs<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Crumb> + 'a> {
        match self {
            Shape::TextLineFmt(shape)  => Box::new(shape.iter_subcrumbs().map(Crumb::TextLineFmt)),
            Shape::TextBlockFmt(shape) => Box::new(shape.iter_subcrumbs().map(Crumb::TextBlockFmt)),
            Shape::TextUnclosed(shape) => Box::new(shape.iter_subcrumbs().map(Crumb::TextUnclosed)),
            Shape::Block(shape)        => Box::new(shape.iter_subcrumbs().map(Crumb::Block)),
            Shape::Module(shape)       => Box::new(shape.iter_subcrumbs().map(Crumb::Module)),
            Shape::Infix(shape)        => Box::new(shape.iter_subcrumbs().map(Crumb::Infix)),
            Shape::Prefix(shape)       => Box::new(shape.iter_subcrumbs().map(Crumb::Prefix)),
            Shape::SectionLeft(shape)  => Box::new(shape.iter_subcrumbs().map(Crumb::SectionLeft)),
            Shape::Import(shape)       => Box::new(shape.iter_subcrumbs().map(Crumb::Import)),
            Shape::Mixfix(shape)       => Box::new(shape.iter_subcrumbs().map(Crumb::Mixfix)),
            Shape::Group(shape)        => Box::new(shape.iter_subcrumbs().map(Crumb::Group)),
            Shape::Def(shape)          => Box::new(shape.iter_subcrumbs().map(Crumb::Def)),
            Shape::SectionRight(shape) => Box::new(shape.iter_subcrumbs().map(Crumb::SectionRight)),
            Shape::SectionSides(shape) => Box::new(shape.iter_subcrumbs().map(Crumb::SectionSides)),
            _ => Box::new(std::iter::empty()),
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

    use crate::{HasRepr, SegmentExpr, SegmentFmt, TextBlockLine, TextLine, TextLineFmt};

    use utils::test::ExpectTuple;

    fn get<T,F:FnOnce(T) -> Crumb>(f:F, ast:&Ast, crumb:T) -> FallibleResult<&Ast> {
        let crumb = f(crumb);
        ast.get(&crumb)
    }

    fn set<T,F:FnOnce(T) -> Crumb>(f:F, ast:&Ast, crumb:T,internal_ast:Ast) -> FallibleResult<Ast> {
        let crumb = f(crumb);
        ast.set(&crumb, internal_ast)
    }

    // === Infix ===

    #[test]
    fn infix_crumb() -> FallibleResult<()> {
        let infix  = Ast::infix_var("foo","+","bar");
        let crumbf = |crumb| Crumb::Infix(crumb);
        let baz    = Ast::var("baz");
        let times  = Ast::opr("*");

        assert_eq!(infix.repr(), "foo + bar");

        assert_eq!(get(crumbf,&infix,InfixCrumb::LeftOperand)?.repr(),  "foo");
        assert_eq!(get(crumbf,&infix,InfixCrumb::Operator)?.repr(),     "+");
        assert_eq!(get(crumbf,&infix,InfixCrumb::RightOperand)?.repr(), "bar");

        assert_eq!(set(crumbf,&infix,InfixCrumb::LeftOperand, baz.clone())?.repr(), "baz + bar");
        assert_eq!(set(crumbf,&infix,InfixCrumb::Operator, times.clone())?.repr(), "foo * bar");
        assert_eq!(set(crumbf,&infix,InfixCrumb::RightOperand, baz.clone())?.repr(), "foo + baz");

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



    // ===========
    // == Text ===
    // ===========


    // === TextLineFmt ===

    #[test]
    fn text_line_fmt_crumb() {
        let expr   = SegmentExpr { value : Some(Ast::var("foo")) };
        let text   = vec![SegmentFmt::SegmentExpr(expr)];
        let ast    = Ast::text_line_fmt(text);
        let crumbf = |crumb| Crumb::TextLineFmt(crumb);
        let bar    = Ast::var("bar");
        let crumb  = TextLineFmtCrumb{segment_index:0};

        assert_eq!(ast.repr(), "'`foo`'");
        assert_eq!(get(crumbf,&ast,crumb).unwrap().repr(),  "foo");
        assert_eq!(set(crumbf,&ast,crumb,bar).unwrap().repr(), "'`bar`'");
    }

    #[test]
    fn iterate_text_line_fmt() {
        let expr1  = SegmentExpr { value : Some(Ast::var("foo")) };
        let expr2  = SegmentExpr { value : Some(Ast::var("bar")) };
        let text   = vec![SegmentFmt::SegmentExpr(expr1),SegmentFmt::SegmentExpr(expr2)];
        let ast    = Ast::text_line_fmt(text);

        let (segment1,segment2) = ast.iter_subcrumbs().expect_tuple();

        assert_eq!(segment1, Crumb::TextLineFmt(TextLineFmtCrumb{segment_index:0}));
        assert_eq!(segment2, Crumb::TextLineFmt(TextLineFmtCrumb{segment_index:1}));
    }

    // === TextBlockFmt ===

    #[test]
    fn text_block_fmt_crumb() {
        let empty_lines = default();
        let expr        = SegmentExpr { value : Some(Ast::var("foo")) };
        let text        = vec![SegmentFmt::SegmentExpr(expr)];
        let line1       = TextBlockLine{empty_lines,text};

        let empty_lines = default();
        let expr        = SegmentExpr { value : Some(Ast::var("bar")) };
        let text        = vec![SegmentFmt::SegmentExpr(expr)];
        let line2       = TextBlockLine{empty_lines,text};

        let lines       = vec![line1,line2];
        let ast         = Ast::text_block_fmt(lines);
        let qux         = Ast::var("qux");
        let baz         = Ast::var("baz");

        let crumbf = |crumb| Crumb::TextBlockFmt(crumb);
        assert_eq!(ast.repr(), "'''\n`foo`\n`bar`");

        let crumb1 = TextBlockFmtCrumb {text_line_index:0, segment_index:0};
        let crumb2 = TextBlockFmtCrumb {text_line_index:1, segment_index:0};

        assert_eq!(get(crumbf,&ast,crumb1).unwrap().repr(), "foo");
        assert_eq!(get(crumbf,&ast,crumb2).unwrap().repr(), "bar");

        assert_eq!(set(crumbf,&ast,crumb1,qux).unwrap().repr(),"'''\n`qux`\n`bar`");
        assert_eq!(set(crumbf,&ast,crumb2,baz).unwrap().repr(),"'''\n`foo`\n`baz`");
    }

    #[test]
    fn iterate_text_block_fmt() {
        let empty_lines = default();
        let expr        = SegmentExpr { value : Some(Ast::var("foo")) };
        let text        = vec![SegmentFmt::SegmentExpr(expr)];
        let line1       = TextBlockLine{empty_lines,text};

        let empty_lines = default();
        let expr        = SegmentExpr { value : Some(Ast::var("bar")) };
        let text        = vec![SegmentFmt::SegmentExpr(expr)];
        let line2       = TextBlockLine{empty_lines,text};

        let lines       = vec![line1,line2];
        let ast         = Ast::text_block_fmt(lines);

        let crumb1 = TextBlockFmtCrumb {text_line_index:0, segment_index:0};
        let crumb2 = TextBlockFmtCrumb {text_line_index:1, segment_index:0};

        let (line1,line2) = ast.iter_subcrumbs().expect_tuple();

        assert_eq!(line1, Crumb::TextBlockFmt(crumb1));
        assert_eq!(line2, Crumb::TextBlockFmt(crumb2));
    }


    // == TextUnclosed ===

    #[test]
    fn text_unclosed_crumb() {
        let expr            = SegmentExpr { value : Some(Ast::var("foo")) };
        let text            = vec![SegmentFmt::SegmentExpr(expr)];
        let text_line       = TextLineFmt{text};
        let line            = TextLine::TextLineFmt(text_line);
        let ast             = Ast::text_unclosed(line);
        let crumbf          = |crumb| Crumb::TextUnclosed(crumb);
        let bar             = Ast::var("bar");
        let text_line_crumb = TextLineFmtCrumb{segment_index:0};
        let crumb           = TextUnclosedCrumb{text_line_crumb};

        assert_eq!(ast.repr(), "'`foo`");
        assert_eq!(get(crumbf,&ast,crumb).unwrap().repr(),  "foo");
        assert_eq!(set(crumbf,&ast,crumb,bar).unwrap().repr(), "'`bar`");
    }

    #[test]
    fn iterate_text_unclosed() {
        let expr1           = SegmentExpr { value : Some(Ast::var("foo")) };
        let expr2           = SegmentExpr { value : Some(Ast::var("bar")) };
        let text            = vec![SegmentFmt::SegmentExpr(expr1),SegmentFmt::SegmentExpr(expr2)];
        let text_line       = TextLineFmt{text};
        let line            = TextLine::TextLineFmt(text_line);
        let ast             = Ast::text_unclosed(line);
        let text_line_crumb = TextLineFmtCrumb{segment_index:0};
        let crumb1          = TextUnclosedCrumb{text_line_crumb};
        let text_line_crumb = TextLineFmtCrumb{segment_index:1};
        let crumb2          = TextUnclosedCrumb{text_line_crumb};

        let (segment1,segment2) = ast.iter_subcrumbs().expect_tuple();
        assert_eq!(segment1,Crumb::TextUnclosed(crumb1));
        assert_eq!(segment2,Crumb::TextUnclosed(crumb2));
    }


    // === Prefix ===

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
    fn iterate_prefix() -> FallibleResult<()> {
        let prefix = Ast::prefix(Ast::var("func"), Ast::var("arg"));

        let (func,arg) = prefix.iter_subcrumbs().expect_tuple();

        assert_eq!(func, Crumb::Prefix(PrefixCrumb::Func));
        assert_eq!(arg, Crumb::Prefix(PrefixCrumb::Arg));

        Ok(())
    }


    // === SectionLeft ===

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
    fn iterate_section_left() -> FallibleResult<()> {
        let app = Ast::section_left(Ast::var("foo"), Ast::var("bar"));

        let (arg,opr) = app.iter_subcrumbs().expect_tuple();
        assert_eq!(arg, Crumb::SectionLeft(SectionLeftCrumb::Arg));
        assert_eq!(opr, Crumb::SectionLeft(SectionLeftCrumb::Opr));

        Ok(())
    }


    // === SectionRight ===

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
    fn iterate_section_right() -> FallibleResult<()> {
        let app = Ast::section_right(Ast::var("foo"), Ast::var("bar"));

        let (opr,arg) = app.iter_subcrumbs().expect_tuple();
        assert_eq!(arg, Crumb::SectionRight(SectionRightCrumb::Arg));
        assert_eq!(opr, Crumb::SectionRight(SectionRightCrumb::Opr));

        Ok(())
    }


    // === SectionSides ===

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
    fn iterate_section_sides() -> FallibleResult<()> {
        let app = Ast::section_sides(Ast::var("foo"));

        let mut iter = app.iter_subcrumbs();

        assert_eq!(iter.next(), Some(Crumb::SectionSides(SectionSidesCrumb)));
        assert_eq!(iter.next(), None);

        Ok(())
    }


    // === Module ===

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


    // === Block ===

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

