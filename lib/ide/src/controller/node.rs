//! Submodules of the node controller.

pub mod mock;

use crate::prelude::*;

use ast::Ast;
use ast::ID;
use nalgebra::Vector2;



// ================
// === Position ===
// ================

/// Used e.g. for node position
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Position {
    /// Vector storing coordinates of the visual position.
    pub vector:Vector2<f32>
}

impl Position {
    pub fn new(x:f32, y:f32) -> Position {
        let vector = Vector2::new(x,y);
        Position {vector}
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::new(default(),default())
    }
}



// =================
// === Interface ===
// =================

pub trait Interface {
    fn id(&self) -> ID;

    fn expression(&self) -> FallibleResult<Ast>;

    fn position(&self) -> FallibleResult<Position>;

    fn set_expression(&self, ast:Ast) -> FallibleResult<()>;

    fn set_expression_text(&self, expression:&str) -> FallibleResult<()>;

    fn set_position(&self, new_position:Position) -> FallibleResult<()>;
}



// =============
// === Trait ===
// =============

struct Controller {
    graph : controller::graph::Handle,
    id    : ID,
}

impl Interface for Controller {
    fn id(&self) -> ID {
        self.id
    }

    fn expression(&self) -> FallibleResult<Ast> {
        let node = self.graph.node_info(self.id)?;
        Ok(node.expression().clone_ref())
    }

    fn position(&self) -> FallibleResult<Position> {
        todo!()
    }

    fn set_expression(&self, ast:Ast) -> FallibleResult<()> {
        todo!()
    }

    fn set_expression_text(&self, expression:&str) -> FallibleResult<()> {
        todo!()
    }

    fn set_position(&self, new_position:Position) -> FallibleResult<()> {
        todo!()
    }
}
