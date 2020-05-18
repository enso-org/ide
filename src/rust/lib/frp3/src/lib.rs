#![feature(test)]
#![feature(trait_alias)]
#![feature(weak_into_raw)]


extern crate test;


use enso_prelude::*;

use std::os::raw::c_void;



// =============
// === AnyRc ===
// =============

#[derive(Clone,Debug)]
pub struct AnyRc {
    raw : *const(),
    rc  : Rc<dyn Any>,
}

impl AnyRc {
    pub fn new<T:'static>(t:T) -> Self {
        let rc   = Rc::new(t);
        let weak = Rc::downgrade(&rc);
        let raw  = weak.into_raw() as *const();
        AnyRc {raw,rc}
    }

    pub fn unsafe_cast<T>(&self) -> &T {
        unsafe { &*(self.raw as *const T) }
    }

    pub fn unsafe_cast_copy<T:Copy>(&self) -> T {
        unsafe { *(self.raw as *const T) }
    }
}


//////////////////////////////




pub struct Node<Def,Out> {
    network  : WeakNetwork,
    node_ref : TypedAnyNodeRef<Def>,
    out_type : PhantomData<Out>,
}

impl<Def,Out> Node<Def,Out> {
    fn new(network:impl Into<WeakNetwork>, node_ref:impl Into<TypedAnyNodeRef<Def>>) -> Self {
        let network  = network.into();
        let node_ref = node_ref.into();
        let out_type = default();
        Self {network,node_ref,out_type}
    }

    fn generalize(&self) -> Node<AnyNode,Out> {
        let network  = self.network.clone_ref();
        let node_ref = self.node_ref.generalize();
        let out_type = self.out_type.clone();
        Node {network,node_ref,out_type}
    }
}

#[derive(Debug)]
pub struct NodeModel<Def> {
    definition : Def,
    targets    : Vec<usize>,
    watchers   : Vec<usize>,
    value_ptr  : AnyRc,
    active     : bool,
}

impl<Def> NodeModel<Def> {
    fn new<Data:'static>(definition:Def, data:Data) -> Self {
        let value_ptr = AnyRc::new(data);
        let targets   = default();
        let watchers  = default();
        let active    = default();
        Self {definition,targets,watchers,value_ptr,active}
    }
}

impl<Def> Deref for NodeModel<Def> {
    type Target = Def;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl<Def> DerefMut for NodeModel<Def> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.definition
    }
}





#[derive(Clone,Copy,Debug)]
pub struct AnyNodeRef(usize);

#[derive(Debug)]
pub struct TypedAnyNodeRef<T> {
    node_ref  : AnyNodeRef,
    node_type : PhantomData<T>,
}

impl<T> TypedAnyNodeRef<T> {
    pub fn unchecked_new(node_ref:AnyNodeRef) -> Self {
        let node_type = PhantomData;
        Self {node_ref,node_type}
    }

    pub fn generalize(&self) -> TypedAnyNodeRef<AnyNode> {
        let node_ref  = self.node_ref;
        let node_type = PhantomData;
        TypedAnyNodeRef {node_ref,node_type}
    }
}

impl<T> Copy  for TypedAnyNodeRef<T> {}
impl<T> Clone for TypedAnyNodeRef<T> {
    fn clone(&self) -> Self {
        let node_ref  = self.node_ref.clone();
        let node_type = self.node_type.clone();
        Self {node_ref,node_type}
    }
}



// ==============
// === Source ===
// ==============

pub type Source<Out> = Node<AnySource,Out>;
pub type AnySource = NodeModel<SourceData>;

#[derive(Debug)]
pub struct SourceData {network:WeakNetwork}

impl SourceData {
    pub fn new(network:impl Into<WeakNetwork>) -> Self {
        let network = network.into();
        Self {network}
    }
}

impl AnySource {
    pub fn update(&mut self, _:&mut NetworkModel) -> bool {
        false
    }
}

impl<Out> Source<Out> {
    pub fn emit(&self, data:&Out) {
        if let Some(network) = self.network.upgrade() {
            let network_ref = network.model.borrow_mut().emit(self.node_ref);
        }
    }
}


// ============
// === Gate ===
// ============

pub type Gate<Out> = Node<GateData,Out>;
pub type AnyGate = NodeModel<GateData>;

#[derive(Debug)]
pub struct GateData {source:AnyNodeRef, condition:AnyNodeRef}

impl GateData {
    pub fn new(source:impl Into<AnyNodeRef>, condition:impl Into<AnyNodeRef>) -> Self {
        let source    = source.into();
        let condition = condition.into();
        Self {source,condition}
    }
}

impl AnyGate {
    pub fn update(&mut self, network:&mut NetworkModel) -> bool {
        let condition_ptr = network.index(self.condition).value_ptr();
        let condition : bool = condition_ptr.unsafe_cast_copy();
        if condition { self.value_ptr = network.index(self.source).value_ptr().clone() }
        condition
    }
}



// ===============
// === AnyNode ===
// ===============

pub type Stream<Out> = Node<AnyNode,Out>;

impl<Out> From<Source<Out>> for Stream<Out> {
    fn from(t:Source<Out>) -> Self { t.generalize() }
}

impl<Out> From<&Source<Out>> for Stream<Out> {
    fn from(t:&Source<Out>) -> Self { t.generalize() }
}


#[derive(Debug)]
pub enum AnyNode {
    Source (AnySource),
    Gate   (AnyGate)
}

impl AnyNode {
    pub fn value_ptr(&self) -> &AnyRc {
        match self {
            Self::Source(t) => &t.value_ptr,
            Self::Gate(t)   => &t.value_ptr,
        }
    }
}



// ===============
// === Network ===
// ===============


