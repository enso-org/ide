use crate::prelude::*;

use crate::data::OptVec;
use crate::data::dirty;
use crate::display::camera::Camera2d;
use crate::display::scene::Scene;
use crate::display::shape::ShapeSystemInstance;
use crate::display::shape::system::DynShapeSystemInstance;
use crate::display::shape::system::DynShapeSystemOf;
use crate::display::shape::system::KnownShapeSystemId;
use crate::display::shape::system::ShapeSystemId;
use crate::display::symbol::SymbolId;
use crate::display;
use crate::gui::component;
use crate::system::gpu::data::attribute;

use enso_data::dependency_graph::DependencyGraph;
use enso_shapely::shared;
use smallvec::alloc::collections::BTreeSet;
use std::any::TypeId;

use crate::data::dirty::traits::*;



// =====================
// === ShapeRegistry ===
// =====================

shared! { ShapeRegistry2
#[derive(Debug,Default)]
pub struct ShapeRegistryData2 {
    scene            : Option<Scene>,
    shape_system_map : HashMap<TypeId,Box<dyn Any>>,
}

impl {
    // FIXME: remove this hack
    pub fn init(&mut self, scene:&Scene) {
        self.scene = Some(scene.clone_ref());
    }

    fn get<T:ShapeSystemInstance>(&self) -> Option<T> {
        let id = TypeId::of::<T>();
        self.shape_system_map.get(&id).and_then(|t| t.downcast_ref::<T>()).map(|t| t.clone_ref())
    }

    fn register<T:ShapeSystemInstance>(&mut self) -> T {
        let id     = TypeId::of::<T>();
        let system = <T as ShapeSystemInstance>::new(self.scene.as_ref().unwrap());
        let any    = Box::new(system.clone_ref());
        self.shape_system_map.insert(id,any);
        system
    }

    fn get_or_register<T:ShapeSystemInstance>(&mut self) -> T {
        self.get().unwrap_or_else(|| self.register())
    }

    pub fn shape_system<T:display::shape::system::DynamicShape>(&mut self, _phantom:PhantomData<T>) -> DynShapeSystemOf<T> {
        self.get_or_register::<DynShapeSystemOf<T>>()
    }

    pub fn instantiate<T:display::shape::system::DynamicShape>(&mut self,shape:&T)
    -> (ShapeSystemId,SymbolId,attribute::InstanceIndex) {
        let system      = self.get_or_register::<DynShapeSystemOf<T>>();
        let system_id   = DynShapeSystemOf::<T>::id();
        let instance_id = system.instantiate(shape);
        let symbol_id   = system.shape_system().sprite_system.symbol.id;
        (system_id,symbol_id,instance_id)
    }
}}



// ====================
// === LayerElement ===
// ====================

#[derive(Clone,Copy,Debug,PartialEq,PartialOrd,Eq,Hash,Ord)]
pub enum LayerElement {
    Symbol      (SymbolId),
    ShapeSystem (ShapeSystemId)
}

impl From<ShapeSystemId> for LayerElement {
    fn from(t:ShapeSystemId) -> Self {
        Self::ShapeSystem(t)
    }
}



// ==================
// === DepthOrder ===
// ==================

pub type ElementDepthOrder = Rc<RefCell<DependencyGraph<LayerElement>>>;

// #[derive(Debug,Clone,CloneRef)]
// pub struct DepthOrder {
//     global : ElementDepthOrder,
//     local  : ElementDepthOrder,
// }
//
// impl DepthOrder {
//     pub fn new(global:&ElementDepthOrder) -> Self {
//         let global = global.clone_ref();
//         let local  = default();
//         Self {global,local}
//     }
//
//     fn sort(&self, symbols:&BTreeSet<LayerElement>) -> Vec<LayerElement> {
//         let mut graph = self.global.borrow().clone();
//         graph.extend(self.local.borrow().clone().into_iter());
//         graph.into_unchecked_topo_sort(symbols.iter().copied().rev().collect_vec())
//     }
// }



// ===============
// === LayerId ===
// ===============

use enso_shapely::newtype_prim;
newtype_prim! {
    /// The ID of a layer. Under the hood, it is the index of the layer.
    LayerId(usize);
}



// =============
// === Layer ===
// =============

#[derive(Debug,Clone,CloneRef)]
pub struct Layer {
    model : Rc<LayerModel>
}

