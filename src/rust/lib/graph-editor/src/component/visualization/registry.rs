//! The `Registry` provides a mechanism to store all available visualization, as well as select
//! visualizations based in their input type.

use crate::prelude::*;

use crate::component::visualization::EnsoType;
use crate::component::visualization::Factory;
use crate::component::visualization::Metadata;
use crate::component::visualization::NativeConstructorFactory;
use crate::component::visualization::Visualization;
use crate::component::visualization::renderer::example::js::constructor_sample_js_bubble_chart;
use crate::component::visualization::renderer::example::native::BubbleChart;

use ensogl::display::scene::Scene;



// ==============================
// === Visualization Registry ===
// ==============================

#[derive(Clone,CloneRef,Default,Debug)]
#[allow(missing_docs)]
pub struct Registry {
    entries : Rc<RefCell<Vec<Rc<dyn Factory>>>>
}

impl Registry {

    /// Return an empty `Registry`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Return a `Registry` prepopulated with default visualizations.
    pub fn with_default_visualisations() -> Self {
        let registry = Self::empty();
        // TODO fix types
        registry.register_factory(NativeConstructorFactory::new(
            Metadata {
                name        : "Bubble Visualisation (native)".to_string(),
                input_types : vec!["[[float;3]]".to_string().into()],
            },
            Rc::new(|scene:&Scene| Ok(Visualization::new(BubbleChart::new(scene))))
        ));
        registry.register_factory(NativeConstructorFactory::new(
            Metadata {
                name        : "Bubble Visualisation (JS)".to_string(),
                input_types : vec!["[[float;3]]".to_string().into()],
            },
            Rc::new(|scene:&Scene| {
                let renderer = constructor_sample_js_bubble_chart();
                renderer.set_dom_layer(&scene.dom.layers.front);
                Ok(Visualization::new(renderer))
            })
        ));

        registry
    }

    /// Register a new visualisation factory with the registry.
    pub fn register_factory<T:Factory + 'static>(&self, factory:T) {
        self.entries.borrow_mut().push(Rc::new(factory));
    }

    /// Register a new visualisation factory that's pre-wrapped in an `Rc` with the registry.
    pub fn register_factory_rc(&self, factory:Rc<dyn Factory>) {
        self.entries.borrow_mut().push(factory);
    }

    /// Return all `Factory`s that can create a visualisation for the given datatype.
    pub fn valid_sources(&self, dtype:&EnsoType) -> Vec<Rc<dyn Factory>>{
        // TODO: this is not super efficient. Consider building a HashMap from type to vis.
        let entries       = self.entries.borrow();
        let entries       = entries.iter();
        let valid_entries = entries.filter(|entry| entry.metadata().input_types.contains(dtype));
        valid_entries.cloned().collect()
    }
}
