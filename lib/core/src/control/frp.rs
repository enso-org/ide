#![allow(missing_docs)]


use crate::prelude::*;


shared! { Switch
pub struct SwitchData {
    prev: Vec<Switch>
}}


macro_rules! alias {
    ($name:ident = $($tok:tt)*) => {
        pub trait $name: $($tok)* {}
        impl<T:$($tok)*> $name for T {}
    }
}


macro_rules! type_property {
    ($name:ident $(:$($tok:tt)*)?) => { paste::item! {
        pub trait [<Known $name>] {
            type $name $(:$($tok)*)?;
        }

        pub type [<$name Of>]<T> = <T as [<Known $name>]>::$name;
    }}
}



// ===============
// === Wrapper ===
// ===============

/// Trait for objects which wrap values.
///
/// Please note that this implements safe wrappers, so the object - value relation must be
/// bijective.
pub trait Wrapper {

    /// The wrapped value type.
    type Content;

    /// Wraps the value and returns the wrapped type.
    fn wrap(t:Self::Content) -> Self;

    /// Unwraps this type to get the inner value.
    fn unwrap(&self) -> &Self::Content;
}

/// Accessor for the wrapped value.
pub type Unwrap<T> = <T as Wrapper>::Content;

/// Wraps the value and returns the wrapped type.
pub fn wrap<T:Wrapper>(t:T::Content) -> T {
    T::wrap(t)
}

/// Unwraps this type to get the inner value.
pub fn unwrap<T:Wrapper>(t:&T) -> &T::Content {
    T::unwrap(t)
}



// ========================
// === Event & Behavior ===
// ========================

// === Definition ===

pub type Event    <T> = OutNode<EventMessage<T>>;
pub type Behavior <T> = OutNode<BehaviorMessage<T>>;

#[derive(Clone,Copy,Debug,Default)]
pub struct EventMessage<T>(T);

#[derive(Clone,Copy,Debug,Default)]
pub struct BehaviorMessage<T>(T);


// === API ===

impl<T:Clone> EventMessage<T> {
    pub fn value(&self) -> T {
        self.unwrap().clone()
    }
}

impl<T:Clone> BehaviorMessage<T> {
    pub fn value(&self) -> T {
        self.unwrap().clone()
    }
}


// === Wrappers ===

impl Wrapper for () {
    type Content = ();
    fn wrap   (t:())  -> Self {}
    fn unwrap (&self) -> &()  { self }
}

impl<T> Wrapper for EventMessage<T> {
    type Content = T;
    fn wrap   (t:T)   -> Self { EventMessage(t) }
    fn unwrap (&self) -> &T   { &self.0 }
}

impl<T> Wrapper for BehaviorMessage<T> {
    type Content = T;
    fn wrap   (t:T)   -> Self { BehaviorMessage(t) }
    fn unwrap (&self) -> &T   { &self.0 }
}



// ============
// === Node ===
// ============

/// Alias to `Wrapper` with the inner type being `Debug`.
pub trait DebugWrapper = Wrapper where Unwrap<Self>:Debug;

pub type Value<T> = Unwrap<T>;




/// Event input associated type. Please note that FRP nodes can have maximum one event input.
/// In such a case this trait points to it.
pub trait KnownEventInput {
    type EventInput : Message;
}

/// Event input accessor.
pub type EventInput<T> = <T as KnownEventInput>::EventInput;


/// Each FRP node has a single node, which type is described by this trait.
pub trait KnownOutput {
    type Output : Message;
}

/// Node output accessor.
pub type Output<T> = <T as KnownOutput>::Output;



pub trait OutEventNodeOps: KnownOutput {
    fn add_event_target(&self, target:AnyEventConsumer<Output<Self>>);
}

pub trait OutBehaviorNodeOps: KnownOutput {
    fn current_value(&self) -> Value<Output<Self>>;
}


