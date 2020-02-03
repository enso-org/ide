//! This module defines a common template structure used to define FRP nodes.

use crate::prelude::*;

use crate::data::*;
use crate::debug::*;
use crate::node::class::*;
use crate::node::id::*;
use crate::node::label::*;



// ===================
// === NodeWrapper ===
// ===================

// === NodeWrapper ===

/// `NodeWrapper` is an outer layer for every FRP node. For example, the `Source<Out>` node is just
/// an alias to `NodeWrapper<SourceShape<Out>>`, where `SourceShape` is it's internal representation.
/// This struct bundles each node with information about target edges. Although the edges are used
/// only to send events, they are bundled to every node type in order to keep the implementation
/// simple.
pub type NodeWrapper<Shape> = NodeWrapperTemplate<Shape,Output<Shape>>;

impl<Shape:KnownOutput> NodeWrapper<Shape> {
    /// Constructor.
    pub fn construct<Label>(label:Label, shape:Shape) -> Self
        where Label : Into<CowString> {
        let data = NodeWrapperTemplateData::construct(label,shape);
        let rc   = Rc::new(RefCell::new(data));
        let this = Self {rc};
        this.set_display_id(this.id());
        this
    }
}

impl<Shape,Out> NodeWrapperTemplate<Shape,Out> {
    /// Sends an event to all the children.
    pub fn emit_event(&self, event:&Out) {
        self.rc.borrow().targets.iter().for_each(|target| {
            target.on_event(event)
        })
    }
}

impl<Shape,T:Value>
HasEventTargets for NodeWrapperTemplate<Shape,EventData<T>> {
    fn add_event_target(&self, target:AnyEventConsumer<EventData<T>>) {
        self.rc.borrow_mut().targets.push(target);
    }
}


// === NodeWrapperTemplate ===

/// Internal representation for `NodeWrapper`.
#[derive(Debug,Derivative)]
#[derivative(Default(bound="Shape:Default"))]
#[derivative(Clone(bound=""))]
#[allow(missing_docs)]
pub struct NodeWrapperTemplate<Shape,Out> {
    pub rc: Rc<RefCell<NodeWrapperTemplateData<Shape,Out>>>
}

impl<Shape,Out> CloneRef for NodeWrapperTemplate<Shape,Out> {}

impl<Shape,Out>
HasId for NodeWrapperTemplate<Shape,Out> {
    fn id(&self) -> usize {
        Rc::downgrade(&self.rc).as_raw() as *const() as usize
    }
}

impl<Shape,Out>
HasDisplayId for NodeWrapperTemplate<Shape,Out> {
    fn display_id     (&self) -> usize  { self.rc.borrow().display_id }
    fn set_display_id (&self, id:usize) { self.rc.borrow_mut().display_id = id; }
}

impl<Shape,Out:Data>
KnownOutput for NodeWrapperTemplate<Shape,Out> {
    type Output = Out;
}

impl<Shape,Out>
KnownEventInput for NodeWrapperTemplate<Shape,Out>
    where Shape:KnownEventInput, EventInput<Shape>:Data {
    type EventInput = EventInput<Shape>;
}

impl<Shape,T:Value>
EventEmitter for NodeWrapperTemplate<Shape,EventData<T>> {
    fn emit(&self, event:&Self::Output) {
        self.emit_event(event);
    }
}

impl<Shape:HasInputs,Out>
HasInputs for NodeWrapperTemplate<Shape,Out> {
    fn inputs(&self) -> Vec<AnyNode> {
        self.rc.borrow().shape.inputs()
    }
}

impl<Shape,Out>
HasLabel for NodeWrapperTemplate<Shape,Out> {
    fn label(&self) -> CowString {
        self.rc.borrow().label.clone()
    }
}

impl<Shape:HasInputs,Out>
GraphvizBuilder for NodeWrapperTemplate<Shape,Out> {
    fn graphviz_build(&self, builder:&mut Graphviz) {
        let type_name  = base_type_name::<Shape>();
        let label      = &self.rc.borrow().label;
        let id         = self.id();
        let display_id = self.display_id();
        if !builder.contains(id) {
            builder.add_node(id,display_id,type_name,label);
            for input in &self.rc.borrow().shape.inputs() {
                let input_display_id = input.display_id();
                let input_type       = input.output_type();
                let input_type_name  = input.output_type_value_name();
                input.graphviz_build(builder);
                builder.add_link(input_display_id,display_id,input_type,&input_type_name);
            }
        }
    }
}


// === NodeWrapperTemplateData ===

/// Internal representation for `NodeWrapperTemplate`.
#[derive(Debug,Derivative)]
#[derivative(Default(bound="Shape:Default"))]
pub struct NodeWrapperTemplateData<Shape,Out> {
    /// The label of the node. Used for debugging purposes.
    pub label : CowString,
    /// The display id of the node. Used to group nodes together in the visualization view.
    pub display_id : usize,
    /// The shape of the node.
    pub shape : Shape,
    /// Event targets of the node.
    pub targets : Vec<AnyEventConsumer<Out>>,
}

impl<Shape,Out> NodeWrapperTemplateData<Shape,Out> {
    /// Constructor.
    pub fn construct<Label>(label:Label, shape:Shape) -> Self
        where Label : Into<CowString> {
        let label      = label.into();
        let targets    = default();
        let display_id = 0;
        Self {label,display_id,shape,targets}
    }
}


// === Utils ===

fn base_type_name<T>() -> String {
    let qual_name = type_name::<T>();
    let base_name = qual_name.split('<').collect::<Vec<_>>()[0];
    let name      = base_name.rsplit("::").collect::<Vec<_>>()[0];
    let name      = name.split("Shape").collect::<Vec<_>>()[0];
    name.into()
}
