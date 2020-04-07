
use crate::prelude::*;
use crate::network::*;
use crate::node::*;
use crate::debug;





#[derive(Debug,Shrinkwrap)]
pub struct Watched<T> {
    #[shrinkwrap(main_field)]
    target : T,
    handle : WatchHandle
}

impl<T> Watched<T> {
    pub fn new(target:T, handle:WatchHandle) -> Self {
        Self {target,handle}
    }
}

impl<T:HasId> HasId for Watched<T> {
    fn id(&self) -> Id {
        self.target.id()
    }
}


#[derive(Debug)]
pub struct WatchHandle {
    counter : WatchCounter
}

impl WatchHandle {
    pub fn new(counter:&WatchCounter) -> Self {
        let counter = counter.clone_ref();
        counter.increase();
        Self {counter}
    }
}

impl Drop for WatchHandle {
    fn drop(&mut self) {
        self.counter.decrease()
    }
}

#[derive(Debug,Clone,CloneRef,Default)]
pub struct WatchCounter {
    count: Rc<Cell<usize>>
}

impl WatchCounter {
    pub fn new() -> Self {
        default()
    }

    pub fn is_zero(&self) -> bool {
        self.count.get() == 0
    }

    pub fn new_watch(&self) -> WatchHandle {
        WatchHandle::new(self)
    }

    fn increase(&self) {
        self.count.set(self.count.get() + 1);
    }

    fn decrease(&self) {
        self.count.set(self.count.get() - 1);
    }
}



// =================
// === TypeLabel ===
// =================

/// Label of the output type of this FRP node. Used mainly for debugging purposes.
pub trait HasOutputTypeLabel {
    /// Output type label of this object.
    fn output_type_label(&self) -> String;
}



// ======================
// === InputBehaviors ===
// ======================

pub trait InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link>;
}

impl<T> InputBehaviors for T {
    default fn input_behaviors(&self) -> Vec<Link> {
        vec![]
    }
}



// ====================
// === EventEmitter ===
// ====================

/// Any type which can be used as FRP stream output.
pub trait EventOutput = 'static + ValueProvider + EventEmitter + CloneRef + HasId;

/// Implementors of this trait have to know how to emit events to subsequent nodes and how to
/// register new event receivers.
pub trait EventEmitter : HasOutput {
    fn emit_event(&self , value:&Self::Output);
    fn register_target(&self , target:EventInput<Output<Self>>);
    fn register_watch(&self) -> WatchHandle;
}

impl<T:EventEmitter> EventEmitterPoly for T {}
pub trait EventEmitterPoly : EventEmitter {
    fn ping(&self) where Self : HasOutput<Output=()> {
        self.emit_event(&())
    }

    fn emit<T:ToRef<Output<Self>>>(&self, value:T) {
        self.emit_event(value.to_ref())
    }
}



// ======================
// === Event Consumer ===
// ======================

/// Implementors of this trait have to know how to consume incoming events.
pub trait EventConsumer<T> {
    /// Callback for a new incoming event.
    fn on_event(&self, value:&T);
}

/// Implementors of this trait have to know how to consume incoming events. However, it is allowed
/// for them not to consume an event if they were already dropped.
pub trait WeakEventConsumer<T> {
    /// Callback for a new incoming event. Returns true if the event was consumed or false if it was
    /// not. Not consuming an event means that the event receiver was already dropped.
    fn on_event_if_exists(&self, value:&T) -> bool;
}



// =====================
// === ValueProvider ===
// =====================

/// Implementors of this trait have to be able to return their current output value.
pub trait ValueProvider : HasOutput {
    /// The current output value of the FRP node.
    fn value(&self) -> Self::Output;
}



// ==================
// === EventInput ===
// ==================

/// A generalization of any stream input which consumes events of the provided type. This is the
/// slowest bit of the whole FRP network as it uses an trait object, however, we can refactor it
/// in the future to an enum-based trait if needed.
#[derive(Clone)]
pub struct EventInput<Input> {
    data : Rc<dyn WeakEventConsumer<Input>>
}

impl<Def,Input> From<WeakNode<Def>> for EventInput<Input>
where Def:HasOutputStatic, Node<Def>:EventConsumer<Input> {
    fn from(node:WeakNode<Def>) -> Self {
        Self {data:Rc::new(node)}
    }
}

impl<Def,Input> From<&WeakNode<Def>> for EventInput<Input>
where Def:HasOutputStatic, Node<Def>:EventConsumer<Input> {
    fn from(node:&WeakNode<Def>) -> Self {
        Self {data:Rc::new(node.clone_ref())}
    }
}