alias! { MessageValue = Clone + Debug + Default + 'static }
alias! { Message      = MessageValue + DebugWrapper + KnownOutNodeStorage }

alias! { IsOutEventNode    = KnownOutput + OutEventNodeOps + Debug }
alias! { IsOutBehaviorNode = KnownOutput + OutBehaviorNodeOps + Debug }
alias! { IsInNode          = KnownEventInput }











type_property! {OutNodeStorage:Clone+Debug}

impl KnownOutNodeStorage for () {
    type OutNodeStorage = ();
}

impl<Out> KnownOutNodeStorage for EventMessage<Out> {
    type OutNodeStorage = Rc<dyn IsOutEventNode<Output=EventMessage<Out>>>;
}


impl<Out> KnownOutNodeStorage for BehaviorMessage<Out> {
    type OutNodeStorage = Rc<dyn IsOutBehaviorNode<Output=BehaviorMessage<Out>>>;
}


impl<Out:Message> KnownOutput for OutNode<Out> { type Output = Out; }

#[derive(Debug)]
pub struct OutNode<Out:KnownOutNodeStorage> {
    storage: OutNodeStorageOf<Out>,
}

impl<Out:Message> OutNode<Out> {
    pub fn new(storage:OutNodeStorageOf<Out>) -> Self {
        Self {storage}
    }

    pub fn clone_ref(&self) -> Self {
        let storage = self.storage.clone();
        Self {storage}
    }
}

impl<Out:KnownOutNodeStorage> Deref for OutNode<Out> {
    type Target = OutNodeStorageOf<Out>;
    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

//impl<A:IsOutEventNode<Output=Out>+CloneRef+'static,Out:Message> From<&A> for OutNode<Out> {
//    fn from(a:&A) -> Self {
//        Self::new(a.clone_ref())
//    }
//}

impl<A:IsOutBehaviorNode<Output=BehaviorMessage<Out>>+CloneRef+'static,Out:MessageValue> From<&A> for OutNode<BehaviorMessage<Out>> {
    fn from(a:&A) -> Self {
        Self::new(Rc::new(a.clone_ref()))
    }
}


impl<A:IsOutEventNode<Output=EventMessage<Out>>+CloneRef+'static,Out:MessageValue> From<&A> for OutNode<EventMessage<Out>> {
    fn from(a:&A) -> Self {
        Self::new(Rc::new(a.clone_ref()))
    }
}


impl<Out:KnownOutNodeStorage+Message> From<&OutNode<Out>> for OutNode<Out> {
    fn from(t:&OutNode<Out>) -> Self {
        t.clone_ref()
    }
}



// =====================
// === EventConsumer ===
// =====================

pub trait EventConsumer: KnownEventInput + Debug {
    fn on_event(&self, input:&Self::EventInput);
}

#[derive(Clone,Debug,Shrinkwrap)]
pub struct AnyEventConsumer<In> {
    raw: Rc<dyn EventConsumer<EventInput=In>>,
}

impl<In:Message> AnyEventConsumer<In> {
    pub fn new<A:EventConsumer<EventInput=In>+'static>(a:A) -> Self {
        let raw = Rc::new(a);
        Self {raw}
    }
}

impl<T,In> From<&T> for AnyEventConsumer<In>
where T  : EventConsumer<EventInput=In> + Clone + 'static,
      In : Message {
    fn from(t:&T) -> Self {
        Self::new(t.clone())
    }
}



// ===============
// === Source ===
// ===============

type Source<Out> = Node<SourceData<Out>>;

type_property! {SourceStorage:Default}

impl<T> KnownSourceStorage for EventMessage<T> {
    type SourceStorage = ();
}

impl<T:Default> KnownSourceStorage for BehaviorMessage<T> {
    type SourceStorage = BehaviorMessage<T>;
}



#[derive(Derivative)]
#[derivative(Default (bound="SourceStorageOf<Out>:Default"))]
#[derivative(Debug   (bound="SourceStorageOf<Out>:Debug"))]
pub struct SourceData<Out:KnownSourceStorage> {
    storage: SourceStorageOf<Out>
}

impl<Out:KnownSourceStorage> SourceData<Out>{
    pub fn new() -> Self {
        default()
    }
}

impl<Out:Message+KnownSourceStorage> KnownEventInput  for SourceData<Out> { type EventInput  = ();  }
impl<Out:Message+KnownSourceStorage> KnownOutput for SourceData<Out> { type Output = Out; }

impl<Out:Message+KnownSourceStorage> Source<Out> {
    pub fn new() -> Self {
        let shape   = SourceData::new();
        let targets = default();
        Self::construct(shape,targets)
    }
}

impl<Out:MessageValue> OutBehaviorNodeOps for SourceData<BehaviorMessage<Out>> {
    fn current_value(&self) -> Out {
        self.storage.value()
    }
}




pub trait Infer<T> {
    type Result;
}

impl<X,T> Infer<EventMessage<T>> for X {
    type Result = EventMessage<X>;
}

impl<X,T> Infer<BehaviorMessage<T>> for X {
    type Result = BehaviorMessage<X>;
}

pub type Inferred<T,X> = <X as Infer<T>>::Result;



impl<X,T1,T2> Infer <(EventMessage<T1>, EventMessage<T2> )> for X { type Result = EventMessage<X>; }
impl<X,T1,T2> Infer <(BehaviorMessage<T1>, EventMessage<T2> )> for X { type Result = EventMessage<X>; }
impl<X,T1,T2> Infer <(EventMessage<T1>, BehaviorMessage<T2> )> for X { type Result = EventMessage<X>; }
impl<X,T1,T2> Infer <(BehaviorMessage<T1>, BehaviorMessage<T2> )> for X { type Result = BehaviorMessage<X>; }






