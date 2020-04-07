
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









// ============
// === Data ===
// ============

/// Data that flows trough the FRP network.
pub trait Data = 'static + Clone + Debug + Default;



// =================
// === HasOutput ===
// =================

/// Implementors of this trait has to know their output type.
#[allow(missing_docs)]
pub trait HasOutput {
    type Output : Data;
}

/// A static version of `HasOutput`.
pub trait HasOutputStatic = 'static + HasOutput;


/// Accessor of the accosiated `Output` type.
pub type Output<T> = <T as HasOutput>::Output;



// ====================
// === EventEmitter ===
// ====================

/// Any type which can be used as FRP stream output.
pub trait StreamOutput = 'static + ValueProvider + EventEmitter + CloneRef + HasId;

/// Implementors of this trait have to know how to emit events to subsequent nodes and how to
/// register new event receivers.
pub trait EventEmitter : HasOutput {
    fn emit_event(&self , value:&Self::Output);
    fn register_target(&self , target:StreamInput<Output<Self>>);
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




// ===================
// === StreamInput ===
// ===================

/// A generalization of any stream input which consumes events of the provided type. This is the
/// slowest bit of the whole FRP network as it uses an trait object, however, we can refactor it
/// in the future to an enum-based trait if needed.
#[derive(Clone)]
pub struct StreamInput<Input> {
    data : Rc<dyn WeakEventConsumer<Input>>
}

impl<Def,Input> From<WeakStreamNode<Def>> for StreamInput<Input>
where Def:HasOutputStatic, StreamNode<Def>:EventConsumer<Input> {
    fn from(node:WeakStreamNode<Def>) -> Self {
        Self {data:Rc::new(node)}
    }
}

impl<Def,Input> From<&WeakStreamNode<Def>> for StreamInput<Input>
where Def:HasOutputStatic, StreamNode<Def>:EventConsumer<Input> {
    fn from(node:&WeakStreamNode<Def>) -> Self {
        Self {data:Rc::new(node.clone_ref())}
    }
}

impl<Input> Debug for StreamInput<Input> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"StreamInput")
    }
}




// ======================
// === StreamNodeData ===
// ======================

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
pub struct StreamNodeData<Out=()> {
    label         : Label,
    targets       : RefCell<Vec<StreamInput<Out>>>,
    value_cache   : RefCell<Out>,
    during_call   : Cell<bool>,
    watch_counter : WatchCounter,
}

impl<Out:Default> StreamNodeData<Out> {
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

impl<Out:Data> HasOutput for StreamNodeData<Out> {
    type Output = Out;
}

impl<Out:Data> EventEmitter for StreamNodeData<Out> {
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

    fn register_target(&self,target:StreamInput<Out>) {
        self.targets.borrow_mut().push(target)
    }

    fn register_watch(&self) -> WatchHandle {
        self.watch_counter.new_watch()
    }
}

impl<Out:Data> ValueProvider for StreamNodeData<Out> {
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
/// See the docs of `StreamNodeData` to learn more about its internal design.
#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Stream<Out=()> {
    data : Weak<StreamNodeData<Out>>,
}

/// A strong reference to FRP stream node. See the docs of `StreamNodeData` to learn more about its
/// internal design.
#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct StreamNode<Def:HasOutputStatic> {
    data       : Rc<StreamNodeData<Output<Def>>>,
    definition : Rc<Def>,
}

/// Weak reference to FRP stream node. See the docs of `StreamNodeData` to learn more about its
/// internal design.
#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct WeakStreamNode<Def:HasOutputStatic> {
    stream     : Stream<Output<Def>>,
    definition : Rc<Def>,
}


// === Output ===

impl<Out:Data>            HasOutput for Stream         <Out> { type Output = Out; }
impl<Def:HasOutputStatic> HasOutput for StreamNode     <Def> { type Output = Output<Def>; }
impl<Def:HasOutputStatic> HasOutput for WeakStreamNode <Def> { type Output = Output<Def>; }


// === Derefs ===

impl<Def> Deref for StreamNode<Def>
where Def:HasOutputStatic {
    type Target = Def;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}


// === Constructors ===

