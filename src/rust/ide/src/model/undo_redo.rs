#![allow(unused_qualifications)]

use crate::prelude::*;

use crate::controller;

use enso_logger::DefaultTraceLogger as Logger;
use ast::prelude::fmt::Formatter;

#[derive(Debug)]
pub struct Transaction {
    logger : Logger,
    frame : RefCell<Frame>,
    urm   : Weak<Repository>,
    aborted : Cell<bool>,
}

pub trait Aware {
    fn repository(&self) -> Rc<Repository>;
    #[must_use]
    fn get_or_open_transaction(&self, name:&str) -> Rc<Transaction> {
        self.repository().transaction(name)
    }
}

impl Transaction {
    pub fn new(urm:&Rc<Repository>, name:String) -> Self {
        Self {
            logger : Logger::sub(&urm.logger,"Transaction"),
            frame: RefCell::new(Frame{name,..default()}),
            urm: Rc::downgrade(urm),
            aborted : default(),
        }
    }

    pub fn name(&self) -> String {
        self.frame.borrow().name.clone()
    }

    pub fn fill_content(&self, id:model::module::Id, content:model::module::Content) {
        with(self.frame.borrow_mut(), |mut data| {
            debug!(self.logger, "Filling transaction '{data.name}' with info for '{id}':\n{content}");
            let _ = data.code.try_insert(id,content);
        })
    }

    pub fn abort(&self) {
        self.aborted.set(true)
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Some(urm) = self.urm.upgrade() {
            if !self.aborted.get() {
                info!(self.logger, "Transaction '{self.name()}' will create a new frame.");
                urm.push(self.frame.borrow().clone())
            } else {
                info!(self.logger, "Dropping the aborted transaction '{self.name()}' without pushing frame.")
            }
        }
    }
}

#[derive(Clone,Debug,Default)]
pub struct Frame {
    name : String,
    module : Option<model::module::Id>,
    graph : Option<controller::graph::Id>,
    code : std::collections::btree_map::BTreeMap<model::module::Id,model::module::Content>,
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"Name: {}; ", self.name)?;
        if let Some(m) = &self.module {write!(f,"Module: {}; ", m)?; }
        if let Some(g) = &self.graph  {write!(f,"Graph: {}; ",  g)?; }
        for (id,code) in &self.code {
            write!(f,"Code for {}: {}; ", id, code)?;
        }
        Ok(())
    }
}

#[derive(Debug,Default)]
pub struct Data {
    pub undo: Vec<Frame>,
    pub redo: Vec<Frame>,
    pub current_transaction : Option<Weak<Transaction>>,
}

#[derive(Debug)]
pub struct Repository {
    logger : Logger,
    data : RefCell<Data>,
}

impl Default for Repository {
    fn default() -> Self {
        Self::new(Logger::new(""))
    }
}

impl Repository {
    pub fn new(parent:impl AnyLogger) -> Self {
        Self {
            logger: Logger::sub(parent,"Repository"),
            data : default(),
        }
    }

    pub fn current_transaction(&self) -> Option<Rc<Transaction>> {
        self.data.borrow().current_transaction.as_ref().and_then(Weak::upgrade)
    }

    pub fn open_transaction(self:&Rc<Self>, name:impl Into<String>) -> Result<Rc<Transaction>,Rc<Transaction>> {
        if let Some(ongoing_transaction) = self.current_transaction() {
            Err(ongoing_transaction)
        } else {
            let name = name.into();
            debug!(self.logger, "Creating a new transaction `{name}`");
            let new_transaction = Rc::new(Transaction::new(self, name));
            self.data.borrow_mut().current_transaction = Some(Rc::downgrade(&new_transaction));
            Ok(new_transaction)
        }
    }

    pub fn transaction(self:&Rc<Self>, name:impl Into<String>) -> Rc<Transaction> {
        self.open_transaction(name).into_ok_or_err()
    }

    pub fn push(&self, frame:Frame) {
        with(self.data.borrow_mut(), |mut data| {
            debug!(self.logger, "Pushing a new frame {frame}");
            data.undo.push(frame);
        })
    }