// ==============
// === Lambda ===
// ==============

pub type Lambda<In,Out> = Node<LambdaShape<In,Out>>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct LambdaShape<In:Message,Out:Message> {
    source : OutNode<In>,
    #[derivative(Debug="ignore")]
    func   : Rc<dyn Fn(&Value<In>) -> Out>,
}

impl<In:Message,Out:Message> LambdaShape<In,Out> {
    pub fn new<F:'static + Fn(&Value<In>) -> Value<Out>, Source:Into<OutNode<In>>>
    (source:Source, f:F) -> Self {
        let source = source.into();
        let func   = Rc::new(move |t:&Value<In>| {wrap(f(t))});
        Self {source,func}
    }
}











impl<In:Message,Out:Message> KnownEventInput for LambdaShape<In,Out> { type EventInput  = In;  }
impl<In:Message,Out:Message> KnownOutput     for LambdaShape<In,Out> { type Output = Out; }



pub trait LambdaNew<Source,Func> {
    fn new(source:Source,f:Func) -> Self;
}


impl<In:Message,X:Infer<In>,Func:'static + Fn(&Value<In>) -> X, Source:Into<OutNode<In>>>
LambdaNew<Source,Func> for Lambda<In,Inferred<In,X>>
where OutNode<In>:AddTarget<Self>, Inferred<In,X>:Message<Content=X> {
    fn new (source:Source, f:Func) -> Self {
        let source     = source.into();
        let source_ref = source.clone_ref();
        let shape      = LambdaShape::new(source,f);
        let targets    = default();
        let this       = Self::construct(shape,targets);
        source_ref.add_target(&this);
        this
    }
}





pub trait AddTarget<T> {
    fn add_target(&self,t:&T);
}

impl<S,T> AddTarget<S> for OutNode<EventMessage<T>>
where for<'t> &'t S : Into<AnyEventConsumer<EventMessage<T>>> {
    fn add_target(&self,t:&S) {
        self.add_event_target(t.into())
    }
}

impl<S,T> AddTarget<S> for OutNode<BehaviorMessage<T>> {
    fn add_target(&self,t:&S) {}
}





impl<In:Message,Out:Message> EventConsumer for Lambda<In,Out> {
    fn on_event(&self, input:&Self::EventInput) {
        println!("GOT {:?}",input);
        let output = (self.rc.borrow().shape.func)(unwrap(input));
        self.emit_event(&output);
    }
}




// ==============
// === Lambda2 ===
// ==============

pub type Lambda2<In1,In2,Out> = Node<Lambda2Shape<In1,In2,Out>>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Lambda2Shape<In1:Message,In2:Message,Out:Message> {
    source1 : OutNode<In1>,
    source2 : OutNode<In2>,
    #[derivative(Debug="ignore")]
    func    : Rc<dyn Fn(&Value<In1>,&Value<In2>) -> Out>,
}

impl<In1:Message,In2:Message,Out:Message>
Lambda2Shape<In1,In2,Out> {
    pub fn new
    < F:'static + Fn(&Value<In1>,&Value<In2>) -> Value<Out>
    , Source1:Into<OutNode<In1>>
    , Source2:Into<OutNode<In2>>
    >
    (source1:Source1, source2:Source2, f:F) -> Self {
        let source1 = source1.into();
        let source2 = source2.into();
        let func    = Rc::new(move |a:&Value<In1>,b:&Value<In2>| { wrap(f(a,b)) });
        Self {source1,source2,func}
    }
}

impl<In1:MessageValue,In2:MessageValue,Out:Message> KnownEventInput for Lambda2Shape<EventMessage<In1>, BehaviorMessage<In2>,Out> { type EventInput  = EventMessage<In1>;  }
impl<In1:Message,In2:Message,Out:Message> KnownOutput for Lambda2Shape<In1,In2,Out> { type Output = Out; }




pub trait Lambda2New<Source1,Source2,Function> {
    fn new(source:Source1, source2:Source2,f:Function) -> Self;
}