impl Deref for Layer {
    type Target = LayerModel;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl Layer {
    pub fn new(logger:&Logger, id:LayerId, model:&Rc<RefCell<LayersModel>>, on_mut:Box<dyn Fn()>) -> Self {
        let model = LayerModel::new(logger,id,model,on_mut);
        let model = Rc::new(model);
        Self {model}
    }

    pub fn downgrade(&self) -> WeakLayer {
        let model = Rc::downgrade(&self.model);
        WeakLayer {model}
    }
}

impl From<&Layer> for LayerId {
    fn from(t:&Layer) -> Self {
        t.id
    }
}



// =================
// === WeakLayer ===
// =================

#[derive(Clone,CloneRef)]
pub struct WeakLayer {
    model : Weak<LayerModel>
}

impl WeakLayer {
    pub fn upgrade(&self) -> Option<Layer> {
        self.model.upgrade().map(|model| Layer {model})
    }
}

impl Eq        for WeakLayer {}
impl PartialEq for WeakLayer {
    fn eq(&self, other:&Self) -> bool {
        self.model.ptr_eq(&other.model)
    }
}

impl Debug for WeakLayer {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"WeakLayer")
    }
}



// ==================
// === LayerModel ===
// ==================

#[derive(Debug,Clone)]
pub struct LayerModel {
    pub id                  : LayerId,
    logger                  : Logger,
    pub camera              : RefCell<Camera2d>,
    pub shape_registry      : ShapeRegistry2,
    shape_system_symbol_map : RefCell<HashMap<ShapeSystemId,SymbolId>>,
    elements                : RefCell<BTreeSet<LayerElement>>,
    symbols_ordered         : RefCell<Vec<SymbolId>>,
    depth_order             : RefCell<DependencyGraph<LayerElement>>,
    depth_order_dirty       : dirty::SharedBool<Box<dyn Fn()>>,
    all_layers_model        : Rc<RefCell<LayersModel>>,
}

impl Drop for LayerModel {
    fn drop(&mut self) {
        let mut model = self.all_layers_model.borrow_mut();
        model.registry.remove(*self.id);
        model.sorted_layers.remove_item(&self.id);
        // TODO: clean symbol placement
    }
}

impl LayerModel {
    pub fn new
    ( logger           : impl AnyLogger
    , id               : LayerId
    , all_layers_model : &Rc<RefCell<LayersModel>>
    , on_mut           : Box<dyn Fn()>
    ) -> Self {
        let width  = 0.0;
        let height = 0.0;
        let logger                  = Logger::sub(logger,"layer");
        let logger_dirty            = Logger::sub(&logger,"dirty");
        let camera                  = RefCell::new(Camera2d::new(&logger,width,height));
        let shape_registry          = default();
        let shape_system_symbol_map = default();
        let elements                = default();
        let symbols_ordered         = default();
        let depth_order             = default();
        let depth_order_dirty       = dirty::SharedBool::new(logger_dirty,on_mut);
        let all_layers_model        = all_layers_model.clone();

        Self {id,logger,camera,shape_registry,shape_system_symbol_map,elements
            ,symbols_ordered,depth_order,depth_order_dirty,all_layers_model}
    }

    pub fn symbols(&self) -> Vec<SymbolId> {
        self.symbols_ordered.borrow().clone()
    }

    pub fn add_elements_order_dependency
    (&self, below:impl Into<LayerElement>, above:impl Into<LayerElement>) {
        let below = below.into();
        let above = above.into();
        self.depth_order_dirty.set();
        self.depth_order.borrow_mut().insert_dependency(below,above);
    }

    pub fn remove_elements_order_dependency
    (&self, below:impl Into<LayerElement>, above:impl Into<LayerElement>) {
        let below = below.into();
        let above = above.into();
        if self.depth_order.borrow_mut().remove_dependency(below,above) {
            self.depth_order_dirty.set();
        }
    }

    /// # Future Improvements
    /// This implementation can be simplified to `S1:KnownShapeSystemId` (not using [`Content`] at
    /// all), after the compiler gets updated to newer version.
    pub fn add_shapes_order_dependency<S1,S2>(&self) -> (PhantomData<S1>,PhantomData<S2>) where
    S1          : HasContent,
    S2          : HasContent,
    Content<S1> : KnownShapeSystemId,
    Content<S2> : KnownShapeSystemId {
        let s1_id = <Content<S1>>::shape_system_id();
        let s2_id = <Content<S2>>::shape_system_id();
        self.add_elements_order_dependency(s1_id,s2_id);
        default()
    }

