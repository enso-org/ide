//! Submodules of the node controller.

pub mod mock;

use crate::prelude::*;

use ast::Ast;
use ast::ID;
use nalgebra::Vector2;

use flo_stream::Subscriber;



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
    /// Create a new `Position` value.
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

/// Interface of the node controller.
pub trait Interface {
    /// Gets the node unique ID.
    fn id(&self) -> ID;

    /// Gets the node's expression's AST.
    fn expression(&self) -> FallibleResult<Ast>;

    /// Gets the node's visual position.
    fn position(&self) -> FallibleResult<Position>;

    /// Sets the node's expression to a new AST.
    fn set_expression(&self, ast:Ast) -> FallibleResult<()>;

    /// Sets the node's expression to a result of parsing the given text.
    fn set_expression_text(&self, expression:&str) -> FallibleResult<()>;

    /// Sets the node's position.
    fn set_position(&self, new_position:Position) -> FallibleResult<()>;

    /// Subscribes to the notifications of the controller.
    fn subscribe(&mut self) -> Subscriber<controller::notification::Node>;
}



// ==================
// === Controller ===
// ==================

/// Node controller.
#[derive(Clone,Debug)]
pub struct Controller {
    graph : controller::graph::Handle,
    id    : ID,
}

impl Controller {
    /// Creates a new node controller, providing a view into a graph's node.
    pub fn new(graph:controller::graph::Handle, id:ID) -> Controller {
        // TODO [mwu] notification
        Controller {graph,id}
    }
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

    fn set_expression(&self, _ast:Ast) -> FallibleResult<()> {
        todo!()
    }

    fn set_expression_text(&self, _expression:&str) -> FallibleResult<()> {
        todo!()
    }

    fn set_position(&self, _new_position:Position) -> FallibleResult<()> {
        todo!()
    }

    fn subscribe(&mut self) -> Subscriber<controller::notification::Node> {
        todo!()
    }
}
