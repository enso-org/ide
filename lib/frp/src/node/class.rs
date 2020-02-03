

use crate::prelude::*;
use crate::debug::*;
use crate::node::id::*;
use crate::node::label::*;
use crate::data::*;



// ===============
// === DynNode ===
// ===============

/// Type level association between FRP data and dynamic node type. The associated `DynNode` type can
/// differ depending on whether it is an event or behavior node, as they provide different APIs.
/// For example, behaviors allow lookup for the current value, which does not make sense in case
/// of events.
pub trait KnownDynNode {
    /// The node storage type.
    type DynNode: DynNodeBounds + CloneRef;
}

/// Accessor.
pub type DynNode<T> = <T as KnownDynNode>::DynNode;

alias! {
    /// Bounds required for every node.
    DynNodeBounds = {
        Debug + GraphvizBuilder + HasId + HasDisplayId + HasInputs + HasLabel + KnownOutput
    }
}

// === EventDynNode ===

/// Newtype wrapper for any event node.
#[derive(Debug,Derivative,Shrinkwrap)]
#[derivative(Clone(bound=""))]
pub struct EventDynNode<Out> {
    rc: Rc<dyn EventDynNodeBounds<Output=EventData<Out>>>,
}

alias! {
    /// Bounds for any event node.
    EventDynNodeBounds = { DynNodeBounds + HasEventTargets + EventEmitter }
}

impl<Out:Value> KnownDynNode for EventData<Out> {
    type DynNode = EventDynNode<Out>;
}

impl<Out> Unwrap     for EventDynNode<Out> {}
impl<Out> CloneRef   for EventDynNode<Out> {}
impl<Out> HasContent for EventDynNode<Out> {
    // TODO: Simplify after fixing https://github.com/rust-lang/rust/issues/68776
    type Content = <EventDynNode<Out> as Deref>::Target;
}

impl<Out:Value> KnownOutput for EventDynNode<Out> {
    type Output = EventData<Out>;
}


// === BehaviorDynNodeBounds ===

/// Newtype wrapper for any behavior node.
#[derive(Debug,Derivative,Shrinkwrap)]
#[derivative(Clone(bound=""))]
pub struct BehaviorDynNode<Out> {
    rc: Rc<dyn BehaviorDynNodeBounds<Output=BehaviorData<Out>>>,
}

alias! {
    /// Bounds for any behavior node.
    BehaviorDynNodeBounds = { DynNodeBounds + HasCurrentValue  }
}

impl<Out:Value> KnownDynNode for BehaviorData<Out> {
    type DynNode = BehaviorDynNode<Out>;
}

impl<Out> Unwrap     for BehaviorDynNode<Out> {}
impl<Out> CloneRef   for BehaviorDynNode<Out> {}
impl<Out> HasContent for BehaviorDynNode<Out> {
    // TODO: Simplify after fixing https://github.com/rust-lang/rust/issues/68776
    type Content = <BehaviorDynNode<Out> as Deref>::Target;
}

impl<Out:Value> KnownOutput for BehaviorDynNode<Out> {
    type Output = BehaviorData<Out>;
}



// ============
// === Node ===
// ============

// === Types ===

/// The type of any FRP node which produces event messages. Having a reference to a node is like
/// having a reference to network endpoint which transmits messages of a given type. Thus, it is a
/// nice mental simplification to think about it just like about an event (stream).
pub type Event<T> = Node<EventData<T>>;

/// The type of any FRP node which can be queried for behavior value. Having a reference to a node
/// is like having a reference to network endpoint which transmits messages of a given type. Thus,
/// it is a nice mental simplification to think about it just like about a behavior.
pub type Behavior <T> = Node<BehaviorData<T>>;


// === Definition ===

/// Node is used as a common types for frp operations. For example, `Event<T>` is just an alias to
/// `Node<EventData<T>>`.
#[derive(Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
pub struct Node<Out:KnownDynNode> {
    storage: DynNode<Out>,
}

impl<Out:Data> Node<Out> {
    /// Constructor.
    pub fn new(storage:DynNode<Out>) -> Self {
        Self {storage}
    }
}


// === Type Deps ===

