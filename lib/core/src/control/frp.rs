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


pub trait Wrapper {
    type Content;
    fn wrap   (t:Self::Content) -> Self;
    fn unwrap (&self)           -> &Self::Content;
}

pub type Unwrap<T> = <T as Wrapper>::Content;

pub fn wrap<T:Wrapper>(t:T::Content) -> T {
    T::wrap(t)
}

pub fn unwrap<T:Wrapper>(t:&T) -> &T::Content {
    T::unwrap(t)
}



// ========================
// === Event & Behavior ===
// ========================

// === Definition ===

#[derive(Clone,Copy,Debug,Default)]
pub struct Event<T>(T);

#[derive(Clone,Copy,Debug,Default)]
pub struct Behavior<T>(T);


// === API ===

impl<T:Clone> Event<T> {
    pub fn value(&self) -> T {
        self.unwrap().clone()
    }
}

impl<T:Clone> Behavior<T> {
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

impl<T> Wrapper for Event<T> {
    type Content = T;
    fn wrap   (t:T)   -> Self { Event(t) }
    fn unwrap (&self) -> &T   { &self.0 }
}

impl<T> Wrapper for Behavior<T> {
    type Content = T;
    fn wrap   (t:T)   -> Self { Behavior(t) }
    fn unwrap (&self) -> &T   { &self.0 }
}



// ============
// === Node ===
// ============

pub trait KnownValue = Wrapper where Unwrap<Self>:Debug;
pub type ValueOf<T> = Unwrap<T>;




pub trait KnownInput {
    type Input:Debug+KnownValue;
}
pub type InputOf<T> = <T as KnownInput>::Input;

pub trait KnownOutput {
    type Output:Debug+KnownValue;
}
pub type OutputOf<T> = <T as KnownOutput>::Output;



pub trait OutEventNodeOps: KnownOutput {
    fn add_event_target(&self, target:InEventNode<OutputOf<Self>>);
}

pub trait OutBehaviorNodeOps: KnownOutput {
    fn current_value(&self) -> ValueOf<OutputOf<Self>>;
}


alias! { InputData = Clone + Debug + Default + 'static }


alias! { Input  = KnownValue + KnownOutNodeStorage + InputData }
alias! { Output = Debug + KnownValue + KnownOutNodeStorage + 'static }



alias! { IsInOutNode   = KnownInput + KnownOutput }
alias! { IsOutEventNode     = KnownOutput + OutEventNodeOps }
alias! { IsOutBehaviorNode     = KnownOutput + OutBehaviorNodeOps }
alias! { IsInNode      = KnownInput }
alias! { IsInEventNode = KnownInput + EventNodeOps }






pub trait EventNodeOps: KnownInput {
    fn handle_event(&self, input:&Self::Input);
}


type_property! {OutNodeStorage:Clone}

impl<Out> KnownOutNodeStorage for Event<Out> {
    type OutNodeStorage = Rc<dyn IsOutEventNode<Output=Event<Out>>>;
}


impl<Out> KnownOutNodeStorage for Behavior<Out> {
    type OutNodeStorage = Rc<dyn IsOutBehaviorNode<Output=Behavior<Out>>>;
}


impl<Out:Output> KnownOutput for OutNode<Out> { type Output = Out; }


pub struct OutNode<Out:KnownOutNodeStorage> {
    storage: OutNodeStorageOf<Out>,
}

impl<Out:Output> OutNode<Out> {
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

//impl<A:IsOutEventNode<Output=Out>+CloneRef+'static,Out:Output> From<&A> for OutNode<Out> {
//    fn from(a:&A) -> Self {
//        Self::new(a.clone_ref())
//    }
//}

impl<A:IsOutBehaviorNode<Output=Behavior<Out>>+CloneRef+'static,Out:InputData> From<&A> for OutNode<Behavior<Out>> {
    fn from(a:&A) -> Self {
        Self::new(Rc::new(a.clone_ref()))
    }
}


impl<A:IsOutEventNode<Output=Event<Out>>+CloneRef+'static,Out:InputData> From<&A> for OutNode<Event<Out>> {
    fn from(a:&A) -> Self {
        Self::new(Rc::new(a.clone_ref()))
    }
}


impl<Out:KnownOutNodeStorage+Output> From<&OutNode<Out>> for OutNode<Out> {
    fn from(t:&OutNode<Out>) -> Self {
        t.clone_ref()
    }
}





#[derive(Shrinkwrap)]
pub struct InEventNode<In> {
    raw: Rc<dyn IsInEventNode<Input=In>>,
}

impl<In:Input> InEventNode<In> {
    pub fn new<A:IsInEventNode<Input=In>+'static>(a:A) -> Self {
        let raw = Rc::new(a);
        Self {raw}
    }

    pub fn clone_ref(&self) -> Self {
        let raw = self.raw.clone();
        Self {raw}
    }
}




#[derive(Shrinkwrap)]
pub struct InOutNode<In,Out> {
    raw: Rc<dyn IsInOutNode<Input=In,Output=Out>>,
}

impl<In:Input,Out:Output> InOutNode<In,Out> {
    pub fn new<A:IsInOutNode<Input=In,Output=Out>+'static>(a:A) -> Self {
        let raw = Rc::new(a);
        Self {raw}
    }