    /// # Future Improvements
    /// This implementation can be simplified to `S1:KnownShapeSystemId` (not using [`Content`] at
    /// all), after the compiler gets updated to newer version.
    pub fn remove_shapes_order_dependency<S1,S2>(&self) -> (PhantomData<S1>,PhantomData<S2>) where
    S1          : HasContent,
    S2          : HasContent,
    Content<S1> : KnownShapeSystemId,
    Content<S2> : KnownShapeSystemId {
        let s1_id = <Content<S1>>::shape_system_id();
        let s2_id = <Content<S2>>::shape_system_id();
        self.remove_elements_order_dependency(s1_id,s2_id);
        default()
    }

    pub fn camera(&self) -> Camera2d {
        self.camera.borrow().clone_ref()
    }

    pub fn set_camera(&self, camera:impl Into<Camera2d>) {
        let camera = camera.into();
        *self.camera.borrow_mut() = camera;
    }

    pub fn update(&self, global_element_depth_order:&DependencyGraph<LayerElement>) {
        if self.depth_order_dirty.check() {
            self.depth_order_dirty.unset();
            self.depth_sort(global_element_depth_order);
        }
    }

    pub fn add_symbol(&self, symbol_id:impl Into<SymbolId>) {
        self.depth_order_dirty.set();
        let symbol_id = symbol_id.into();
        let placement = self.all_layers_model.borrow().symbols_placement.get(&symbol_id).cloned();
        if let Some(placement) = placement {
            for layer_id in placement {
                let opt_layer = self.all_layers_model.borrow().registry[*layer_id].upgrade();
                if let Some(layer) = opt_layer {
                    layer.remove_symbol(symbol_id)
                }
            }
        }
        self.all_layers_model.borrow_mut().symbols_placement.entry(symbol_id).or_default().push(self.id);
        self.elements.borrow_mut().insert(LayerElement::Symbol(symbol_id));
    }

    pub fn add_shape(&self, shape_system_id:ShapeSystemId, symbol_id:impl Into<SymbolId>) {
        self.depth_order_dirty.set();
        let symbol_id = symbol_id.into();
        let placement = self.all_layers_model.borrow().symbols_placement.get(&symbol_id).cloned();
        if let Some(placement) = placement {
            for layer_id in placement {
                let opt_layer = self.all_layers_model.borrow().registry[*layer_id].upgrade();
                if let Some(layer) = opt_layer {
                    layer.remove_shape(shape_system_id,symbol_id)
                }
            }
        }
        self.all_layers_model.borrow_mut().symbols_placement.entry(symbol_id).or_default().push(self.id);

        self.shape_system_symbol_map.borrow_mut().insert(shape_system_id,symbol_id);
        self.elements.borrow_mut().insert(LayerElement::ShapeSystem(shape_system_id));
    }

    pub fn remove_symbol(&self, symbol_id:impl Into<SymbolId>) {
        self.depth_order_dirty.set();
        let symbol_id = symbol_id.into();

        self.elements.borrow_mut().remove(&LayerElement::Symbol(symbol_id));

        if let Some(placement) = self.all_layers_model.borrow_mut().symbols_placement.get_mut(&symbol_id) {
            placement.remove_item(&self.id);
        }
    }

    pub fn remove_shape(&self, shape_system_id:ShapeSystemId, symbol_id:impl Into<SymbolId>) {
        self.depth_order_dirty.set();
        let symbol_id = symbol_id.into();

        self.elements.borrow_mut().remove(&LayerElement::Symbol(symbol_id));
        self.shape_system_symbol_map.borrow_mut().remove(&shape_system_id);
        self.elements.borrow_mut().remove(&LayerElement::ShapeSystem(shape_system_id));

        if let Some(placement) = self.all_layers_model.borrow_mut().symbols_placement.get_mut(&symbol_id) {
            placement.remove_item(&self.id);
        }
    }

