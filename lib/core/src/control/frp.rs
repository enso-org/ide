#![allow(missing_docs)]


use crate::prelude::*;


shared! { Switch
pub struct SwitchData {
    prev: Vec<Switch>
}}



pub struct Event<T> {
    phantom: PhantomData<T>,
}


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

type_property! {Input}
type_property! {Output}



pub trait NodeOps {
//    fn send_event()
}

pub trait OutNodeOps: KnownOutput + NodeOps {
    fn add_target(&self, target:InEventNode<Self::Output>);
}

alias! { Input  = Debug }
alias! { Output = Debug }



alias! { IsInOutNode   = KnownInput + KnownOutput + NodeOps }
alias! { IsOutNode     = KnownOutput + OutNodeOps }
alias! { IsInNode      = KnownInput + NodeOps }
alias! { IsInEventNode = KnownInput + EventNodeOps }





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

impl<A:IsOutNode<Output=Out>+CloneRef+'static,Out:Debug> From<&A> for OutNode<Out> {
    fn from(a:&A) -> Self {
        Self::new(a.clone_ref())
    }
}


#[derive(Shrinkwrap)]
pub struct InEventNode<In> {
    raw: Rc<dyn IsInEventNode<Input=In>>,
}

impl<In:Debug> InEventNode<In> {
    pub fn new<A:IsInEventNode<Input=In>+'static>(a:A) -> Self {
        let raw = Rc::new(a);
        Self {raw}
    }

    pub fn clone_ref(&self) -> Self {
        let raw = self.raw.clone();
        Self {raw}
    }
}

impl<A:IsInEventNode<Input=In>+CloneRef+'static,In:Debug> From<&A> for InEventNode<In> {
    fn from(a:&A) -> Self {
        Self::new(a.clone_ref())
    }
}




#[derive(Shrinkwrap)]
pub struct InOutNode<In,Out> {
    raw: Rc<dyn IsInOutNode<Input=In,Output=Out>>,
}

impl<In:Debug,Out:Debug> InOutNode<In,Out> {
    pub fn new<A:IsInOutNode<Input=In,Output=Out>+'static>(a:A) -> Self {
        let raw = Rc::new(a);
        Self {raw}
    }

    pub fn clone_ref(&self) -> Self {
        let raw = self.raw.clone();
        Self {raw}
    }
}

impl<A:IsInOutNode<Input=In,Output=Out>+CloneRef+'static,In:Debug,Out:Debug>
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
// === EventEmitter ===
// ====================

shared! { EventEmitterShape

pub struct EventEmitterData<T> {
    callbacks: Vec<Rc<dyn Fn(T)>>
}

impl<T> {
    pub fn new() -> Self {
        let callbacks = default();
        Self {callbacks}
    }
}}

impl<T:Debug> KnownOutput for EventEmitterShape<T> {
    type Output = T;
}

impl<T> KnownInput for EventEmitterShape<T> {
    type Input = ();
}

impl<T:Debug> NodeOps for EventEmitter<T> {}



type EventEmitter<T> = NodeTemplate<EventEmitterShape<T>>;


impl<T:Debug> EventEmitter<T> {
    pub fn new() -> Self {
        let shape   = EventEmitterShape::new();
        let targets = default();
        Self {shape,targets}
    }

    pub fn emit(&self, value:&T) {
        self.targets.borrow().iter().for_each(|target| {

        })
    }
}







// ===============
// === Map ===
// ===============

shared! { MapShape

pub struct MapShapeData<A,T> {
    source : OutNode<A>,
    func   : Rc<dyn Fn(&A) -> T>,
}

impl<A,T> {
    pub fn new<F:'static + Fn(&A) -> T, Source:Into<OutNode<A>>>
    (source:Source, f:F) -> Self {
        let source = source.into();
        let func   = Rc::new(f);
        Self {source,func}
    }
}}


impl<A:Debug,T> KnownInput for MapShape<A,T> {
    type Input = A;
}

impl<A,T:Debug> KnownOutput for MapShape<A,T> {
    type Output = T;
}

impl<A,T:Debug> NodeOps for Map<A,T> {}



type Map<A,T> = NodeTemplate<MapShape<A,T>>;


impl<A:Debug+'static,T:Debug+'static> Map<A,T> {
    pub fn new<F:'static + Fn(&A) -> T, Source:Into<OutNode<A>>>
    (source:Source, f:F) -> Self {
        let source  = source.into();
        let source_ref = source.clone_ref();
        let shape   = MapShape::new(source,f);
        let targets = default();
        let this = Self {shape,targets};
        let foo: InEventNode<A> = (&this).into();
        source_ref.add_target(foo);
        this
    }
}

impl<A:Debug,T:Debug> EventNodeOps for Map<A,T> {
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


#[derive(Clone)]
pub struct NodeTemplate<T:KnownOutput> {
    pub shape   : T,
    pub targets : Rc<RefCell<Vec<InEventNode<  <T as KnownOutput>::Output  >>>>
}

impl<T:KnownInput+KnownOutput> KnownInput for NodeTemplate<T> {
    type Input = <T as KnownInput>::Input;
}

impl<T:KnownOutput> KnownOutput for NodeTemplate<T> {
    type Output = <T as KnownOutput>::Output;
}


impl<T:CloneRef+KnownOutput> CloneRef for NodeTemplate<T> {
    fn clone_ref(&self) -> Self {
        let shape   = self.shape.clone_ref();
        let targets = self.targets.clone();
        Self {shape,targets}
    }
}


impl<T:KnownOutput> NodeTemplate<T> {
    pub fn emit_event(&self, event: &<T as KnownOutput>::Output) {
        self.targets.borrow().iter().for_each(|target| {
            target.handle_event(event)
        })
    }
}


impl<T:KnownOutput> OutNodeOps for NodeTemplate<T>
where NodeTemplate<T>:NodeOps {
    fn add_target(&self, target:InEventNode<Self::Output>) {
        self.targets.borrow_mut().push(target);
    }
}

//impl<T:KnownOutput> NodeOps for NodeTemplate<T> {
//
//}



//////////////////////////////////////////////////////

pub fn test () {
    println!("\n\n\n--- FRP ---\n");

    let e1 = EventEmitter::<i32>::new();

    let n1 = Map::new(&e1, |i| {i+1});
    let n2 = Map::new(&e1, |i| {i+1});

//    let n3 = Lambda2::new(&n1,&n2,|i,j| {i * j});


    e1.emit_event(&7);

}