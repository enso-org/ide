//! Provides traits that let us know about the capabilities of complex UI components..

use crate::prelude::*;

use ensogl::display::Scene;
use ensogl::display::Symbol;



// ==================================
// === UI Component Helper Traits ===
// ==================================

/// Indicates the desired target layer.
#[derive(Copy,Clone,Debug)]
pub enum TargetLayer {
    /// A symbol that goes onto the main layer.
    Main,
    /// A visualisation symbol that goes into the visualisation layer.
    Visualisation,
}

/// Contains a symbol and information about which layer it should be placed on.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct SymbolLayoutData {
    pub symbol       : Symbol,
    pub target_layer : TargetLayer,
}

/// Should be implemented by UI component that have `Symbol`s. Provides access to the symbols,
/// as well as some helpers for placing them on the correct layers.
pub trait HasSymbols {
    /// Return all `Symbol`s that make up this component.
    fn symbols(&self) -> Vec<Symbol>;

    /// Return all symbols with their layout data for this component.
    fn symbols_with_data(&self) -> Vec<SymbolLayoutData>;

    /// Remove the `Symbol`s from all scene layers.
    fn unset_layers_all(&self, scene:&Scene) {
        self.symbols().iter().for_each(|symbol| scene.views.remove_symbol(symbol));
    }

    /// Moves the given components shapes to the default scene layers.
    fn set_layers_normal(&self, scene:&Scene){
        self.unset_layers_all(&scene);
        for symbol_data in self.symbols_with_data() {
            match symbol_data.target_layer {
                TargetLayer::Main          => scene.views.main.add(&symbol_data.symbol),
                TargetLayer::Visualisation => scene.views.visualisation.add(&symbol_data.symbol),
            }
        }
    }

    /// Moves the given components shapes to the fullscreen scene layers.
    fn set_layers_fullscreen(&self, scene:&Scene) {
        self.unset_layers_all(&scene);
        for symbol_data in self.symbols_with_data() {
            match symbol_data.target_layer {
                TargetLayer::Main          => scene.views.overlay.add(&symbol_data.symbol),
                TargetLayer::Visualisation => scene.views.overlay_visualisation.add(&symbol_data.symbol),
            }
        }
    }
}

/// Provides information and functionality to resize an element. A complex UI component should
/// implement this and propagate size and layout changes to all its sub-components.
pub trait Resizable {
    /// Set the size for the UI component.
    fn set_size(&self, size:Vector3<f32>);
    /// Return the size of the UI element.
    fn size(&self) -> Vector3<f32>;
}
