//! Definition of FRP Network – set of FRP nodes with a common lifetime.

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
pub struct Subnetwork {
    data : Rc<RefCell<Option<Network>>>
}

impl Subnetwork {
    pub fn new() -> Self {
        default()
    }

    fn destroy(&self) {
        *self.data.borrow_mut() = None
    }
}

impl Default for Subnetwork {
    fn default() -> Self {
        let data = Rc::new(RefCell::new(Some(default())));
        Self {data}
    }
}

impl From<Network> for Subnetwork {
    fn from(net:Network) -> Self {
        let data = Rc::new(RefCell::new(Some(net)));
        Self {data}
    }
}



/// Network manages lifetime of set of FRP nodes. FRP networks are designed to be static. You can
/// add new elements while constructing it, but you are not allowed to remove the elements.
/// Moreover, you should not grow the FRP network after it is constructed.
#[derive(Clone,CloneRef,Debug,Default,)]
pub struct Network {
    data : Rc<NetworkData>
}

/// Weak version of `Network`.
#[derive(Clone,CloneRef,Debug)]
pub struct WeakNetwork {
    data : Weak<NetworkData>
}

/// Network item.
pub trait Item : HasId + HasLabel + stream::HasOutputTypeLabel {}
impl<T> Item for T where T : HasId + HasLabel + stream::HasOutputTypeLabel {}

/// Internal data of `Network`.
#[derive(Derivative)]
#[derivative(Debug,Default)]
pub struct NetworkData {
    #[derivative(Debug="ignore")]
    nodes       : RefCell<Vec<Box<dyn Item>>>,
    links       : RefCell<HashMap<Id,Link>>,
    subnetworks : RefCell<Vec<Subnetwork>>,
}





// === API ===

impl NetworkData {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }
}

impl Drop for NetworkData {
    fn drop(&mut self) {
        self.subnetworks.borrow().iter().for_each(|subnetwork| subnetwork.destroy())
    }
}

impl Network {
    /// Constructor.
    pub fn new() -> Self {
        let data = Rc::new(NetworkData::new());
        Self {data}
    }

    /// Get the weak version.
    pub fn downgrade(&self) -> WeakNetwork {
        WeakNetwork {data:Rc::downgrade(&self.data)}
    }

    /// Register the node and return it's weak reference.
    pub fn register_raw<T:HasOutputStatic>(&self, node:stream::Node<T>) -> stream::WeakNode<T> {
        let weak = node.downgrade();
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        weak
    }

    /// Register the node and return a new `Stream` reference.
    pub fn register<Def:HasOutputStatic>(&self, node:stream::Node<Def>) -> Stream<Output<Def>> {
        let stream = node.clone_ref().into();
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        stream
    }

    /// Register a new link between nodes. Visualization purposes only.
    pub fn register_link(&self, target:Id, link:Link) {
        self.data.links.borrow_mut().insert(target,link);
    }

    pub fn register_subnetwork(&self, sub_network:&Subnetwork) {
        self.data.subnetworks.borrow_mut().push(sub_network.clone_ref())
    }

    /// Draw the network using GraphViz.
    pub fn draw(&self) {
        let mut viz = debug::Graphviz::default();
        self.data.nodes.borrow().iter().for_each(|node| {
            viz.add_node(node.id().into(),node.output_type_label(),node.label());
        });
        debug::display_graphviz(viz);
    }
}

impl WeakNetwork {
    /// Upgrade to strong reference.
    pub fn upgrade(&self) -> Option<Network> {
        self.data.upgrade().map(|data| Network {data})
    }
}


/// Link between nodes. It is used for visualization purposes only.
#[derive(Debug,Copy,Clone)]
#[allow(missing_docs)]
pub struct Link {
    pub source : Id,
    pub tp     : LinkType,
}

impl Link {
    /// Event link constructor.
    pub fn event<T:HasId>(t:&T) -> Link {
        let source = t.id();
        let tp     = LinkType::Event;
        Self {source,tp}
    }

    /// Behavior link constructor.
    pub fn behavior<T:HasId>(t:&T) -> Link {
        let source = t.id();
        let tp     = LinkType::Behavior;
        Self {source,tp}
    }

    /// Mixed link constructor.
    pub fn mixed<T:HasId>(t:&T) -> Link {
        let source = t.id();
        let tp     = LinkType::Mixed;
        Self {source,tp}
    }
}

/// Type of the link between FRP nodes.
#[derive(Debug,Clone,Copy)]
#[allow(missing_docs)]
pub enum LinkType {Event,Behavior,Mixed}