impl<Out:Data> KnownOutput for Node<Out> { type Output  = Out; }
impl<Out:Data> HasContent  for Node<Out> { type Content = DynNode<Out>; }
impl<Out:Data> Unwrap      for Node<Out> {}


// === Instances ===

impl<Out:Data> Deref for Node<Out> {
    type Target = DynNode<Out>;
    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<Out:Data> CloneRef for Node<Out> {
    fn clone_ref(&self) -> Self {
        let storage = self.storage.clone_ref();
        Self {storage}
    }
}


// === Construction ===

impl<Out:Data> From<&Node<Out>> for Node<Out> {
    fn from(t:&Node<Out>) -> Self {
        t.clone_ref()
    }
}

impl<Storage,Out:Value>
From<&Storage> for Behavior<Out>
    where Storage : BehaviorDynNodeBounds<Output=BehaviorData<Out>> + Clone + 'static {
    fn from(storage:&Storage) -> Self {
        Self::new(BehaviorDynNode{rc:Rc::new(storage.clone())})
    }
}

impl<Storage,Out:Value>
From<&Storage> for Event<Out>
    where Storage : EventDynNodeBounds<Output=EventData<Out>> + Clone + 'static {
    fn from(storage:&Storage) -> Self {
        Self::new(EventDynNode{rc:Rc::new(storage.clone())})
    }
}


// === AddTarget ===

/// Abstraction for adding a target to a given node. Nodes which carry behaviors do not need to
/// perform any operation here, while event streams want to register the nodes they want to send
/// notifications to.
pub trait AddTarget<T> {
    /// Adds a node as a target of the current flow.
    fn add_target(&self,t:&T);
}

impl<S,T:Value> AddTarget<S> for Event<T>
    where for<'t> &'t S : Into<AnyEventConsumer<EventData<T>>> {
    fn add_target(&self,t:&S) {
        self.add_event_target(t.into())
    }
}

impl<S,T:Value> AddTarget<S> for Behavior<T> {
    fn add_target(&self,_:&S) {}
}



// ===============
// === AnyNode ===
// ===============

#[derive(Debug,Shrinkwrap)]
pub struct AnyNode {
    rc: Rc<dyn AnyNodeOps>,
}

alias! { AnyNodeOps = { Debug + GraphvizBuilder + HasId + HasDisplayId + KnownOutputType } }


// === Instances ===

impls! { [Out:Data+'static] From <&Node<Out>> for AnyNode { |t| t.clone_ref().into() } }
impls! { [Out:Data+'static] From  <Node<Out>> for AnyNode { |t| Self {rc:Rc::new(t)} } }

impl KnownOutputType for AnyNode {
    fn output_type(&self) -> DataType {
        self.rc.output_type()
    }

    fn output_type_value_name(&self) -> String {
        self.rc.output_type_value_name()
    }
}



// =====================
// === EventConsumer ===
// =====================

// === Definition ===

/// Abstraction for nodes which are able to consume events.
pub trait EventConsumer: KnownEventInput + Debug {
    /// Function called on every new received event.
    fn on_event(&self, input:&Self::EventInput);
}


// === AnyEventConsumer ===

/// Abstraction for any node which consumes events of a given type.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct AnyEventConsumer<In> {
    raw: Rc<dyn EventConsumer<EventInput=In>>,
}

impl<In:Data> AnyEventConsumer<In> {
    /// Constructor.
    pub fn new<A:EventConsumer<EventInput=In>+'static>(a:A) -> Self {
        let raw = Rc::new(a);
        Self {raw}
    }
}

impl<T,In> From<&T> for AnyEventConsumer<In>
    where T  : EventConsumer<EventInput=In> + Clone + 'static,
          In : Data {
    fn from(t:&T) -> Self {
        Self::new(t.clone())
    }
}



// ====================
// === EventEmitter ===
// ====================

// === Definition ===

/// Abstraction for nodes which are able to consume events.
pub trait EventEmitter: KnownOutput {
    /// Function called on every new received event.
    fn emit(&self, event:&Self::Output);
}

impl<T> EventEmitter for T
    where T:Unwrap+KnownOutput, Content<T>:EventEmitter<Output=Output<Self>> {
    fn emit(&self, event:&Self::Output) {
        self.unwrap().emit(event)
    }
}