    pub fn clone_ref(&self) -> Self {
        let raw = self.raw.clone();
        Self {raw}
    }
}

impl<A:IsInOutNode<Input=In,Output=Out>+CloneRef+'static,In:Input,Out:Output>
From<&A> for InOutNode<In,Out> {
    fn from(a:&A) -> Self {
        Self::new(a.clone_ref())
    }
}



//#[derive(Shrinkwrap)]
//pub struct AnyNode {
//    raw: Rc<dyn NodeOps>,
//}






// ===============
// === Source ===
// ===============

type Source<Out> = Node<SourceData<Out>>;

type_property! {SourceStorage:Default}

impl<T> KnownSourceStorage for Event<T> {
    type SourceStorage = ();
}

impl<T:Default> KnownSourceStorage for Behavior<T> {
    type SourceStorage = Behavior<T>;
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

impl<Out:Output+KnownSourceStorage> KnownInput  for SourceData<Out> { type Input  = ();  }
impl<Out:Output+KnownSourceStorage> KnownOutput for SourceData<Out> { type Output = Out; }

impl<Out:Output+KnownSourceStorage> Source<Out> {
    pub fn new() -> Self {
        let shape   = SourceData::new();
        let targets = default();
        Self::construct(shape,targets)
    }
}

impl<Out:InputData> OutBehaviorNodeOps for SourceData<Behavior<Out>> {
    fn current_value(&self) -> Out {
        self.storage.value()
    }
}




pub trait Infer<T> {
    type Result;
}

impl<X,T> Infer<Event<T>> for X {
    type Result = Event<X>;
}

impl<X,T> Infer<Behavior<T>> for X {
    type Result = Behavior<X>;
}

pub type Inferred<T,X> = <X as Infer<T>>::Result;



impl<X,T1,T2> Infer <( Event    <T1> , Event    <T2> )> for X { type Result = Event    <X>; }
impl<X,T1,T2> Infer <( Behavior <T1> , Event    <T2> )> for X { type Result = Event    <X>; }
impl<X,T1,T2> Infer <( Event    <T1> , Behavior <T2> )> for X { type Result = Event    <X>; }
impl<X,T1,T2> Infer <( Behavior <T1> , Behavior <T2> )> for X { type Result = Behavior <X>; }






// ==============
// === Lambda ===
// ==============

pub type Lambda<In,Out> = Node<LambdaShape<In,Out>>;

pub struct LambdaShape<In:Input,Out:Output> {
    source : OutNode<In>,
    func   : Rc<dyn Fn(&ValueOf<In>) -> Out>,
}

impl<In:Input,Out:Output> LambdaShape<In,Out> {
    pub fn new<F:'static + Fn(&ValueOf<In>) -> ValueOf<Out>, Source:Into<OutNode<In>>>
    (source:Source, f:F) -> Self {
        let source = source.into();
        let func   = Rc::new(move |t:&ValueOf<In>| {wrap(f(t))});
        Self {source,func}
    }
}











impl<In:Input,Out:Output> KnownInput  for LambdaShape<In,Out> { type Input  = In;  }
impl<In:Input,Out:Output> KnownOutput for LambdaShape<In,Out> { type Output = Out; }



pub trait LambdaNew<Source,Func> {
    fn new(source:Source,f:Func) -> Self;
}


impl<In:Input,X:InputData+Infer<In>,Func:'static + Fn(&ValueOf<In>) -> X, Source:Into<OutNode<In>>>
LambdaNew<Source,Func> for Lambda<In,Inferred<In,X>>
where OutNode<In>:AddTarget<Self>, Inferred<In,X>:Output<Content=X> {
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

impl<S,T> AddTarget<S> for OutNode<Event<T>>
where for<'t> &'t S : Into<InEventNode<Event<T>>> {
    fn add_target(&self,t:&S) {
        self.add_event_target(t.into())
    }
}

impl<S,T> AddTarget<S> for OutNode<Behavior<T>> {
    fn add_target(&self,t:&S) {}
}





impl<In:Input,Out:Output> EventNodeOps for Lambda<In,Out> {
    fn handle_event(&self, input:&Self::Input) {
        println!("GOT {:?}",input);
        let output = (self.rc.borrow().shape.func)(unwrap(input));
        self.emit_event(&output);
    }
}


impl<A:IsInEventNode<Input=In>+CloneRef+'static,In:Input> From<&A> for InEventNode<In> {
    fn from(a:&A) -> Self {
        Self::new(a.clone_ref())
    }
}

// ==============
// === Lambda2 ===
// ==============

pub type Lambda2<In1,In2,Out> = Node<Lambda2Shape<In1,In2,Out>>;

pub struct Lambda2Shape<In1:Input,In2:Input,Out:Output> {
    source1 : OutNode<In1>,
    source2 : OutNode<In2>,
    func    : Rc<dyn Fn(&ValueOf<In1>,&ValueOf<In2>) -> Out>,
}

impl<In1:Input,In2:Input,Out:Output>
Lambda2Shape<In1,In2,Out> {
    pub fn new
    < F:'static + Fn(&ValueOf<In1>,&ValueOf<In2>) -> ValueOf<Out>
    , Source1:Into<OutNode<In1>>
    , Source2:Into<OutNode<In2>>
    >
    (source1:Source1, source2:Source2, f:F) -> Self {
        let source1 = source1.into();
        let source2 = source2.into();
        let func    = Rc::new(move |a:&ValueOf<In1>,b:&ValueOf<In2>| { wrap(f(a,b)) });
        Self {source1,source2,func}
    }
}

impl<In1:InputData,In2:InputData,Out:Output> KnownInput for Lambda2Shape<Event<In1>,Behavior<In2>,Out> { type Input  = Event<In1>;  }
impl<In1:Input,In2:Input,Out:Output> KnownOutput for Lambda2Shape<In1,In2,Out> { type Output = Out; }




pub trait Lambda2New<Source1,Source2,Function> {
    fn new(source:Source1, source2:Source2,f:Function) -> Self;
}


impl<In1:Input,In2:Input,X:InputData+Infer<(In1,In2)>,Source1,Source2,Function>
Lambda2New<Source1,Source2,Function> for Lambda2<In1,In2,Inferred<(In1,In2),X>> where
    Inferred<(In1,In2),X> : Output<Content=X>,
    Function : 'static + Fn(&ValueOf<In1>,&ValueOf<In2>) -> X,
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

impl<In1:InputData,In2:InputData,Out:Output> EventNodeOps for Lambda2<Event<In1>,Behavior<In2>,Out> {
    fn handle_event(&self, input:&Self::Input) {
        println!("GOT {:?}",input);
        let value2 = self.rc.borrow().shape.source2.current_value();
        let output = (self.rc.borrow().shape.func)(&input.0,&value2);
        self.emit_event(&output);
    }
}


// ============
// === Node ===
// ============


pub struct NodeTemplateData<Shape,Out> {
    shape   : Shape,
    targets : Vec<InEventNode<Out>>,
}

impl<Shape,Out> NodeTemplateData<Shape,Out> {
    pub fn construct(shape:Shape, targets:Vec<InEventNode<Out>>) -> Self {
        Self {shape,targets}
    }
}

pub struct NodeTemplate<Shape,Out> {
    rc: Rc<RefCell<NodeTemplateData<Shape,Out>>>,
}

pub type Node<Shape> = NodeTemplate<Shape,OutputOf<Shape>>;

impl<Shape:KnownOutput> Node<Shape> {
    pub fn construct(shape:Shape, targets:Vec<InEventNode<OutputOf<Shape>>>) -> Self {
        let data = NodeTemplateData::construct(shape,targets);
        let rc   = Rc::new(RefCell::new(data));
        Self {rc}
    }
}

impl<Shape:KnownOutput> Node<Shape> {
    pub fn emit_event(&self, event:&OutputOf<Shape>) {
        self.rc.borrow().targets.iter().for_each(|target| {
            target.handle_event(event)
        })
    }
}



impl<Shape:KnownOutput>
OutEventNodeOps for Node<Shape>
where OutputOf<Self>:'static, OutputOf<Shape>:Output {
    fn add_event_target(&self, target:InEventNode<OutputOf<Self>>) {
        self.rc.borrow_mut().targets.push(target);
    }
}


