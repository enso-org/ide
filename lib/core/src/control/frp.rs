#![allow(missing_docs)]


use crate::prelude::*;


shared! { Switch
pub struct SwitchData {
    prev: Vec<Switch>
}}


#[derive(Clone,Copy,Debug)]
pub struct Event<T>(T);


pub struct Behavior<T> {
    phantom: PhantomData<T>,
}



macro_rules! alias {
    ($name:ident = $($tok:tt)*) => {
        pub trait $name: $($tok)* {}
        impl<T:$($tok)*> $name for T {}
    }
}


macro_rules! type_property {
    ($name:ident) => { paste::item! {
        pub trait [<Known $name>] {
            type $name:Debug;
        }

        pub type [<$name Of>]<T> = <T as [<Known $name>]>::$name;
    }}
}



// ============
// === Node ===
// ============

//type_property! {Input}
//type_property! {Output}
type_property! {Value}


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
alias! { IsInEventNode2 = KnownInput + EventNodeOps + Clone }



impl KnownValue for () {
    type Value = ();
}


impl<T:Debug> KnownValue for Event<T> {
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






// ====================
// === Emitter ===
// ====================

shared! { EmitterShape

pub struct EmitterData<Out> {
    callbacks: Vec<Rc<dyn Fn(Out)>>
}

impl<Out> {
    pub fn new() -> Self {
        let callbacks = default();
        Self {callbacks}
    }
}}

impl<Out:Output> KnownOutput for EmitterShape<Out> {
    type Output = Out;
}

impl<Out> KnownInput for EmitterShape<Out> {
    type Input = ();
}

impl<Out:Output> NodeOps for Emitter<Out> {}



type Emitter<Out> = NodeTemplate<EmitterShape<Out>>;


impl<Out:Output> Emitter<Out> {
    pub fn new() -> Self {
        let shape   = EmitterShape::new();
        let targets = default();
        Self::construct(shape,targets)
    }
}



// ===============
// === Map ===
// ===============

shared! { MapShape

pub struct MapShapeData<In,Out> {
    source : OutNode<In>,
    func   : Rc<dyn Fn(&In) -> Out>,
}

impl<In,Out> {
    pub fn new<F:'static + Fn(&In) -> Out, Source:Into<OutNode<In>>>
    (source:Source, f:F) -> Self {
        let source = source.into();
        let func   = Rc::new(f);
        Self {source,func}
    }
}}


impl<In:Input,Out:Output> KnownInput for MapShape<In,Out> {
    type Input = In;
}

impl<In:Input,Out:Output> KnownOutput for MapShape<In,Out> {
    type Output = Out;
}

impl<In:Input,Out:Output> NodeOps for Map<In,Out> {}



type Map<In,Out> = NodeTemplate<MapShape<In,Out>>;


impl<In:Input,Out:Output> Map<In,Out> {
    pub fn new<F:'static + Fn(&In) -> Out, Source:Into<OutNode<In>>>
    (source:Source, f:F) -> Self {
        let source  = source.into();
        let source_ref = source.clone_ref();
        let shape   = MapShape::new(source,f);
        let targets = default();
        let this = Self::construct(shape,targets);
        let foo: InEventNode<In> = (&this).into();
//        ttt(this.clone_ref());
//        let bar: impl IsInEventNode2 = this.clone_ref();
        source_ref.add_target(foo);
        this
    }
}

impl<In:Input,Out:Output> EventNodeOps for Map<In,Out> {
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

pub struct NodeTemplateX<Shape,Out> {
    rc: Rc<RefCell<NodeTemplateData<Shape,Out>>>,
}

pub type NodeTemplate<Shape> = NodeTemplateX<Shape,OutputOf<Shape>>;

impl<Shape:KnownOutput> NodeTemplate<Shape> {
    pub fn construct(shape:Shape, targets:Vec<InEventNode<OutputOf<Shape>>>) -> Self {
        let data = NodeTemplateData::construct(shape,targets);
        let rc   = Rc::new(RefCell::new(data));
        Self {rc}
    }
}

impl<Shape:KnownOutput> NodeTemplate<Shape> {
    pub fn emit_event(&self, event:&OutputOf<Shape>) {
        self.rc.borrow().targets.iter().for_each(|target| {
            target.handle_event(event)
        })
    }
}


impl<Shape:KnownOutput> OutNodeOps for NodeTemplate<Shape>
where NodeTemplate<Shape>:NodeOps, OutputOf<Self>:'static {
    fn add_target(&self, target:InEventNode<OutputOf<Self>>) {
        self.rc.borrow_mut().targets.push(target);
    }
}


impl<Shape:KnownInput,Out> KnownInput for NodeTemplateX<Shape,Out> {
    type Input = InputOf<Shape>;
}

impl<Shape,Out:Output> KnownOutput for NodeTemplateX<Shape,Out> {
    type Output = Out;
}

impl<Shape,Out> Clone for NodeTemplateX<Shape,Out> {
    fn clone(&self) -> Self {
        let rc = self.rc.clone();
        Self {rc}
    }
}

impl<Shape,Out> CloneRef for NodeTemplateX<Shape,Out> {}


//
//#[derive(Clone)]
//pub struct NodeTemplate<T:KnownOutput> {
//    pub shape   : T,
//    pub targets : Rc<RefCell<Vec<InEventNode<  OutputOf<T>  >>>>
//}
//
//impl<T:KnownInput+KnownOutput> KnownInput for NodeTemplate<T> {
//    type Input = <T as KnownInput>::Input;
//}
//
//impl<T:KnownOutput> KnownOutput for NodeTemplate<T> {
//    type Output = <T as KnownOutput>::Output;
//}
//
//
//impl<T:CloneRef+KnownOutput> CloneRef for NodeTemplate<T> {
//    fn clone_ref(&self) -> Self {
//        let shape   = self.shape.clone_ref();
//        let targets = self.targets.clone();
//        Self {shape,targets}
//    }
//}
//
//
//impl<T:KnownOutput> NodeTemplate<T> {
//    pub fn emit_event(&self, event:&OutputOf<T>) {
//        self.targets.borrow().iter().for_each(|target| {
//            target.handle_event(event)
//        })
//    }
//}
//
//
//impl<T:KnownOutput> OutNodeOps for NodeTemplate<T>
//where NodeTemplate<T>:NodeOps {
//    fn add_target(&self, target:InEventNode<OutputOf<Self>>) {
//        self.targets.borrow_mut().push(target);
//    }
//}

//impl<T:KnownOutput> NodeOps for NodeTemplate<T> {
//
//}



//////////////////////////////////////////////////////

pub fn test () {
    println!("\n\n\n--- FRP ---\n");

    let e1: Emitter<Event<i32>> = Emitter::new();
//
    let n1: Map<Event<i32>,Event<i32>> = Map::new(&e1, |Event(i)| { Event(i+1) });
//    let n2 = Map::new(&e1, |i| {i+1});

//    let n3 = Lambda2::new(&n1,&n2,|i,j| {i * j});


    e1.emit_event(&Event(7));

}

