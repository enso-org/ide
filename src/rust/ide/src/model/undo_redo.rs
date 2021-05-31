#![allow(unused_qualifications)]

use crate::prelude::*;

//use crate::model::traits::*;
use crate::controller;

use enso_logger::DefaultTraceLogger as Logger;
use ast::prelude::fmt::Formatter;

/// Trait represents undo-aware type that is able to access undo-redo repository.
///
/// It allows to open transactions and check state of the repository.
/// It does not allow however to execute undo/redo itself, this is done through [`Manager`].
pub trait Aware {
    /// Get handle to undo-redo [Repository].
    fn repository(&self) -> Rc<Repository>;

    /// Get current ongoing transaction. If there is no ongoing transaction, create a one.
    #[must_use]
    fn get_or_open_transaction(&self, name:&str) -> Rc<Transaction> {
        self.repository().transaction(name)
    }
}

/// Transaction is a RAII-style object used to group a number of actions into a single undoable
/// operation.
///
/// When the transaction is dropped, it adds itself to the undo stack, unless it was aborted.
#[derive(Debug)]
pub struct Transaction {
    #[allow(missing_docs)]
    pub logger : Logger,
    frame      : RefCell<Frame>,
    urm        : Weak<Repository>,
    aborted    : Cell<bool>,
}

impl Transaction {
    /// Create a new transaction, that will add to the given's repository undo stack on destruction.
    pub fn new(urm:&Rc<Repository>, name:String) -> Self {
        Self {
            logger : Logger::sub(&urm.logger,"Transaction"),
            frame: RefCell::new(Frame{name,..default()}),
            urm: Rc::downgrade(urm),
            aborted : default(),
        }
    }

    /// Get the transaction name.
    ///
    /// Currently the name serves only debugging purposes.
    pub fn name(&self) -> String {
        self.frame.borrow().name.clone()
    }

    /// Stores the state of given module.
    ///
    /// This is the state that will be restored, when the transaction is undone. As such is should
    /// be the state "from before" the undoable action.
    ///
    /// This method stores content only once for given module. Thus it is safe to call this on
    /// the current transaction in context where it is not clear whether transaction was already set
    /// up or not.
    pub fn fill_content(&self, id:model::module::Id, content:model::module::Content) {
        with(self.frame.borrow_mut(), |mut data| {
            debug!(self.logger, "Filling transaction '{data.name}' with info for '{id}':\n{content}");
            let _ = data.shapshots.try_insert(id, content);
        })
    }

    /// Abort the transaction.
    ///
    /// Aborted transaction when dropped is discarded, rather than being put on top of "Redo" stack.
    /// It does not affect the actions belonging to transaction in any way.
    pub fn abort(&self) {
        self.aborted.set(true)
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Some(urm) = self.urm.upgrade() {
            if !self.aborted.get() {
                info!(self.logger, "Transaction '{self.name()}' will create a new frame.");
                urm.push_to(Stack::Undo, self.frame.borrow().clone());
                urm.clear(Stack::Redo);
            } else {
                info!(self.logger, "Dropping the aborted transaction '{self.name()}' without \
                pushing a frame to repository.")
            }
        }
    }
}

#[derive(Clone,Debug,Default)]
pub struct Frame {
    /// Name of the transaction that created this frame.
    pub name   : String,
    /// Context module where the change was made.
    pub module : Option<model::module::Id>,
    /// Context graph where the change was made.
    pub graph  : Option<controller::graph::Id>,
    /// Snapshots of content for all edited modules.
    pub shapshots: std::collections::btree_map::BTreeMap<model::module::Id,model::module::Content>,
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"Name: {}; ", self.name)?;
        if let Some(m) = &self.module {write!(f,"Module: {}; ", m)?; }
        if let Some(g) = &self.graph  {write!(f,"Graph: {}; ",  g)?; }
        for (id,code) in &self.shapshots {
            write!(f,"Code for {}: {}; ", id, code)?;
        }
        Ok(())
    }
}