impl<Input> Debug for EventInput<Input> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"EventInput")
    }
}



// ================
// === NodeData ===
// ================

/// Internal structure of every stream FRP node.
///
/// A few important design decisions are worth mentioning. The `during_call` field is set to `true`
/// after a new event is emitted from this structure and is set to false after the event stops
/// propagating. It is used for preventing events to loop indefinitely. It is especially useful
/// in recursive FRP network. The `watch_counter` field counts the amount of nodes which are not
/// event targets (the `targets` field), but are watching this node and can ask it for the last
/// value any time. If the number of such nodes is zero, the value propagated trough this node does
/// not need to be cached, and it will not be cloned. This minimizes the amount of clones in FRP
/// networks drastically.
#[derive(Debug)]
pub struct NodeData<Out=()> {
    label         : Label,
    targets       : RefCell<Vec<EventInput<Out>>>,
    value_cache   : RefCell<Out>,
    during_call   : Cell<bool>,
    watch_counter : WatchCounter,
}

impl<Out:Default> NodeData<Out> {
    /// Constructor.
    pub fn new(label:Label) -> Self {
        let targets       = default();
        let value_cache   = default();
        let during_call   = default();
        let watch_counter = default();
        Self {label,targets,value_cache,during_call,watch_counter}
    }

    fn use_caching(&self) -> bool {
        !self.watch_counter.is_zero()
    }
}

impl<Out:Data> HasOutput for NodeData<Out> {
    type Output = Out;
}

impl<Out:Data> EventEmitter for NodeData<Out> {
    fn emit_event(&self, value:&Out) {
        if !self.during_call.get() {
            self.during_call.set(true);
            if self.use_caching() {
                *self.value_cache.borrow_mut() = value.clone();
            }
            self.targets.borrow_mut().retain(|target| target.data.on_event_if_exists(value));
            self.during_call.set(false);
        }
    }

    fn register_target(&self,target:EventInput<Out>) {
        self.targets.borrow_mut().push(target)
    }

    fn register_watch(&self) -> WatchHandle {
        self.watch_counter.new_watch()
    }
}

impl<Out:Data> ValueProvider for NodeData<Out> {
    fn value(&self) -> Out {
        self.value_cache.borrow().clone()
    }
}



// ====================
// === Event Stream ===
// ====================

// === Types ===

/// Weak reference to FRP stream node with limited functionality and parametrized only by the
/// output type. This should be the main type used in public FRP APIs.
/// See the docs of `NodeData` to learn more about its internal design.
#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Stream<Out=()> {
    data : Weak<NodeData<Out>>,
}

/// A strong reference to FRP stream node. See the docs of `NodeData` to learn more about its
/// internal design.
#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Node<Def:HasOutputStatic> {
    data       : Rc<NodeData<Output<Def>>>,
    definition : Rc<Def>,
}

/// Weak reference to FRP stream node. See the docs of `NodeData` to learn more about its
/// internal design.
#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct WeakNode<Def:HasOutputStatic> {
    stream     : Stream<Output<Def>>,
    definition : Rc<Def>,
}


// === Output ===

impl<Out:Data>            HasOutput for Stream   <Out> { type Output = Out; }
impl<Def:HasOutputStatic> HasOutput for Node     <Def> { type Output = Output<Def>; }
impl<Def:HasOutputStatic> HasOutput for WeakNode <Def> { type Output = Output<Def>; }


// === Derefs ===

impl<Def> Deref for Node<Def>
where Def:HasOutputStatic {
    type Target = Def;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}


// === Constructors ===

impl<Def:HasOutputStatic> Node<Def> {
    /// Constructor.
    pub fn construct(label:Label, definition:Def) -> Self {
        let data       = Rc::new(NodeData::new(label));
        let definition = Rc::new(definition);
        Self {data,definition}
    }

    /// Constructor which registers the newly created node as the event target of the argument.
    pub fn construct_and_connect<S>(label:Label, stream:&S, definition:Def) -> Self
    where S:EventOutput, Self:EventConsumer<Output<S>> {
        let this = Self::construct(label,definition);
        let weak = this.downgrade();
        stream.register_target(weak.into());
        this
    }

    /// Downgrades to the weak version.
    pub fn downgrade(&self) -> WeakNode<Def> {
        let stream     = Stream {data:Rc::downgrade(&self.data)};
        let definition = self.definition.clone_ref();
        WeakNode {stream,definition}
    }
}

