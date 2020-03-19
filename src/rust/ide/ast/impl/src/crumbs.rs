use crate::prelude::*;

use crate::Ast;
use crate::known;
use crate::Shape;
use utils::fail::FallibleResult;
use crate::Shape::Mod;

pub type Crumbs = Vec<Crumb>;

pub enum Crumb {
    Block(BlockCrumb),
    Module(ModuleCrumb),
    Infix(InfixCrumb),
}

pub enum BlockCrumb {
    HeadLine,
    TailLine {tail_index:usize},
}

pub struct ModuleCrumb {pub line_index:usize}

#[derive(Clone,Copy,Debug)]
pub enum InfixCrumb {
    LeftOperand,
    Operator,
    RightOperand,
}

#[derive(Debug,Display,Fail,Clone,Copy)]
struct NotYetImplemented;

#[derive(Debug,Display,Fail,Clone,Copy)]
struct LineIndexOutOfBounds;

#[derive(Debug,Display,Fail,Clone,Copy)]
struct LineDoesNotContainAst;

#[derive(Debug,Display,Fail,Clone,Copy)]
struct MismatchedCrumbType;

pub trait Crumbable {
    type Crumb;

    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast>;
    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> where Self:Sized;

    fn children_crumbs(&self) -> Vec<Self::Crumb>;

    fn enumerate(&self) -> Vec<(Self::Crumb,&Ast)> {
        self.children_crumbs().into_iter().map(|crumb| {
            // NOTE safe if this module is correct - children crumbs are always accessible.
            let child = self.get(&crumb).unwrap();
            (crumb,child)
        }).collect()
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

    fn children_crumbs(&self) -> Vec<Self::Crumb> {
        vec![InfixCrumb::LeftOperand, InfixCrumb::Operator, InfixCrumb::RightOperand]
    }
}

impl Crumbable for crate::Module<Ast> {
    type Crumb = ModuleCrumb;
    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        let line = self.lines.get(crumb.line_index).ok_or(LineIndexOutOfBounds)?;
        line.elem.as_ref().ok_or(LineDoesNotContainAst.into())
    }
    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        let mut module = self.clone();
        let line = module.lines.get_mut(crumb.line_index).ok_or(LineIndexOutOfBounds)?;
        line.elem.replace(new_ast);
        Ok(module)
    }

    fn children_crumbs(&self) -> Vec<Self::Crumb> {
        let lines = self.lines.iter().enumerate();
        lines.flat_map(|(line_index,line)| {
            line.elem.as_ref().map(|_| ModuleCrumb {line_index})
        }).collect()
    }
}

impl Crumbable for crate::Block<Ast> {
    type Crumb = BlockCrumb;
    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        match crumb {
            BlockCrumb::HeadLine => Ok(&self.first_line.elem),
            BlockCrumb::TailLine {tail_index} => {
                let line = self.lines.get(*tail_index).ok_or(LineIndexOutOfBounds)?;
                line.elem.as_ref().ok_or(LineDoesNotContainAst.into())
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

    fn children_crumbs(&self) -> Vec<Self::Crumb> {
        let tail_lines = self.lines.iter().enumerate();
        let tail_line_crumbs = tail_lines.flat_map(|(tail_index,line)| {
            line.elem.as_ref().map(|_| BlockCrumb::TailLine {tail_index})
        });

        let mut ret = Vec::new();
        ret.push(BlockCrumb::HeadLine);
        ret.extend(tail_line_crumbs);
        ret
    }
}

impl Crumbable for Ast {
    type Crumb = Crumb;
    fn get(&self, crumb:&Self::Crumb) -> FallibleResult<&Ast> {
        match (self.shape(),crumb) {
            (Shape::Block(shape), Crumb::Block(crumb))  => shape.get(crumb),
            (Shape::Module(shape),Crumb::Module(crumb)) => shape.get(crumb),
            (Shape::Infix(shape), Crumb::Infix(crumb))  => shape.get(crumb),
            _                                           => Err(MismatchedCrumbType.into()),
        }
    }

    fn set(&self, crumb:&Self::Crumb, new_ast:Ast) -> FallibleResult<Self> {
        match (self.shape(),crumb) {
            (Shape::Block(shape),  Crumb::Block(crumb))  => Ok(self.with_shape(shape.set(crumb,new_ast)?)),
            (Shape::Module(shape), Crumb::Module(crumb)) => Ok(self.with_shape(shape.set(crumb,new_ast)?)),
            (Shape::Infix(shape),  Crumb::Infix(crumb))  => Ok(self.with_shape(shape.set(crumb,new_ast)?)),
            _                                            => Err(MismatchedCrumbType.into()),
        }
    }

    fn children_crumbs(&self) -> Vec<Self::Crumb> {
        match self.shape() {
            Shape::Block(shape)  => shape.children_crumbs().into_iter().map(Crumb::Block).collect(),
            Shape::Module(shape) => shape.children_crumbs().into_iter().map(Crumb::Module).collect(),
            Shape::Infix(shape)  => shape.children_crumbs().into_iter().map(Crumb::Infix).collect(),
            _                    => vec![],
        }
    }
}

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
    fn children_crumbs(&self) -> Vec<Self::Crumb> { self.ast().children_crumbs() }
}

fn set_traversing(ast:&Ast, crumbs:&[Crumb], new_ast:Ast) -> FallibleResult<Ast> {
    if let Some(first_crumb) = crumbs.first() {
        let child = ast.get(first_crumb)?;
        let updated_child = set_traversing(child, &crumbs[1..], new_ast)?;
        ast.set(first_crumb,updated_child)
    } else {
        Ok(new_ast)
    }
}

fn get_traversing<'a>(ast:&'a Ast, crumbs:&[Crumb]) -> FallibleResult<&'a Ast> {
    if let Some(first_crumb) = crumbs.first() {
        let child = ast.get(first_crumb)?;
        get_traversing(child, &crumbs[1..])
    } else {
        Ok(ast)
    }
}

mod tests {
    use super::*;
    use crate::HasRepr;

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

        let sum = Ast::infix_var("foo", "+", "bar");
        let infix = Ast::infix(Ast::var("main"), "=", sum);
        assert_eq!(infix.repr(), "main = foo + bar");

        let set = |crumbs: &[InfixCrumb], ast| {
            let crumbs = crumbs.iter().map(|c| Crumb::Infix(*c)).collect_vec();
            set_traversing(&infix, &crumbs, ast)
        };
        let get = |crumbs: &[InfixCrumb]| {
            let crumbs = crumbs.iter().map(|c| Crumb::Infix(*c)).collect_vec();
            get_traversing(&infix, &crumbs)
        };

        assert_eq!(set(&[RightOperand,LeftOperand], Ast::var("baz"))?.repr(), "main = baz + bar");
        assert_eq!(set(&[LeftOperand], Ast::var("baz"))?.repr(), "baz = foo + bar");


        assert_eq!(get(&[Operator])?.repr(), "=");
        assert_eq!(get(&[RightOperand])?.repr(), "foo + bar");
        assert_eq!(get(&[RightOperand,LeftOperand])?.repr(), "foo");
        assert_eq!(get(&[RightOperand,RightOperand])?.repr(), "bar");
        Ok(())
    }
}

