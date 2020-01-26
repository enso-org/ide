#![allow(missing_docs)]


use crate::prelude::*;


shared! { Switch
pub struct SwitchData {
    prev: Vec<Switch>
}}


#[derive(Clone,Copy,Debug)]
pub struct Event<T>(T);

#[derive(Clone,Debug,Default)]
pub struct Behavior<T> {
    rc: Rc<RefCell<T>>
}

impl<T> Behavior<T> {
    pub fn new(t:T) -> Self {
        let rc = Rc::new(RefCell::new(t));
        Self {rc}
    }
}



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



// ============
// === Node ===
// ============

//type_property! {Input}
//type_property! {Output}
type_property! {Value : Debug}


pub trait KnownInput {
    type Input:Debug+KnownValue;
}
pub type InputOf<T> = <T as KnownInput>::Input;

pub trait KnownOutput {
    type Output:Debug+KnownValue;
}
pub type OutputOf<T> = <T as KnownOutput>::Output;


pub trait NodeOps {
//    fn send_event()
}

pub trait OutNodeOps: KnownOutput + NodeOps {
    fn add_target(&self, target:InEventNode<OutputOf<Self>>);
}

alias! { Input  = Debug + KnownValue + 'static }
alias! { Output = Debug + KnownValue + 'static }



alias! { IsInOutNode   = KnownInput + KnownOutput + NodeOps }
alias! { IsOutNode     = KnownOutput + OutNodeOps }
alias! { IsInNode      = KnownInput + NodeOps }
alias! { IsInEventNode = KnownInput + EventNodeOps }



impl KnownValue for () {
    type Value = ();
}


impl<T:Debug> KnownValue for Event<T> {
    type Value = T;
}

impl<T:Debug> KnownValue for Behavior<T> {
    type Value = T;
}




pub trait EventNodeOps: KnownInput + NodeOps {
    fn handle_event(&self, input:&Self::Input);
}


#[derive(Shrinkwrap)]
pub struct OutNode<Out> {
    raw: Rc<dyn IsOutNode<Output=Out>>,
}

impl<Out:Output> OutNode<Out> {
    pub fn new<A:IsOutNode<Output=Out>+'static>(a:A) -> Self {
        let raw = Rc::new(a);
        Self {raw}
    }

    pub fn clone_ref(&self) -> Self {
        let raw = self.raw.clone();
        Self {raw}
    }
}

impl<A:IsOutNode<Output=Out>+CloneRef+'static,Out:Output> From<&A> for OutNode<Out> {
    fn from(a:&A) -> Self {
        Self::new(a.clone_ref())
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

impl<A:IsInEventNode<Input=In>+CloneRef+'static,In:Input> From<&A> for InEventNode<In> {
    fn from(a:&A) -> Self {
        Self::new(a.clone_ref())
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



#[derive(Shrinkwrap)]
pub struct AnyNode {
    raw: Rc<dyn NodeOps>,
}






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

impl<Out:Output+KnownSourceStorage> NodeOps for Source<Out> {}

impl<Out:Output+KnownSourceStorage> Source<Out> {
    pub fn new() -> Self {
        let shape   = SourceData::new();
        let targets = default();
        Self::construct(shape,targets)
    }
}



// ==============
// === Lambda ===
// ==============

pub type Lambda<In,Out> = Node<LambdaShape<In,Out>>;

pub struct LambdaShape<In,Out> {
    source : OutNode<In>,
    func   : Rc<dyn Fn(&In) -> Out>,
}

impl<In:Input,Out:Output> LambdaShape<In,Out> {
    pub fn new<F:'static + Fn(&In) -> Out, Source:Into<OutNode<In>>>
    (source:Source, f:F) -> Self {
        let source = source.into();
        let func   = Rc::new(f);
        Self {source,func}
    }
}


impl<In:Input,Out:Output> KnownInput  for LambdaShape<In,Out> { type Input  = In;  }
impl<In:Input,Out:Output> KnownOutput for LambdaShape<In,Out> { type Output = Out; }

impl<In:Input,Out:Output> NodeOps for Lambda<In,Out> {}

impl<In:Input,Out:Output> Lambda<In,Out> {
    pub fn new<F:'static + Fn(&In) -> Out, Source:Into<OutNode<In>>>
    (source:Source, f:F) -> Self {
        let source     = source.into();
        let source_ref = source.clone_ref();
        let shape      = LambdaShape::new(source,f);
        let targets    = default();
        let this       = Self::construct(shape,targets);
        source_ref.add_target((&this).into());
        this
    }
}

impl<In:Input,Out:Output> EventNodeOps for Lambda<In,Out> {
    fn handle_event(&self, input:&Self::Input) {
        println!("GOT {:?}",input)
    }
}



//// ===============
//// === Lambda2 ===
//// ===============
//
//shared! { Lambda2
//
//pub struct LambdaData2<A,B,T> {
//    source1 : Node<A>,
//    source2 : Node<B>,
//    func    : Rc<dyn Fn(&A,&B) -> T>,
//}
//
//impl<A,B,T> {
//    pub fn new<F:'static + Fn(&A,&B) -> T, Source1:Into<Node<A>>, Source2:Into<Node<B>>>
//    (source1:Source1, source2:Source2, f:F) -> Self {
//        let source1 = source1.into();
//        let source2 = source2.into();
//        let func    = Rc::new(f);
//        Self {source1,source2,func}
//    }
//}}
//
//
//impl<A,B,T> KnownOutput for Lambda2<A,B,T> {
//    type Output = T;
//}
//
//impl<A,B,T> NodeOps for Lambda2<A,B,T> {}


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
OutNodeOps for Node<Shape>
where Node<Shape>:NodeOps, OutputOf<Self>:'static {
    fn add_target(&self, target:InEventNode<OutputOf<Self>>) {
        self.rc.borrow_mut().targets.push(target);
    }
}


impl<Shape:KnownInput,Out> KnownInput for NodeTemplate<Shape,Out> {
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



pub fn test () {
    println!("\n\n\n--- FRP ---\n");


    let mouse_position : Source<Behavior<Position>> = Source::new();

    let e1: Source<Event<i32>> = Source::new();
//
    let n1: Lambda<Event<i32>,Event<i32>> = Lambda::new(&e1, |Event(i)| { Event(i+1) });
//    let n2 = Lambda::new(&e1, |i| {i+1});

//    let n3 = Lambda2::new(&n1,&n2,|i,j| {i * j});


    e1.emit_event(&Event(7));

}

