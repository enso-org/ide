use crate::prelude::*;
use crate::stream;
use crate::stream::Stream;
use crate::node::*;
use crate::debug;

// ===============
// === Network ===
// ===============

// === Definition ===

#[derive(Clone,CloneRef,Debug)]
pub struct Network {
    data : Rc<NetworkData>
}

#[derive(Clone,CloneRef,Debug)]
pub struct WeakNetwork {
    data : Weak<NetworkData>
}

pub trait Anyyy : HasId + HasLabel + stream::HasOutputTypeLabel {}
impl<T> Anyyy for T where T : HasId + HasLabel + stream::HasOutputTypeLabel {}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NetworkData {
    #[derivative(Debug="ignore")]
    nodes : RefCell<Vec<Box<dyn Anyyy>>>,
    links : RefCell<HashMap<Id,Link>>,
}

#[derive(Debug,Clone)]
pub struct Link {
    pub source : Id,
    pub tp     : LinkType,
}

impl Link {
    pub fn event<T:HasId>(t:&T) -> Link {
        let source = t.id();
        let tp     = LinkType::Event;
        Self {source,tp}
    }

    pub fn behavior<T:HasId>(t:&T) -> Link {
        let source = t.id();
        let tp     = LinkType::Behavior;
        Self {source,tp}
    }

    pub fn mixed<T:HasId>(t:&T) -> Link {
        let source = t.id();
        let tp     = LinkType::Mixed;
        Self {source,tp}
    }
}

#[derive(Debug,Clone,Copy)]
pub enum LinkType {Event,Behavior,Mixed}



// === API ===

impl NetworkData {
    /// Constructor.
    pub fn new() -> Self {
        let nodes = default();
        let links = default();
        Self {nodes,links}
    }
}

impl Network {
    /// Constructor.
    pub fn new() -> Self {
        let data = Rc::new(NetworkData::new());
        Self {data}
    }

    pub fn downgrade(&self) -> WeakNetwork {
        WeakNetwork {data:Rc::downgrade(&self.data)}
    }

    pub fn register_raw<T:HasOutputStatic>(&self, node:stream::Node<T>) -> stream::WeakNode<T> {
        let weak = node.downgrade();
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        weak
    }

    pub fn register<Def:HasOutputStatic>(&self, node:stream::Node<Def>) -> Stream<Output<Def>> {
        let stream = node.clone_ref().into();
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        stream
    }

    pub fn register_link(&self, target:Id, link:Link) {
        self.data.links.borrow_mut().insert(target,link);
    }

    pub fn draw(&self) {
        let mut viz = debug::Graphviz::default();
        self.data.nodes.borrow().iter().for_each(|node| {
            viz.add_node(node.id().into(),node.output_type_label(),node.label());
            println!(">>> {:?}",node.id())
        });
        debug::display_graphviz(viz);
    }
}

impl WeakNetwork {
    pub fn upgrade(&self) -> Option<Network> {
        self.data.upgrade().map(|data| Network {data})
    }
}