impl<In1:Message,In2:Message,X:Infer<(In1,In2)>,Source1,Source2,Function>
Lambda2New<Source1,Source2,Function> for Lambda2<In1,In2,Inferred<(In1,In2),X>> where
    Inferred<(In1,In2),X> : Message<Content=X>,
    Function : 'static + Fn(&Value<In1>,&Value<In2>) -> X,
    Source1  : Into<OutNode<In1>>,
    Source2  : Into<OutNode<In2>>,
    OutNode<In1>:AddTarget<Self>,
    OutNode<In2>:AddTarget<Self>,
{
    fn new (source1:Source1, source2:Source2, f:Function) -> Self {
        let source1     = source1.into();
        let source2     = source2.into();
        let source1_ref = source1.clone_ref();
        let source2_ref = source2.clone_ref();
        let shape       = Lambda2Shape::new(source1,source2,f);
        let targets     = default();
        let this        = Self::construct(shape,targets);
        source1_ref.add_target(&this);
        source2_ref.add_target(&this);
        this
    }
}

impl<In1:MessageValue,In2:MessageValue,Out:Message> EventConsumer for Lambda2<EventMessage<In1>, BehaviorMessage<In2>,Out> {
    fn on_event(&self, input:&Self::EventInput) {
        println!("GOT {:?}",input);
        let value2 = self.rc.borrow().shape.source2.current_value();
        let output = (self.rc.borrow().shape.func)(&input.0,&value2);
        self.emit_event(&output);
    }
}


// ============
// === Node ===
// ============

#[derive(Debug)]
pub struct NodeTemplateData<Shape,Out> {
    shape   : Shape,
    targets : Vec<AnyEventConsumer<Out>>,
}

impl<Shape,Out> NodeTemplateData<Shape,Out> {
    pub fn construct(shape:Shape, targets:Vec<AnyEventConsumer<Out>>) -> Self {
        Self {shape,targets}
    }
}

#[derive(Debug)]
pub struct NodeTemplate<Shape,Out> {
    rc: Rc<RefCell<NodeTemplateData<Shape,Out>>>,
}

pub type Node<Shape> = NodeTemplate<Shape,Output<Shape>>;

impl<Shape:KnownOutput> Node<Shape> {
    pub fn construct(shape:Shape, targets:Vec<AnyEventConsumer<Output<Shape>>>) -> Self {
        let data = NodeTemplateData::construct(shape,targets);
        let rc   = Rc::new(RefCell::new(data));
        Self {rc}
    }
}

impl<Shape:KnownOutput> Node<Shape> {
    pub fn emit_event(&self, event:&Output<Shape>) {
        self.rc.borrow().targets.iter().for_each(|target| {
            target.on_event(event)
        })
    }
}



impl<Shape:KnownOutput>
OutEventNodeOps for Node<Shape>
where Output<Self>:'static, Output<Shape>:Message {
    fn add_event_target(&self, target:AnyEventConsumer<Output<Self>>) {
        self.rc.borrow_mut().targets.push(target);
    }
}


impl<Shape:OutBehaviorNodeOps>
OutBehaviorNodeOps for Node<Shape>
where Output<Shape>:Message {
    fn current_value(&self) -> Value<Output<Self>> {
        self.rc.borrow().shape.current_value()
    }
}



impl<Shape:KnownEventInput,Out> KnownEventInput for NodeTemplate<Shape,Out>
where <<Shape as KnownEventInput>::EventInput as Wrapper>::Content : Debug {
    type EventInput = EventInput<Shape>;
}

impl<Shape,Out:Message> KnownOutput for NodeTemplate<Shape,Out> {
    type Output = Out;
}

impl<Shape,Out> Clone for NodeTemplate<Shape,Out> {
    fn clone(&self) -> Self {
        let rc = self.rc.clone();
        Self {rc}
    }
}

impl<Shape,Out> CloneRef for NodeTemplate<Shape,Out> {}



//////////////////////////////////////////////////////


#[derive(Clone,Copy,Debug,Default)]
pub struct Position {
    x:i32,
    y:i32,
}

impl Position {
    pub fn new(x:i32, y:i32) -> Self {
        Self {x,y}
    }
}




//
//pub fn trace<Source:Into<OutNode<T>>,T:Input+Output>
//(source:Source) -> Lambda<T,T> {
//    Lambda::new(source, |t| {t.clone()})
//}

pub fn test () {
    println!("\n\n\n--- FRP ---\n");


    let mouse_position = Source::<BehaviorMessage<Position>>::new();

    let e1 = Source::<EventMessage<i32>>::new();
//
    let n1  = Lambda::new(&e1, |i| { i+1 });
    let nn1: Event<i32> = (&n1).into();
    let n2 = Lambda::new(&nn1, |i| { i*2 });

    let n3: Lambda<BehaviorMessage<Position>, BehaviorMessage<Position>> = Lambda::new(&mouse_position, |t| { t.clone() });


    let n3 = Lambda2::new(&n1,&mouse_position, |e,b| { e.clone() });

//    let n3 = Lambda2::new(&n1,&n2,|i,j| {i * j});


    e1.emit_event(&EventMessage(7));

}