impl<T:HasOutputStatic> WeakNode<T> {
    /// Upgrades to the strong version.
    pub fn upgrade(&self) -> Option<Node<T>> {
        self.stream.data.upgrade().map(|data| {
            let definition = self.definition.clone_ref();
            Node{data,definition}
        })
    }
}

impl<Def> From<Node<Def>> for Stream<Def::Output>
where Def:HasOutputStatic {
    fn from(node:Node<Def>) -> Self {
        let data = Rc::downgrade(&node.data);
        Stream {data}
    }
}


// === EventEmitter ===

impl<Out:Data> EventEmitter for Stream<Out> {
    fn emit_event(&self, value:&Self::Output) {
        self.data.upgrade().for_each(|t| t.emit_event(value))
    }

    fn register_target(&self,target:EventInput<Output<Self>>) {
        self.data.upgrade().for_each(|t| t.register_target(target))
    }

    fn register_watch(&self) -> WatchHandle {
        self.data.upgrade().map(|t| t.register_watch()).unwrap() // FIXME
    }
}

impl<Def:HasOutputStatic> EventEmitter for Node<Def>  {
    fn emit_event      (&self, value:&Output<Def>)           { self.data.emit_event(value) }
    fn register_target (&self,tgt:EventInput<Output<Self>>) { self.data.register_target(tgt) }
    fn register_watch  (&self) -> WatchHandle                { self.data.register_watch() }
}

impl<Def:HasOutputStatic> EventEmitter for WeakNode<Def> {
    fn emit_event      (&self, value:&Output<Def>)           { self.stream.emit_event(value) }
    fn register_target (&self,tgt:EventInput<Output<Self>>) { self.stream.register_target(tgt) }
    fn register_watch  (&self) -> WatchHandle                { self.stream.register_watch() }
}


// === WeakEventConsumer ===

impl<Def,T> WeakEventConsumer<T> for WeakNode<Def>
    where Def:HasOutputStatic, Node<Def>:EventConsumer<T> {
    fn on_event_if_exists(&self, value:&T) -> bool {
        self.upgrade().map(|node| {node.on_event(value);}).is_some()
    }
}


// === ValueProvider ===

impl<Out:Data> ValueProvider for Stream<Out> {
    fn value(&self) -> Self::Output {
        self.data.upgrade().map(|t| t.value()).unwrap_or_default()
    }
}

impl<Def:HasOutputStatic> ValueProvider for Node<Def> {
    fn value(&self) -> Self::Output {
        self.data.value_cache.borrow().clone()
    }
}

impl<Def:HasOutputStatic> ValueProvider for WeakNode<Def> {
    fn value(&self) -> Self::Output {
        self.stream.value()
    }
}


// === HasId ===

impl<Out> HasId for Stream<Out> {
    fn id(&self) -> Id {
        let raw = self.data.as_raw() as *const() as usize;
        raw.into()
    }
}

impl<Def:HasOutputStatic> HasId for Node<Def> {
    fn id(&self) -> Id {
        self.downgrade().id()
    }
}

impl<Def:HasOutputStatic> HasId for WeakNode<Def> {
    fn id(&self) -> Id {
        self.stream.id()
    }
}


// === HasLabel ===

impl<Def:HasOutputStatic> HasLabel for Node<Def>
    where Def:InputBehaviors {
    fn label(&self) -> Label {
        self.data.label
    }
}

// FIXME code quality below:
impl<Def> HasOutputTypeLabel for Node<Def>
    where Def:HasOutputStatic+InputBehaviors {
    fn output_type_label(&self) -> String {
        let label = type_name::<Def>().to_string();
        let label = label.split(|c| c == '<').collect::<Vec<_>>()[0];
        let mut label = label.split(|c| c == ':').collect::<Vec<_>>();
        label.reverse();
        let mut label = label[0];
        let sfx = "Data";
        if label.ends_with(sfx) {
            label = &label[0..label.len()-sfx.len()];
        }
        label.into()
    }
}


// === InputBehaviors ===

impl<Def:HasOutputStatic> InputBehaviors for Node<Def>
where Def:InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![] // FIXME
//        self.data.input_behaviors()
    }
}

impl<Def:HasOutputStatic> InputBehaviors for WeakNode<Def>
    where Def:InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link> {
        self.stream.input_behaviors()
    }
}


// === Debug ===

impl<Out> Debug for Stream<Out> {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data.upgrade() {
            None    => write!(f,"Stream(Dropped)"),
            Some(_) => write!(f,"Stream"),
        }

    }
}








