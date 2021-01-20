use crate::prelude::*;

use crate::data::OptVec;
use crate::data::dirty;
use crate::display::camera::Camera2d;
use crate::display::scene::Scene;
use crate::display::shape::DynShapeSystemInstance;
use crate::display::shape::system::DynShapeSystemOf;
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

    fn get<T:DynShapeSystemInstance>(&self) -> Option<T> {
        let id = TypeId::of::<T>();
        self.shape_system_map.get(&id).and_then(|t| t.downcast_ref::<T>()).map(|t| t.clone_ref())
    }

    fn register<T:DynShapeSystemInstance>(&mut self) -> T {
        let id     = TypeId::of::<T>();
        let system = <T as DynShapeSystemInstance>::new(self.scene.as_ref().unwrap());
        let any    = Box::new(system.clone_ref());
        self.shape_system_map.insert(id,any);
        system
    }

    fn get_or_register<T:DynShapeSystemInstance>(&mut self) -> T {
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



// =========================
// === ElementDepthOrder ===
// =========================

pub type GlobalElementDepthOrder = Rc<RefCell<DependencyGraph<LayerElement>>>;
pub type LocalElementDepthOrder  = Rc<RefCell<Option<DependencyGraph<LayerElement>>>>;

#[derive(Debug,Clone,CloneRef)]
pub struct ElementDepthOrder {
    global : GlobalElementDepthOrder,
    local  : LocalElementDepthOrder,
}

impl ElementDepthOrder {
    pub fn new(global:&GlobalElementDepthOrder) -> Self {
        let global = global.clone_ref();
        let local  = default();
        Self {global,local}
    }

    fn sort(&self, symbols:&BTreeSet<LayerElement>) -> Vec<LayerElement> {
        let local  = self.local.borrow();
        let global = self.global.borrow();
        let graph  = local.as_ref().unwrap_or_else(||&*global);
        graph.unchecked_topo_sort(symbols.iter().copied().rev().collect_vec())
    }
}



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
    pub fn new(logger:&Logger, id:LayerId, global_depth_order:&GlobalElementDepthOrder, model:&Rc<RefCell<LayersModel>>) -> Self {
        let model = LayerModel::new(logger,id,global_depth_order,model);
        let model = Rc::new(model);
        Self {model}
    }

    pub fn camera(&self) -> Camera2d {
        self.camera.borrow().clone_ref()
    }

    pub fn set_camera(&self, camera:impl Into<Camera2d>) {
        let camera = camera.into();
        *self.camera.borrow_mut() = camera;
    }

    pub fn downgrade(&self) -> WeakLayer {
        let model = Rc::downgrade(&self.model);
        WeakLayer {model}
    }

    pub fn add_symbol(&self, symbol_id:impl Into<SymbolId>) {
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
        self.depth_sort();
    }

    pub fn add_shape(&self, shape_system_id:ShapeSystemId, symbol_id:impl Into<SymbolId>) {
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

        self.depth_sort();
    }

    pub fn remove_symbol(&self, symbol_id:impl Into<SymbolId>) {
        let symbol_id = symbol_id.into();

        self.elements.borrow_mut().remove(&LayerElement::Symbol(symbol_id));

        self.depth_sort();

        if let Some(placement) = self.all_layers_model.borrow_mut().symbols_placement.get_mut(&symbol_id) {
            placement.remove_item(&self.id);
        }
    }

    pub fn remove_shape(&self, shape_system_id:ShapeSystemId, symbol_id:impl Into<SymbolId>) {
        let symbol_id = symbol_id.into();

        self.elements.borrow_mut().remove(&LayerElement::Symbol(symbol_id));
        self.shape_system_symbol_map.borrow_mut().remove(&shape_system_id);
        self.elements.borrow_mut().remove(&LayerElement::ShapeSystem(shape_system_id));

        self.depth_sort();

        if let Some(placement) = self.all_layers_model.borrow_mut().symbols_placement.get_mut(&symbol_id) {
            placement.remove_item(&self.id);
        }
    }

    fn depth_sort(&self) {
        let elements_ordered = self.depth_order.sort(&*self.elements.borrow());
        let symbols_ordered  = elements_ordered.into_iter().filter_map(|element| {
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
        *self.symbols_ordered.borrow_mut() = symbols_ordered;
    }

    /// Add all `Symbol`s associated with the given ShapeView_DEPRECATED. Please note that this
    /// function was used only in one place in the codebase and should be removed in the future.
    pub fn add_shape_view_DEPRECATED<T: display::shape::primitive::system::Shape>
    (&self, shape_view:&component::ShapeView_DEPRECATED<T>) {
        self.add_symbol(&shape_view.shape.sprite().symbol)
    }

    /// Remove all `Symbol`s associated with the given ShapeView_DEPRECATED. Please note that this
    /// function was used only in one place in the codebase and should be removed in the future.
    pub fn remove_shape_view_DEPRECATED<T: display::shape::primitive::system::Shape>
    (&self, shape_view:&component::ShapeView_DEPRECATED<T>) {
        self.remove_symbol(&shape_view.shape.sprite().symbol)
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
    depth_order             : ElementDepthOrder,
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
    (logger:impl AnyLogger, id:LayerId, global_depth_order:&GlobalElementDepthOrder, all_layers_model:&Rc<RefCell<LayersModel>>) -> Self {
        let width  = 0.0;
        let height = 0.0;
        let logger                       = Logger::sub(logger,"view");
        let camera                       = RefCell::new(Camera2d::new(&logger,width,height));
        let shape_registry               = default();
        let shape_system_symbol_map      = default();
        let elements                     = default();
        let symbols_ordered              = default();
        let depth_order                  = ElementDepthOrder::new(global_depth_order);
        let all_layers_model             = all_layers_model.clone();

        Self {id,logger,camera,shape_registry,shape_system_symbol_map,elements
            ,symbols_ordered,depth_order,all_layers_model}
    }

    pub fn symbols(&self) -> Vec<SymbolId> {
        self.symbols_ordered.borrow().clone()
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
    logger                   : Logger,
    element_depth_order      : GlobalElementDepthOrder,
    model                    : Rc<RefCell<LayersModel>>,
    layers_depth_order_dirty : dirty::SharedBool,
}

impl Layers {
    pub fn new(logger:impl AnyLogger) -> Self {
        let logger              = Logger::sub(logger,"views");
        let dirty_logger        = Logger::sub(&logger,"dirty");
        let element_depth_order = default();
        let model               = default();
        let layers_depth_order_dirty        = dirty::SharedBool::new(dirty_logger,());
        Self {logger,element_depth_order,model,layers_depth_order_dirty}
    }

    pub fn add(&self) -> Layer {
        let (_,layer) = self.model.borrow_mut().registry.insert_with_ix2(|ix| {
            let id    = LayerId::from(ix);
            let layer = Layer::new(&self.logger,id,&self.element_depth_order,&self.model);
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
    }

    pub fn all(&self) -> Vec<Layer> {
        self.model.borrow().all()
    }

    pub fn order(&self, below:impl Into<LayerId>, above:impl Into<LayerId>) {
        let below = below.into();
        let above = above.into();
        self.model.borrow_mut().layer_depth_order.insert_dependency(below,above);
        self.layers_depth_order_dirty.set();
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
}