/// The inner state of the Und-Redo repository.
#[derive(Debug,Default)]
pub struct Data {
    pub undo: Vec<Frame>,
    pub redo: Vec<Frame>,
    pub current_transaction : Option<Weak<Transaction>>,
}

/// Identifies a stack in Undo-Redo repository.
#[derive(Clone,Copy,Debug,Display)]
#[allow(missing_docs)]
pub enum Stack {
    Undo,
    Redo,
}

/// Repository stores undo and redo stacks.
///
/// Also, provides API that allows open transactions that will add themselves to the undo stack.
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
    /// Create a new repository.
    pub fn new(parent:impl AnyLogger) -> Self {
        Self {
            logger : Logger::sub(parent,"Repository"),
            data   : default(),
        }
    }

    /// Get the currently open transaction. [`None`] if there is none.
    pub fn current_transaction(&self) -> Option<Rc<Transaction>> {
        self.data.borrow().current_transaction.as_ref().and_then(Weak::upgrade)
    }

    /// Open a new transaction.
    ///
    /// If there is already an opened transaction, it will returned as [`Err`].
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

    fn borrow_mut(&self, stack:Stack) -> RefMut<Vec<Frame>> {
        let data = self.data.borrow_mut();
        match stack {
            Stack::Undo => RefMut::map(data,|d| &mut d.undo),
            Stack::Redo => RefMut::map(data,|d| &mut d.redo),
        }
    }

    fn push_to(&self, stack:Stack, frame:Frame) {
        debug!(self.logger, "Pushing to {stack} stack a new frame: {frame}");
        self.borrow_mut(stack).push(frame);
    }

    fn clear(&self, stack:Stack) {
        debug!(self.logger, "Clearing {stack} stack.");
        self.borrow_mut(stack).clear();
    }

    pub fn clear_all(&self) {
        for stack in [Stack::Undo,Stack::Redo] {
            self.clear(stack)
        };
    }

    pub fn last(&self) -> Option<Frame> {
        // TODO needed ?  if we cant borrow ref...
        self.data.borrow().undo.last().cloned()
    }

    fn pop(&self) -> Option<Frame> {
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
            repository : Rc::new(Repository::new(&logger)),
            modules : default(),
            logger,
        }
    }

    pub fn module_opened(&self, module:model::Module) {
        self.modules.borrow_mut().insert(module.id(),module);
    }
    pub fn module_closed(&self, _module:model::Module) {
        //self.modules.borrow_mut().remove(&module.id());
    }

    pub fn reset_to(&self, frame:&Frame) -> FallibleResult {
        //use utils::test::traits::*;
        use failure::format_err;
        warning!(self.logger,"Resetting to initial state on frame {frame}");

        // First we must have all modules resolved. Only then we can start applying changes.
        // Otherwise, if one of the modules could not be retrieved, we'd risk ending up with
        // a partially undone operation and inconsistent state.
        //
        // In general this should never happen, as we store strong references to all opened modules
        // and don't allow getting snapshots of modules that are not opened.
        let module_and_content = with(self.modules.borrow(), |modules| {
            frame.shapshots.iter()
                .map(|(id,content)| {
                    let err           = || format_err!("Cannot find handle to module {}", id);
                    let module_result = modules.get(id).cloned().ok_or_else(err);
                    module_result.map(|module| (module,content.clone()))
                })
                .collect::<FallibleResult<Vec<_>>>()
        })?;

        for (module,content) in module_and_content {
            warning!(self.logger,"Undoing on module {module.path()}");
            // The below should never fail, because it can fail only if serialization to code fails.
            // And it cannot fail, as it already underwent this procedure successfully in the past
            // (we are copying an old state, so it must ba a representable state).
            module.update_whole(content.clone())?;
        }
        Ok(())
    }

    pub fn undo(&self) -> FallibleResult {
        debug!(self.logger, "Undo requested, stack size is {self.repository.len()}.");

        let frame = self.repository.last().ok_or_else(|| failure::format_err!("Nothing to undo"))?;

        let undo_transaction = self.repository.open_transaction("Undo faux transaction").map_err(|_| failure::format_err!("Cannot undo while there is an ongoing transaction."))?;
        undo_transaction.abort();
        self.reset_to(&frame)?;
        self.repository.pop(); // TODO upewnić się, że to to, co cofnęliśmy

        let undo_transaction = Rc::try_unwrap(undo_transaction).map_err(|_| failure::format_err!("Someone stole the undo/redo internal transaction. Should never happen."))?;
        self.repository.data.borrow_mut().redo.push(undo_transaction.frame.borrow().clone());
        Ok(())
    }

    pub fn redo(&self) -> FallibleResult {
        let frame = self.repository.data.borrow_mut().redo.pop().ok_or_else(|| failure::format_err!("Nothing to redo"))?;
        let redo_transaction = self.get_or_open_transaction(&frame.name);
        redo_transaction.abort();
        self.reset_to(&frame)?;
        self.repository.data.borrow_mut().undo.push(redo_transaction.frame.borrow().clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //use utils::test::traits::*;
    use crate::test::mock::Fixture;
    use super::*;

    #[test]
    #[allow(unused_variables)]
    fn move_node() {
        use model::module::Position;

        let mut fixture = crate::test::mock::Unified::new().fixture();
        let Fixture{executed_graph,execution,executor,graph,project,module,logger,..} = &mut fixture;
        let logger:&DefaultTraceLogger = logger;

        let urm = project.urm();
        let nodes = executed_graph.graph().nodes().unwrap();
        let node = &nodes[0];

        debug!(logger, "{node.position():?}");

        graph.set_node_position(node.id(), Position::new(500.0, 250.0)).unwrap();
        graph.set_node_position(node.id(), Position::new(300.0, 150.0)).unwrap();

        assert_eq!(graph.node(node.id()).unwrap().position(), Some(model::module::Position::new(300.0, 150.0)));
        project.urm().undo().unwrap();
        assert_eq!(graph.node(node.id()).unwrap().position(), Some(model::module::Position::new(500.0, 250.0)));
        project.urm().undo().unwrap();
        assert_eq!(graph.node(node.id()).unwrap().position(), None);
        project.urm().redo().unwrap();
        assert_eq!(graph.node(node.id()).unwrap().position(), Some(model::module::Position::new(500.0, 250.0)));
        project.urm().redo().unwrap();
        assert_eq!(graph.node(node.id()).unwrap().position(), Some(model::module::Position::new(300.0, 150.0)));
    }

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

        // Check initial state.
        assert_eq!(urm.repository.len(), 0);
        assert_eq!(module.ast().to_string(), "main = \n    2 + 2");

        // Perform an action.
        executed_graph.graph().set_expression(node.info.id(),"5 * 20").unwrap();

        // We can undo action.
        assert_eq!(urm.repository.len(), 1);
        assert_eq!(module.ast().to_string(), "main = \n    5 * 20");
        project.urm().undo().unwrap();
        assert_eq!(module.ast().to_string(), "main = \n    2 + 2");

        // We cannot undo more actions than we made.
        assert_eq!(urm.repository.len(), 0);
        assert!(project.urm().undo().is_err());
        assert_eq!(module.ast().to_string(), "main = \n    2 + 2");

        // We can redo since we undid.
        project.urm().redo().unwrap();
        assert_eq!(module.ast().to_string(), "main = \n    5 * 20");

        // And we can undo once more.
        project.urm().undo().unwrap();
        assert_eq!(module.ast().to_string(), "main = \n    2 + 2");

        //We cannot redo after edit has been made.
        executed_graph.graph().set_expression(node.info.id(),"4 * 20").unwrap();
        assert!(project.urm().redo().is_err());
    }
}