impl<Def:HasOutputStatic> StreamNode<Def> {
    /// Constructor.
    pub fn construct(label:Label, definition:Def) -> Self {
        let data       = Rc::new(StreamNodeData::new(label));
        let definition = Rc::new(definition);
        Self {data,definition}
    }

    /// Constructor which registers the newly created node as the event target of the argument.
    pub fn construct_and_connect<S>(label:Label, stream:&S, definition:Def) -> Self
    where S:StreamOutput, Self:EventConsumer<Output<S>> {
        let this = Self::construct(label,definition);
        let weak = this.downgrade();
        stream.register_target(weak.into());
        this
    }

    /// Downgrades to the weak version.
    pub fn downgrade(&self) -> WeakStreamNode<Def> {
        let stream     = Stream {data:Rc::downgrade(&self.data)};
        let definition = self.definition.clone_ref();
        WeakStreamNode {stream,definition}
    }
}

impl<T:HasOutputStatic> WeakStreamNode<T> {
    /// Upgrades to the strong version.
    pub fn upgrade(&self) -> Option<StreamNode<T>> {
        self.stream.data.upgrade().map(|data| {
            let definition = self.definition.clone_ref();
            StreamNode{data,definition}
        })
    }
}

impl<Def> From<StreamNode<Def>> for Stream<Def::Output>
where Def:HasOutputStatic {
    fn from(node:StreamNode<Def>) -> Self {
        let data = Rc::downgrade(&node.data);
        Stream {data}
    }
}


// === EventEmitter ===

impl<Out:Data> EventEmitter for Stream<Out> {
    fn emit_event(&self, value:&Self::Output) {
        self.data.upgrade().for_each(|t| t.emit_event(value))
    }

    fn register_target(&self,target:StreamInput<Output<Self>>) {
        self.data.upgrade().for_each(|t| t.register_target(target))
    }

    fn register_watch(&self) -> WatchHandle {
        self.data.upgrade().map(|t| t.register_watch()).unwrap() // FIXME
    }
}

impl<Def:HasOutputStatic> EventEmitter for StreamNode<Def>  {
    fn emit_event      (&self, value:&Output<Def>)           { self.data.emit_event(value) }
    fn register_target (&self,tgt:StreamInput<Output<Self>>) { self.data.register_target(tgt) }
    fn register_watch  (&self) -> WatchHandle                { self.data.register_watch() }
}

impl<Def:HasOutputStatic> EventEmitter for WeakStreamNode<Def> {
    fn emit_event      (&self, value:&Output<Def>)           { self.stream.emit_event(value) }
    fn register_target (&self,tgt:StreamInput<Output<Self>>) { self.stream.register_target(tgt) }
    fn register_watch  (&self) -> WatchHandle                { self.stream.register_watch() }
}


// === WeakEventConsumer ===

impl<Def,T> WeakEventConsumer<T> for WeakStreamNode<Def>
    where Def:HasOutputStatic, StreamNode<Def>:EventConsumer<T> {
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

impl<Def:HasOutputStatic> ValueProvider for StreamNode<Def> {
    fn value(&self) -> Self::Output {
        self.data.value_cache.borrow().clone()
    }
}

impl<Def:HasOutputStatic> ValueProvider for WeakStreamNode<Def> {
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

impl<Def:HasOutputStatic> HasId for StreamNode<Def> {
    fn id(&self) -> Id {
        self.downgrade().id()
    }
}

impl<Def:HasOutputStatic> HasId for WeakStreamNode<Def> {
    fn id(&self) -> Id {
        self.stream.id()
    }
}


// === HasLabel ===

impl<Def:HasOutputStatic> HasLabel for StreamNode<Def>
    where Def:InputBehaviors {
    fn label(&self) -> Label {
        self.data.label
    }
}

// FIXME code quality below:
impl<Def> HasOutputTypeLabel for StreamNode<Def>
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

impl<Def:HasOutputStatic> InputBehaviors for StreamNode<Def>
where Def:InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![] // FIXME
//        self.data.input_behaviors()
    }
}

impl<Def:HasOutputStatic> InputBehaviors for WeakStreamNode<Def>
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