    fn depth_sort(&self, global_element_depth_order:&DependencyGraph<LayerElement>) {
        let symbols_rev = self.elements.borrow().iter().copied().rev().collect_vec();
        let mut graph   = global_element_depth_order.clone();
        graph.extend(self.depth_order.borrow().clone().into_iter());
        let sorted_elements = graph.into_unchecked_topo_sort(symbols_rev);

        let sorted_symbols  = sorted_elements.into_iter().filter_map(|element| {
            match element {
                LayerElement::Symbol(symbol_id) => Some(symbol_id),
                LayerElement::ShapeSystem(id) => {
                    let out = self.shape_system_symbol_map.borrow().get(&id).copied();
                    if out.is_none() {
                        warning!(self.logger,"Trying to perform depth-order of non-existing element '{id:?}'.")
                    }
                    out
                }
            }
        }).collect();
        *self.symbols_ordered.borrow_mut() = sorted_symbols;
    }

    /// Add all `Symbol`s associated with the given ShapeView_DEPRECATED. Please note that this
    /// function was used only in one place in the codebase and should be removed in the future.
    #[allow(non_snake_case)]
    pub fn add_shape_view_DEPRECATED<T: display::shape::primitive::system::Shape>
    (&self, shape_view:&component::ShapeView_DEPRECATED<T>) {
        self.add_symbol(&shape_view.shape.sprite().symbol)
    }

    /// Remove all `Symbol`s associated with the given ShapeView_DEPRECATED. Please note that this
    /// function was used only in one place in the codebase and should be removed in the future.
    #[allow(non_snake_case)]
    pub fn remove_shape_view_DEPRECATED<T: display::shape::primitive::system::Shape>
    (&self, shape_view:&component::ShapeView_DEPRECATED<T>) {
        self.remove_symbol(&shape_view.shape.sprite().symbol)
    }
}

impl AsRef<LayerModel> for Layer {
    fn as_ref(&self) -> &LayerModel {
        &self.model
    }
}

impl std::borrow::Borrow<LayerModel> for Layer {
    fn borrow(&self) -> &LayerModel {
        &self.model
    }
}



// ==============
// === Layers ===
// ==============

#[derive(Clone,CloneRef,Debug)]
pub struct Layers {
    logger                     : Logger,
    global_element_depth_order : ElementDepthOrder,
    model                      : Rc<RefCell<LayersModel>>,
    element_depth_order_dirty  : dirty::SharedBool,
    layers_depth_order_dirty   : dirty::SharedBool,
}

impl Layers {
    pub fn new(logger:impl AnyLogger) -> Self {
        let logger                     = Logger::sub(logger,"views");
        let element_dirty_logger       = Logger::sub(&logger,"element_dirty");
        let layers_dirty_logger        = Logger::sub(&logger,"layers_dirty");
        let global_element_depth_order = default();
        let model                      = default();
        let element_depth_order_dirty  = dirty::SharedBool::new(element_dirty_logger,());
        let layers_depth_order_dirty   = dirty::SharedBool::new(layers_dirty_logger,());
        Self {logger,global_element_depth_order,model,element_depth_order_dirty
             ,layers_depth_order_dirty}
    }

    pub fn get(&self, layer_id:LayerId) -> Option<Layer> {
        self.model.borrow().get(layer_id)
    }

    pub fn add(&self) -> Layer {
        let (_,layer) = self.model.borrow_mut().registry.insert_with_ix(|ix| {
            let id     = LayerId::from(ix);
            let dirty  = &self.element_depth_order_dirty;
            let on_mut = Box::new(f!(dirty.set()));
            let layer  = Layer::new(&self.logger,id,&self.model,on_mut);
            (layer.downgrade(),layer)
        });
        self.layers_depth_order_dirty.set();
        layer
    }

    pub fn update(&self) {
        if self.layers_depth_order_dirty.check() {
            self.layers_depth_order_dirty.unset();
            let model         = &mut *self.model.borrow_mut();
            let layers        = model.registry.iter().filter_map(|t|t.upgrade().map(|t|t.id));
            let layers_rev    = layers.rev().collect_vec();
            let sorted_layers = model.layer_depth_order.unchecked_topo_sort(layers_rev);
            model.sorted_layers = sorted_layers;
        }

        if self.element_depth_order_dirty.check() {
            self.element_depth_order_dirty.unset();
            for layer in self.all() {
                layer.update(&*self.global_element_depth_order.borrow())
            }
        }
    }

