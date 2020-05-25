//! Provides generic operations that can be applied to UI components.
use crate::prelude::*;

use ensogl::display::DomScene;
use ensogl::display::DomSymbol;
use ensogl::display::Scene;
use ensogl::display::Symbol;
use ensogl::display::scene::View;
use ensogl::display::traits::*;
use ensogl::display;



// ==================================
// === UI Component Helper Traits ===
// ==================================

/// Should be implemented by UI component that consist of `Symbol`. Provides access to the shapes
/// and some helper methods for working with those shapes.
pub trait NativeUiElement {
    /// Return all `Symbol`s that make up this component.
    fn shapes(&self) -> Vec<Symbol>;

    /// Change the scene layer of all `Symbol`s.
    fn set_layer(&self, layer:&View) {
        self.shapes().iter().for_each(|symbol| layer.add(symbol))
    }

    /// Remove the `Symbol`s from all scene layers.
    fn unset_layer(&self, scene:&Scene) {
        self.shapes().iter().for_each(|symbol| scene.views.remove_symbol(symbol))
    }
}

/// Should be implemented by UI component that consist of `DomSymbol`s. Provides access to the
/// symbols some helper methods for working with them.
pub trait HtmlUiElement {
    /// Return all `DomSymbol`s that make up this component.
    fn elements(&self) -> Vec<DomSymbol>;

    /// Change the `DomScene` of all `DomSymbol`s.
    fn set_layer(&self, scene:&DomScene) {
        self.elements().iter().for_each(|element| scene.manage(&element))
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



// ==================================
// === Fullscreen Operator Handle ===
// ==================================

/// FullscreenOperatorCellHandle is a helper type that wraps a `FullscreenOperator` and applies an
/// undos the operator on the inner component. This can be used to ensure that only a single
/// component is made fullscreen at any time.
#[derive(Debug,CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Default(bound=""))]
pub struct FullscreenOperatorHandle<T> {
    operator: Rc<RefCell<Option<FullscreenOperator<T>>>>
}

impl<T:display::Object+Resizable+NativeUiElement> FullscreenOperatorHandle<T> {
    /// returns whether there is a component that is in fullscreen mode.
    pub fn is_active(&self) -> bool {
        self.operator.borrow().is_some()
    }

    /// Enables fullscreen mode for the given component. If there is another component already in
    /// fullscreen mode, it disables fullscreen for that component.
    pub fn set_fullscreen(&self, target:T, scene:Scene) {
        self.disable_fullscreen();
        let operator = FullscreenOperator::apply(target,scene);
        self.operator.set(operator);
    }

    /// Disables fullscreen mode for the given component.
    pub fn disable_fullscreen(&self) {
        if let Some(old) = self.operator.borrow_mut().take() {
            old.undo();
        }
    }
}



// ===========================
// === Fullscreen Operator ===
// ===========================

/// A `FullscreenOperator` can be used to apple fullscreen mode to a UI element as well as undo the
/// undo the fullscreen operation and restore the previous state. The  `FullscreenOperator` can be
/// applied to any target that implements `display::Object`, `Resizable` and `NativeUiElement`.
// TODO consider incorporating these traits into display::Object or another common "SceneElement"
// type. But it important that complex UI components can provide information about their
// sub-components (for example, multiple sub-shapes or HTML components).
#[derive(Debug)]
pub struct FullscreenOperator<T> {
    target            : T,
    scene             : Scene,
    size_original     : Vector3<f32>,
    position_original : Vector3<f32>,
    parent_original   : Option<display::object::Instance>,
}

impl<T:display::Object+Resizable+NativeUiElement> FullscreenOperator<T> {

    /// Make the provided target fullscreen within the given scene and return the
    /// `FullscreenOperator`.
    pub fn apply(target:T, scene:Scene) -> Self {
        let size_original     = target.size();
        let position_original = target.position();
        let parent_original   = target.display_object().rc.parent();
        FullscreenOperator {target,scene,size_original,position_original,parent_original}.init()
    }

    fn init(self) -> Self {
        // Change parent
        self.target.display_object().set_parent(self.scene.display_object());
        self.target.unset_layer(&self.scene);
        self.target.set_layer(&self.scene.views.overlay);
        // Change size
        // TODO enable resizing on scene size change
        let margin = 0.1;
        let scene_shape = self.scene.shape();
        let size_new    = Vector3::new(scene_shape.width(), scene_shape.height(),0.0) * (1.0 - margin);
        self.target.set_size(size_new);
        // Change position
        // TODO Currently we assume objects are center aligned, but this needs to be properly
        // accounted for here.
        self.target.set_position(size_new/2.0);

        self.scene.views.toggle_overlay_cursor();

        self
    }

    /// Undo the fullscreen operation and restore the previous state exactly as it was.
    pub fn undo(self) {
        self.target.unset_layer(&self.scene);
        self.target.set_layer(&self.scene.views.main);

        self.target.set_size(self.size_original);
        self.target.set_position(self.position_original);
        if let Some(parent) = self.parent_original {
            self.target.display_object().set_parent(&parent);
        }
        self.scene.views.toggle_overlay_cursor();
    }
}