#[derive(Clone,CloneRef,Debug,Default)]
pub struct Network {
    model : Rc<RefCell<NetworkModel>>
}

#[derive(Clone,CloneRef,Debug)]
pub struct WeakNetwork {
    model : Weak<RefCell<NetworkModel>>
}

#[derive(Debug,Default)]
pub struct NetworkModel {
    nodes : Vec<AnyNode>
}

impl WeakNetwork {
    pub fn upgrade(&self) -> Option<Network> {
        self.model.upgrade().map(|model| Network{model})
    }
}

impl Network {
    pub fn downgrade(&self) -> WeakNetwork {
        let model = Rc::downgrade(&self.model);
        WeakNetwork {model}
    }

    fn source<Out:Default+'static>(&self) -> Source<Out> {
        self.source_with(default())
    }

    fn source_with<Out:'static>(&self, data:Out) -> Source<Out> {
        let network      = self.downgrade();
        let node_data    = SourceData::new(&network);
        let node_model   = NodeModel::new(node_data,data);
        let any_node     = AnyNode::Source(node_model);
        let any_node_ref = self.model.borrow_mut().insert(any_node);
        let node_ref     = TypedAnyNodeRef::unchecked_new(any_node_ref);
        Source::new(network,node_ref)
    }

    fn gate<S,C,Out:Default+'static>
    (&self, source:Node<S,Out>, condition:Node<C,bool>) -> Gate<Out> {
        let source       = source.generalize();
        let condition    = condition.generalize();
        let data:Out     = default();
        let network      = self.downgrade();
        let node_data    = GateData::new(source.node_ref.node_ref,condition.node_ref.node_ref);
        let node_model   = NodeModel::new(node_data,data);
        let any_node     = AnyNode::Gate(node_model);
        let any_node_ref = self.model.borrow_mut().insert(any_node);
        let node_ref     = TypedAnyNodeRef::unchecked_new(any_node_ref);
        Gate::new(network,node_ref)
    }
}

impl NetworkModel {
    fn insert(&mut self, node:AnyNode) -> AnyNodeRef {
        let index = self.nodes.len();
        self.nodes.push(node);
        AnyNodeRef(index)
    }

    fn emit(&mut self, node_ref:TypedAnyNodeRef<AnySource>) {
        todo!()
    }
}

impl Index<AnyNodeRef> for NetworkModel {
    type Output = AnyNode;
    fn index(&self, index:AnyNodeRef) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl IndexMut<AnyNodeRef> for NetworkModel {
    fn index_mut(&mut self, index:AnyNodeRef) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}



impl Index<TypedAnyNodeRef<AnyGate>> for NetworkModel {
    type Output = AnyGate;
    fn index(&self, index:TypedAnyNodeRef<AnyGate>) -> &Self::Output {
        match &self.nodes[index.node_ref.0] {
            AnyNode::Gate(t) => t,
            _ => unreachable!()
        }
    }
}

impl IndexMut<TypedAnyNodeRef<AnyGate>> for NetworkModel {
    fn index_mut(&mut self, index:TypedAnyNodeRef<AnyGate>) -> &mut Self::Output {
        match &mut self.nodes[index.node_ref.0] {
            AnyNode::Gate(t) => t,
            _ => unreachable!()
        }
    }
}



impl Index<TypedAnyNodeRef<AnySource>> for NetworkModel {
    type Output = AnySource;
    fn index(&self, index:TypedAnyNodeRef<AnySource>) -> &Self::Output {
        match &self.nodes[index.node_ref.0] {
            AnyNode::Source(t) => t,
            _ => unreachable!()
        }
    }
}

impl IndexMut<TypedAnyNodeRef<AnySource>> for NetworkModel {
    fn index_mut(&mut self, index:TypedAnyNodeRef<AnySource>) -> &mut Self::Output {
        match &mut self.nodes[index.node_ref.0] {
            AnyNode::Source(t) => t,
            _ => unreachable!()
        }
    }
}




/////////////////////////////////////



pub fn add(i:usize) -> usize {
    i + 4
}

pub fn add_(i: *const c_void) -> usize {
    let j = unsafe { *(i as *const usize) };
    add(j)
}




pub struct X {i:usize}

impl Drop for X {
    fn drop(&mut self) {
        println!("DROP");
    }
}



fn tst() {
    let v = AnyRc::new(X{i:7});
    let x = unsafe { &*(v.raw as *const X) };
    println!(">> {}", x.i);

}



pub fn test () {
    println!("Hello world");

    let mut state : usize = 20;
    let state_ptr: *const c_void = &state as *const _ as *const c_void;
    println!(">> {}", add_(state_ptr));

    tst();

    let network = Network::default();
    let src1  : Source<usize> = network.source();
    let src1_ : Stream<usize> = (&src1).into();
    let cond  : Source<bool>  = network.source();
    let gate  : Gate<usize>   = network.gate(src1,cond);
    println!("network: {:#?}", network);
    // TODO: How to propagate initial value?
}

pub trait Adder {
    fn adder(&self) -> usize;
}

impl Adder for usize {
    fn adder(&self) -> usize {
        add(*self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;


    #[bench]
    fn bench_1(b: &mut Bencher) {
        let state : usize = test::black_box(10);
        let n = test::black_box(1000000);
        let state_ptr: *const c_void = test::black_box(&state as *const _ as *const c_void);

        b.iter(|| {
            (0..n).for_each(|_| {
                add_(state_ptr);
            })
        });
    }

    #[bench]
    fn bench_2(b: &mut Bencher) {
        let state : usize = test::black_box(10);
        let n = test::black_box(1000000);

        let add_x = test::black_box(Box::new(state) as Box<dyn Adder>);

        b.iter(move || {
            (0..n).for_each(|_| {
                add_x.adder();
            })
        });
    }
}