    pub fn last(&self) -> Option<Frame> {
        // TODO needed ?  if we cant borrow ref...
        self.data.borrow().undo.last().cloned()
    }

    pub fn pop(&self) -> Option<Frame> {
        with(self.data.borrow_mut(), |mut data| {
            let frame = data.undo.pop();
            if let Some(frame) = &frame {
                debug!(self.logger, "Popping a frame: {frame}, remained: {data.undo.len()}");
            }
            frame
        })
    }

    pub fn len(&self) -> usize {
        self.data.borrow().undo.len()
    }
}

#[derive(Debug)]
pub struct Manager {
    pub logger : Logger,
    pub repository : Rc<Repository>,
    modules : RefCell<BTreeMap<model::module::Id,model::Module>>
}

impl Aware for Manager {
    fn repository(&self) -> Rc<Repository> {
        self.repository.clone()
    }
}

impl Manager {
    pub fn new() -> Self {
        let logger = Logger::new("URM");
        Self {
            repository :Rc::new(Repository::new(&logger)),
            modules : default(),
            logger,
        }
    }

    pub fn module_opened(&self, module:model::Module) {
        self.modules.borrow_mut().insert(module.id(),module);
    }
    pub fn module_closed(&self, module:model::Module) {
        // TODO
        // jak niepotrzebne (nie ma w transakcjach), mozna zrzucic
    }

    pub fn reset_to(&self, frame:&Frame, project:&dyn model::project::API) -> FallibleResult {
        use utils::test::traits::*;
        warning!(self.logger,"Resetting to initial state on frame {frame}");
        for (id,content) in &frame.code {
            let path = model::module::Path::from_id(project.content_root_id(),id);
            warning!(self.logger,"Undoing on module {path}");
            let mc = controller::module::Handle::new(&self.logger,path,project).boxed_local().expect_ok();
            mc.modify(|info| {
                warning!(self.logger, "Restoring code to:\n{content}");
                info.ast = content.ast.clone();
            }).unwrap();
        }
        Ok(())
    }

    pub fn undo(&self, project:&dyn model::project::API) -> FallibleResult {

        let undo_transaction = self.repository.open_transaction("Undo faux transaction").map_err(|_| failure::format_err!("Cannot undo while there is an ongoing transaction."))?;
        undo_transaction.abort();

        if let Some(frame) = self.repository.last() {
            self.reset_to(&frame,project)?;
            self.repository.pop(); // TODO upewnić się, że to to, co cofnęliśmy
        } else {
            warning!(self.logger,"Nothing to undo");
        }

        // TODO
        assert_eq!(Rc::strong_count(&undo_transaction), 1);
        self.repository.data.borrow_mut().redo.push(undo_transaction.frame.borrow().clone());
        Ok(())
    }

    pub fn redo(&self, project:&dyn model::project::API) -> FallibleResult {
        use utils::test::traits::*;
        let frame = self.repository.data.borrow_mut().redo.pop().ok_or_else(|| failure::format_err!("Nothing to redo"))?;
        self.reset_to(&frame,project);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use utils::test::traits::*;
    use super::*;
    use enso_protocol::language_server::ExpressionId;
    use enso_protocol::language_server::test::value_update_with_type;
    use utils::test::traits::*;
    use crate::controller::graph::executed::Notification;

    // Test that checks that value computed notification is properly relayed by the executed graph.
    #[test]
    #[allow(unused_variables)]
    fn undo_redo() {
        use crate::test::mock::Fixture;
        // Setup the controller.
        let mut fixture = crate::test::mock::Unified::new().fixture();
        let Fixture{executed_graph,execution,executor,graph,project,module,logger,..} = &mut fixture;

        let urm = project.urm();
        let nodes = executed_graph.graph().nodes().unwrap();
        let node = &nodes[0];
        executed_graph.graph().set_expression(node.info.id(),"5 * 20").unwrap();

        println!("{:?}",nodes);

        {
            println!("=== BEFORE UNDO");
            println!("{}",module.ast());
            project.urm().undo(&**project);
            println!("=== AFTER UNDO");
            println!("{}",module.ast());
        }
        println!("{:?}",nodes);
    }
}
