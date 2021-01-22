//! Scene layers implementation. See docs of [`Layers`] to learn more.
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
    -> (ShapeSystemId,SymbolId,attribute::InstanceIndex,Vec<ShapeSystemId>,Vec<ShapeSystemId>) {
        let system       = self.get_or_register::<DynShapeSystemOf<T>>();
        let system_id    = DynShapeSystemOf::<T>::id();
        let instance_id  = system.instantiate(shape);
        let symbol_id    = system.shape_system().sprite_system.symbol.id;
        let always_above = DynShapeSystemOf::<T>::always_above();
        let always_below = DynShapeSystemOf::<T>::always_below();
        (system_id,symbol_id,instance_id,always_above,always_below)
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

/// A single scene layer. See documentation of [`Layers`] to learn more.
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


#[derive(Clone,Debug)]
pub struct ShapeSystemSymbolInfo {
    symbol_id    : SymbolId,
    always_above : Vec<ShapeSystemId>,
    always_below : Vec<ShapeSystemId>,
}

impl ShapeSystemSymbolInfo {
    fn new
    (symbol_id:SymbolId, always_above:Vec<ShapeSystemId>, always_below:Vec<ShapeSystemId>) -> Self {
        Self {symbol_id,always_above,always_below}
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
    shape_system_symbol_info : RefCell<HashMap<ShapeSystemId,ShapeSystemSymbolInfo>>,
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
        let shape_system_symbol_info = default();
        let elements                = default();
        let symbols_ordered         = default();
        let depth_order             = default();
        let depth_order_dirty       = dirty::SharedBool::new(logger_dirty,on_mut);
        let all_layers_model        = all_layers_model.clone();

        Self {id,logger,camera,shape_registry,shape_system_symbol_info,elements
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

    pub fn add_shape(&self, shape_system_id:ShapeSystemId, symbol_id:impl Into<SymbolId>,always_above:Vec<ShapeSystemId>,always_below:Vec<ShapeSystemId>) {
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

        let info = ShapeSystemSymbolInfo::new(symbol_id,always_above,always_below);
        self.shape_system_symbol_info.borrow_mut().insert(shape_system_id,info);
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
        self.shape_system_symbol_info.borrow_mut().remove(&shape_system_id);
        self.elements.borrow_mut().remove(&LayerElement::ShapeSystem(shape_system_id));

        if let Some(placement) = self.all_layers_model.borrow_mut().symbols_placement.get_mut(&symbol_id) {
            placement.remove_item(&self.id);
        }
    }

    fn combined_depth_order_graph(&self, global_element_depth_order:&DependencyGraph<LayerElement>)
    -> DependencyGraph<LayerElement> {
        let mut graph = global_element_depth_order.clone();
        graph.extend(self.depth_order.borrow().clone().into_iter());
        for element in &*self.elements.borrow() {
            match element {
                LayerElement::ShapeSystem(id) => {
                    if let Some(info) = self.shape_system_symbol_info.borrow().get(&id) {
                        for &id2 in &info.always_below {
                            graph.insert_dependency(*element,LayerElement::ShapeSystem(id2));
                        }
                        for &id2 in &info.always_above {
                            graph.insert_dependency(LayerElement::ShapeSystem(id2),*element);
                        }
                    }
                }
                _ => {}
            }
        };
        graph
    }

    fn depth_sort(&self, global_element_depth_order:&DependencyGraph<LayerElement>) {
        let graph           = self.combined_depth_order_graph(global_element_depth_order);
        let elements_rev    = self.elements.borrow().iter().copied().rev().collect_vec();
        let sorted_elements = graph.into_unchecked_topo_sort(elements_rev);

        let sorted_symbols = sorted_elements.into_iter().filter_map(|element| {
            match element {
                LayerElement::Symbol(symbol_id) => Some(symbol_id),
                LayerElement::ShapeSystem(id) => {
                    let out = self.shape_system_symbol_info.borrow().get(&id).map(|t|t.symbol_id);
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

/// [`Scene`] layers implementation. Scene can consist of one or more layers. Each layer is assigned
/// with a camera and set of [`Symbol`]s to be displayed. Layers can both share cameras and symbols.
///
/// For example, you can create a layer which displays the same symbols as another layer, but from a
/// different camera to create a "mini-map view" of a graph editor.
///
/// ```ignore
/// +------+.
/// |`.    | `.  Layer 1 (top)
/// |  `+--+---+ (Camera 1 and symbols [1,2,3])
/// +---+--+.  |
/// |`. |  | `.| Layer 2 (middle)
/// |  `+------+ (Camera 2 and symbols [3,4,5])
/// +---+--+.  |
///  `. |    `.| Layer 3 (bottom)
///    `+------+ (Camera 1 and symbols [3,6,7])
/// ```
///
///
/// # Layer Ordering
/// Layers can be ordered by using the `add_layers_order_dependency` and the
/// `remove_layers_order_dependency` methods, respectively. The API allows defining a depth-order
/// dependency graph which will be resolved during a frame update. All symbols from lower layers
/// will be drawn to the screen before symbols from the upper layers.
///
///
/// # Symbols Ordering
/// There are two ways to define symbol ordering in scene layers, a global, and local (per-layer)
/// one. In order to define a global depth-order dependency, you can use the
/// `add_elements_order_dependency` and the `remove_elements_order_dependency` methods respectively.
/// In order to define local (per-layer) depth-order dependency, you can use methods of the same
/// names in every layer instance. After changing a dependency graph, the layer management marks
/// appropriate dirty flags and re-orders symbols on each new frame processed.
///
/// During symbol sorting, the global and local dependency graphs are merged together. The defined
/// rules are equivalently important, so local rules will not override global ones. In case of
/// lack of dependencies or circular dependencies, the symbol ids are considered (the ids are
/// increasing with every new symbol created).
///
/// Please note, that symbol ordering doesn't work cross-layer. Even if you define that symbol A has
/// to be above symbol B, but you place symbol B on a layer above the layer of the symbol A, the
/// symbol A will be drawn first, below symbol B!
///
///
/// # Shapes Ordering
/// Ordering of shapes is more tricky than ordering of [`Symbol`]s. Each shape instance will be
/// assigned with an unique [`Symbol`] when placed on a stage, but the connection may change or can
/// be missing when the shape will be detached from the display object hierarchy or when the shape
/// will be moved between the layers. Read the "Shape Management" section below to learn why.
///
/// Shapes can be ordered by using the same methods as symbols (described above). In fact, the
/// depth-order dependencies can be seamlessly defined between both [`Symbol`]s and
/// [`DynamicShape`]s thanks to the [`LayerElement`] abstraction. Moreover, there is a special
/// shapes ordering API allowing describing their dependencies without requiring references to their
/// instances (unlike the API described above). You can add or remove depth-order dependencies for
/// shapes based solely on their types by using the [`add_shapes_order_dependency`] and the
/// [`remove_shapes_order_dependency`] methods, respectively.
///
/// Also, there is a macro [`shapes_order_dependencies!`] which allows convenient form for
/// defining the depth-order dependency graph for shapes based on their types.
///
///
/// # Compile Time Shapes Ordering Relations
/// There is also a third way to define depth-dependencies for shapes. However, unlike previous
/// methods, this one does not require you to own a reference to [`Scene`] or its [`Layers`]. Also,
/// it is impossible to remove during runtime dependencies created this way. This might sound
/// restrictive, but actually it is what you may often want to do. For example, when creating a
/// text area, you want to define that the cursor should always be above its background and there is
/// no situation when it should not be hold. In such a way, you should use this method to define
/// depth-dependencies. In order to define such compile tie shapes ordering relations, you have to
/// define them while defining the shape system. The easiest way to do it is by using the
/// [`define_shape_system!`] macro. Refer to its documentation to learn more.
///
///
/// # Layer Lifetime Management
/// Both [`Layers`] and every [`Layer`] instance are strongly interconnected. This is needed for a
/// nice API. For example, [`Layer`] allows you to add symbols while removing them from other layers
/// automatically. Although the [`LayersModel`] registers [`WeakLayer`], the weak form is used only
/// to break cycles and never points to a dropped [`Layer`], as layers update the information on
/// drop.
#[derive(Clone,CloneRef,Debug)]
pub struct Layers {
    logger                     : Logger,
    global_element_depth_order : ElementDepthOrder,
    model                      : Rc<RefCell<LayersModel>>,
    element_depth_order_dirty  : dirty::SharedBool,
    layers_depth_order_dirty   : dirty::SharedBool,
}

impl Layers {
    /// Constructor.
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

    /// Query [`Layer`] by [`LayerId`].
    pub fn get(&self, layer_id:LayerId) -> Option<Layer> {
        self.model.borrow().get(layer_id)
    }

    /// All layers getter.
    pub fn all(&self) -> Vec<Layer> {
        self.model.borrow().all()
    }

    /// Add a new [`Layer`].
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

    /// Update the layers. This checks all dirty flags, sorts the layers and sort symbols in all
    /// layers affected by previous changes. This function is usually called once per animation
    /// frame.
    pub(crate) fn update(&self) {
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

    pub fn add_layers_order_dependency(&self, below:impl Into<LayerId>, above:impl Into<LayerId>) {
        let below = below.into();
        let above = above.into();
        self.layers_depth_order_dirty.set();
        self.model.borrow_mut().layer_depth_order.insert_dependency(below,above);
    }

    pub fn remove_layers_order_dependency(&self, below:impl Into<LayerId>, above:impl Into<LayerId>) {
        let below = below.into();
        let above = above.into();
        if self.model.borrow_mut().layer_depth_order.remove_dependency(below,above) {
            self.layers_depth_order_dirty.set();
        }
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
/// shapes_order_dependencies! {
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
macro_rules! shapes_order_dependencies {
    ($scene:expr => {
        $( $p1:ident $(:: $ps1:ident)* -> $p2:ident $(:: $ps2:ident)*; )*
    }) => {$(
        $scene.layers.add_shapes_order_dependency::<$p1$(::$ps1)*::View, $p2$(::$ps2)*::View>();
    )*};
}
