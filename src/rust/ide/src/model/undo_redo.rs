#![allow(unused_qualifications)]

use crate::prelude::*;

use crate::controller;

#[derive(Debug)]
pub struct Transaction {
    frame : RefCell<Frame>,
    urm   : Weak<Model>,
}

impl Transaction {
    pub fn new(urm:&Rc<Model>) -> Self {
        Self {
            frame: RefCell::new(default()),
            urm: Rc::downgrade(urm),
        }
    }

    pub fn fill_content(&self, id:model::module::Id, content:model::module::Content) {
        with(self.frame.borrow_mut(), |mut data| {
            println!("Filling transaction with info {:?} {:?}", id,content);
            data.code.insert(id,content);
        })
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Some(urm) = self.urm.upgrade() {
            urm.push(self.frame.borrow().clone())
        }
    }
}

// #[derive(Debug,Default)]
// pub struct Context {
// }

#[derive(Clone,Debug,Default)]
pub struct Frame {
    module : Option<model::module::Id>,
    graph : Option<controller::graph::Id>,
    code : std::collections::btree_map::BTreeMap<model::module::Id,model::module::Content>,
}

#[derive(Debug)]
pub struct Data {
    pub frames : Vec<Frame>,
    pub current_transaction : Option<Weak<Transaction>>,
}

#[derive(Debug)]
pub struct Model {
    data : RefCell<Data>
}

impl Model {
    pub fn new() -> Self {
        let data = Data {
            frames : default(),
            current_transaction : default(),
        };
        Model {
            data: RefCell::new(data)
        }
    }

    pub fn transaction(self:&Rc<Self>) -> Rc<Transaction> {
        with(self.data.borrow_mut(), |mut data| {
            if let Some(transaction) = data.current_transaction.as_ref().and_then(Weak::upgrade) {
                transaction
            } else {
                let transaction = Rc::new(Transaction::new(self));
                data.current_transaction = Some(Rc::downgrade(&transaction));
                transaction
            }
        })
    }

    pub fn push(&self, frame:Frame) {
        with(self.data.borrow_mut(), |mut data| {
            println!("Pushing a new frame {:?}", frame);
            data.frames.push(frame);
        })
    }

    pub fn last(&self) -> Option<Frame> {
        // TODO needed ?  if we cant borrow ref...
        self.data.borrow().frames.last().cloned()
    }

    pub fn pop(&self) -> Option<Frame> {
        with(self.data.borrow_mut(), |mut data| {
            data.frames.pop()
        })
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
        println!("{}",module.ast());

        {
            println!("=== BEFORE UNDO");

            if let Some(frame) = urm.last() {
                println!("Undoing to frame {:?}",frame);
                for (id,content) in &frame.code {
                    let path = model::module::Path::from_id(project.content_root_id(),id);
                    println!("Undoing on module {}",path);
                    let mc = controller::module::Handle::new(logger.clone(),path,&project).boxed_local().expect_ok();
                    mc.modify(|info| {
                        info.ast = content.ast.clone();
                    }).unwrap();
                }
            } else {
                println!("Nothing to undo");
            }
            // let frame =
            // let mc = controller::module::Handle::new(&fixture.logger,module.path().clone(),&project).boxed_local().expect_ok();
            // mc.modify(|info| {
            //         info.ast = frame.code.get(frame.)
            //     };
            // });

            // UNDO
            println!("=== AFTER UNDO");
        }
        println!("{:?}",nodes);
        println!("{}",module.ast());
        // // Generate notification.
        // let updated_id = ExpressionId::new_v4();
        // let typename   = crate::test::mock::data::TYPE_NAME;
        // let update     = value_update_with_type(updated_id,typename);
        //
        // // Notification not yet send.
        // let registry          = executed_graph.computed_value_info_registry();
        // let mut notifications = executed_graph.subscribe().boxed_local();
        // notifications.expect_pending();
        // assert!(registry.get(&updated_id).is_none());
        //
        // // Sending notification.
        // execution.computed_value_info_registry().apply_updates(vec![update]);
        // executor.run_until_stalled();
        //
        // // Observing that notification was relayed.
        // // Both computed values update and graph invalidation are expected, in any order.
        // notifications.expect_both(
        //     |notification| match notification {
        //         Notification::ComputedValueInfo(updated_ids) => {
        //             assert_eq!(updated_ids,&vec![updated_id]);
        //             let typename_in_registry = registry.get(&updated_id).unwrap().typename.clone();
        //             let expected_typename    = Some(ImString::new(typename));
        //             assert_eq!(typename_in_registry,expected_typename);
        //             true
        //         }
        //         _ => false,
        //     },
        //     |notification| match notification {
        //         Notification::Graph(graph_notification) => {
        //             assert_eq!(graph_notification,&controller::graph::Notification::PortsUpdate);
        //             true
        //         }
        //         _ => false,
        //     });
        //
        // notifications.expect_pending();
    }
}
