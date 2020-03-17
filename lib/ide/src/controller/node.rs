//! Submodules of the node controller.

pub mod mock;

use crate::prelude::*;

use ast::Ast;
use ast::ID;
use nalgebra::Vector2;

use serde::Serialize;
use serde::Deserialize;

use flo_stream::Subscriber;



// ================
// === Position ===
// ================

/// Used e.g. for node position
#[derive(Clone,Copy,Debug,PartialEq,Serialize,Deserialize)]
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

    /// Sets the node's position.
    fn set_position(&self, new_position:Position) -> FallibleResult<()>;

    /// Sets the node's expression to a new AST.
    fn set_expression(&self, ast:Ast) -> FallibleResult<()>;

    /// Sets the node's expression to a result of parsing the given text.
    fn set_expression_text(&self, expression:&str) -> FallibleResult<()>;

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
        self.graph.get_module().get_node_position(self.id)
    }

    fn set_position(&self, new_position:Position) -> FallibleResult<()> {
        Ok(self.graph.get_module().set_node_position(self.id, new_position))
    }

    fn set_expression(&self, _ast:Ast) -> FallibleResult<()> {
        todo!()
    }

    fn set_expression_text(&self, _expression:&str) -> FallibleResult<()> {
        todo!()
    }

    fn subscribe(&mut self) -> Subscriber<controller::notification::Node> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use enso_prelude::default;
    use crate::double_representation::definition::DefinitionName;
    use crate::double_representation::graph::Id;
    use crate::controller;
    use controller::node::Position;
    use controller::module;
    use controller::graph;
    use controller::graph::Interface;
    use json_rpc::test_util::transport::mock::MockTransport;
    use parser::Parser;
    use uuid::Uuid;
    use wasm_bindgen_test::wasm_bindgen_test;
    use basegl_system_web::set_stdout;


    #[wasm_bindgen_test]
    fn node_operations() {
        set_stdout();
        let transport    = MockTransport::new();
        let file_manager = file_manager_client::Handle::new(transport);
        let parser       = Parser::new().unwrap();
        let location     = module::Location("Test".to_string());

        let code         = "main = Hello World";
        let idmap        = default();

        let module       = module::Handle::new_mock
            (location,code,idmap,file_manager,parser).unwrap();

        let uid          = Uuid::new_v4();
        let pos          = Position::default();

        module.set_node_position(uid, pos);

        let crumbs       = vec![DefinitionName::new_plain("main")];
        let controller   = graph::Handle::new(module, Id {crumbs}).unwrap();


        assert_eq!(controller.get_node(uid).unwrap().position().unwrap(), pos);
    }
}