impl<Shape:OutBehaviorNodeOps>
OutBehaviorNodeOps for Node<Shape>
where OutputOf<Shape>:Output {
    fn current_value(&self) -> ValueOf<OutputOf<Self>> {
        self.rc.borrow().shape.current_value()
    }
}



impl<Shape:KnownInput,Out> KnownInput for NodeTemplate<Shape,Out>
where <<Shape as KnownInput>::Input as Wrapper>::Content : Debug {
    type Input = InputOf<Shape>;
}

impl<Shape,Out:Output> KnownOutput for NodeTemplate<Shape,Out> {
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


    let mouse_position = Source::<Behavior<Position>>::new();

    let e1 = Source::<Event<i32>>::new();
//
    let n1  = Lambda::new(&e1, |i| { i+1 });
    let nn1: OutNode<Event<i32>> = (&n1).into();
    let n2 = Lambda::new(&nn1, |i| { i*2 });

    let n3: Lambda<Behavior<Position>,Behavior<Position>> = Lambda::new(&mouse_position, |t| { t.clone() });


    let n3 = Lambda2::new(&n1,&mouse_position, |e,b| { e.clone() });

//    let n3 = Lambda2::new(&n1,&n2,|i,j| {i * j});


    e1.emit_event(&Event(7));

}