    pub fn all(&self) -> Vec<Layer> {
        self.model.borrow().all()
    }

    pub fn add_layers_order_dependency(&self, below:impl Into<LayerId>, above:impl Into<LayerId>) {
        let below = below.into();
        let above = above.into();
        self.layers_depth_order_dirty.set();
        self.model.borrow_mut().layer_depth_order.insert_dependency(below,above);
    }

    pub fn add_elements_order_dependency
    (&self, below:impl Into<LayerElement>, above:impl Into<LayerElement>) {
        let below = below.into();
        let above = above.into();
        self.element_depth_order_dirty.set();
        self.global_element_depth_order.borrow_mut().insert_dependency(below,above);
    }

    pub fn remove_elements_order_dependency
    (&self, below:impl Into<LayerElement>, above:impl Into<LayerElement>) {
        let below = below.into();
        let above = above.into();
        if self.global_element_depth_order.borrow_mut().remove_dependency(below,above) {
            self.element_depth_order_dirty.set();
        }
    }

    /// # Future Improvements
    /// This implementation can be simplified to `S1:KnownShapeSystemId` (not using [`Content`] at
    /// all), after the compiler gets updated to newer version.
    pub fn add_shapes_order_dependency<S1,S2>(&self) -> (PhantomData<S1>,PhantomData<S2>) where
    S1          : HasContent,
    S2          : HasContent,
    Content<S1> : KnownShapeSystemId,
    Content<S2> : KnownShapeSystemId {
        let s1_id = <Content<S1>>::shape_system_id();
        let s2_id = <Content<S2>>::shape_system_id();
        self.add_elements_order_dependency(s1_id,s2_id);
        default()
    }

    /// # Future Improvements
    /// This implementation can be simplified to `S1:KnownShapeSystemId` (not using [`Content`] at
    /// all), after the compiler gets updated to newer version.
    pub fn remove_shapes_order_dependency<S1,S2>(&self) -> (PhantomData<S1>,PhantomData<S2>) where
    S1          : HasContent,
    S2          : HasContent,
    Content<S1> : KnownShapeSystemId,
    Content<S2> : KnownShapeSystemId {
        let s1_id = <Content<S1>>::shape_system_id();
        let s2_id = <Content<S2>>::shape_system_id();
        self.remove_elements_order_dependency(s1_id,s2_id);
        default()
    }
}



// ===================
// === LayersModel ===
// ===================

#[derive(Debug,Default)]
pub struct LayersModel {
    registry          : OptVec<WeakLayer>,
    symbols_placement : HashMap<SymbolId,Vec<LayerId>>,
    sorted_layers     : Vec<LayerId>,
    layer_depth_order : DependencyGraph<LayerId>,
}

impl LayersModel {
    pub fn all(&self) -> Vec<Layer> {
        self.sorted_layers.iter().filter_map(|id| self.registry[**id].upgrade()).collect()
    }

    pub fn get(&self, layer_id:LayerId) -> Option<Layer> {
        self.registry.safe_index(*layer_id).and_then(|t|t.upgrade())
    }
}



// ==============
// === Macros ===
// ==============

/// Shape ordering utility. Currently, this macro supports ordering of shapes for a given stage.
/// For example, the following usage:
///
/// ```ignore
/// shapes_order_depenendencies! {
///     scene => {
///         output::port::single_port -> shape;
///         output::port::multi_port  -> shape;
///         shape                     -> input::port::hover;
///         input::port::hover        -> input::port::viz;
///     }
/// }
/// ```
///
/// Will expand to:
///
/// ```ignore
/// scene.layers.add_shapes_order_dependency::<output::port::single_port::View, shape::View>();
/// scene.layers.add_shapes_order_dependency::<output::port::multi_port::View, shape::View>();
/// scene.layers.add_shapes_order_dependency::<shape::View, input::port::hover::View>();
/// scene.layers.add_shapes_order_dependency::<input::port::hover::View, input::port::viz::View>();
/// ```
#[macro_export]
macro_rules! shapes_order_depenendencies {
    ($scene:expr => {
        $( $p1:ident $(:: $ps1:ident)* -> $p2:ident $(:: $ps2:ident)*; )*
    }) => {$(
        $scene.layers.add_shapes_order_dependency::<$p1$(::$ps1)*::View, $p2$(::$ps2)*::View>();
    )*};
}
