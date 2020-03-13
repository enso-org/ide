
use crate::prelude::*;


#[derive(Clone,Debug,Fail)]
#[fail(display = "Node by ID {} was not found.", _0)]
struct NodeNotFound(ast::ID);


pub trait Interface {
    fn get_node(&self, id:ast::ID) -> FallibleResult<Rc<dyn controller::node::Interface>>;
    fn remove_node(&self, id:ast::ID) -> FallibleResult<()>;
}

struct MockGraph {
    nodes:HashMap<ast::ID,Rc<dyn controller::node::Interface>>,
}

struct Handle(Rc<RefCell<MockGraph>>);

impl Interface for Handle {
    fn get_node(&self, id:ast::ID) -> FallibleResult<Rc<dyn controller::node::Interface>> {
        self.0.borrow().nodes.get(&id).cloned().ok_or_else(|| NodeNotFound(id).into())
    }

    fn remove_node(&self, id:ast::ID) -> FallibleResult<()> {
        if let Some(_) = self.0.borrow_mut().nodes.remove(&id) {
            Ok(())
        } else {
            Err(NodeNotFound(id).into())
        }
    }
}



