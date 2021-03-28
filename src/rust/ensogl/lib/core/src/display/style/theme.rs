//! Defines `Theme`, a smart style manager on top of style sheets.

use crate::prelude::*;

use crate::data::HashMapTree;
use crate::data::color;

use crate::control::callback;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use super::sheet as style;
use super::sheet::Change;
use super::sheet::Path;
use super::sheet::Value;



// =============
// === Theme ===
// =============

/// Smart style manager. Keeps a hierarchical style map. Styles can either be simple values or
/// expressions. Please note that expressions are not bound in themes and are being bound to
/// specific style sheet endpoints when theme is enabled in the themes `Manager`.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Theme {
    tree   : Rc<RefCell<HashMapTree<String,Option<Value>>>>,
    on_mut : callback::SharedRegistryMut,
}

impl Theme {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Insert a new style in the theme. Returns [`true`] if the operation was successful. It can
    /// fail if provided with malformed value, for example with a string "rgba(foo)". In such a
    /// case, the value will not be applied and the function will return [`false`].
    pub fn set<P,E>(&self, path:P, value:E) -> bool
    where P:Into<Path>, E:TryInto<Value> {
        let path  = path.into();
        let value = value.try_into();
        if let Ok(value) = value {
            self.tree.borrow_mut().set(&path.rev_segments,Some(value));
            self.on_mut.run_all();
            true
        } else {
            false
        }
    }

    /// Add a new callback which will be triggered everytime this theme is modified.
    pub fn on_mut(&self, callback:impl callback::CallbackMutFn) -> callback::Handle {
        self.on_mut.add(callback)
    }

    pub fn value_tree(&self) -> HashMapTree<String,Option<Value>> {
        self.tree.borrow().clone()
    }

    pub fn values(&self) -> Vec<(String,Value)> {
        self.tree.borrow().iter().filter_map(|(path,opt_val)|
            opt_val.as_ref().map(|val| {
                let path = path.into_iter().rev().map(|s|s.clone()).collect_vec().join(".");
                let val  = val.clone();
                (path,val)
            })
        ).collect_vec()
    }
}

impl PartialSemigroup<&Theme> for Theme {
    fn concat_mut(&mut self, other:&Self) {
        self.tree.borrow_mut().concat_mut(&*other.tree.borrow());
    }
}



// ===============
// === Manager ===
// ===============

/// Internal data used by the `Manager`.
#[derive(Debug,Default)]
pub struct ManagerData {
    all         : HashMap<String,Theme>,
    enabled     : Vec<String>,
    combined    : Theme,
    style_sheet : style::Sheet,
}

impl ManagerData {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Return names of all enabled themes.
    pub fn enabled(&self) -> &Vec<String> {
        &self.enabled
    }

    /// Return a reference to the theme of the given name.
    pub fn get(&self, name:&str) -> Option<&Theme> {
        self.all.get(name)
    }

    /// Sets a new set of enabled themes.
    pub fn set_enabled<N>(&mut self, names:N)
    where N:IntoIterator, N::Item:ToString {
        let mut combined = Theme::new();
        self.enabled = names.into_iter().map(|name| name.to_string()).collect();
        for name in &self.enabled {
            if let Some(theme) = self.all.get(name) {
                combined.concat_mut(theme);
            }
        };

        let mut changes = Vec::<Change>::new();
        let diff        = self.combined.tree.borrow().zip_clone(&combined.tree.borrow());
        for (segments,values) in &diff {
            let path   = Path::from_rev_segments(segments);
            let first   = values.first().and_then(|t|t.as_ref());
            let second = values.second().and_then(|t|t.as_ref());
            if !values.same() {
                match (first,second) {
                    (None,None)     => {}
                    (Some(_),None)  => changes.push(Change::new(path,None)),
                    (_,Some(value)) => changes.push(Change::new(path,Some(value.clone()))),
                }
            }
        }
        self.combined = combined;
        self.style_sheet.apply_changes(changes);
    }

    /// Reload the currently selected themes. This function is automatically called when an used
    /// theme changes. The refresh is done lazily, only on fields that actually changed. You should
    /// not need to call it manually.
    pub fn refresh(&mut self) {
        self.set_enabled(self.enabled.clone())
    }

    /// Registers a new theme.
    pub fn register<T:Into<Theme>>(&mut self, name:impl Str, theme:T) {
        let name  = name.into();
        let theme = theme.into();
        self.all.insert(name,theme);
    }

    /// Removes the theme from the regitry.
    pub fn remove(&mut self, name:impl Str) {
        let name = name.as_ref();
        self.all.remove(name);
    }
}

impl From<&style::Sheet> for ManagerData {
    fn from(style_sheet:&style::Sheet) -> Self {
        let style_sheet = style_sheet.clone_ref();
        Self {style_sheet,..default()}
    }
}



// ===============
// === Manager ===
// ===============

/// Theme manager. Allows registering themes by names, enabling, and disabling them.
#[derive(Clone,CloneRef,Debug)]
pub struct Manager {
    logger        : Logger,
    data          : Rc<RefCell<ManagerData>>,
    handles       : Rc<RefCell<HashMap<String,callback::Handle>>>,
    current_dirty : dirty::SharedBool,
    enabled_dirty : dirty::SharedVector<String>,
    initialized   : Rc<Cell<bool>>,
}

impl Manager {
    /// Constructor.
    pub fn new() -> Self {
        let logger        = Logger::new("Theme Manager");
        let current_dirty = dirty::SharedBool::new(Logger::sub(&logger,"dirty"),());
        let enabled_dirty = dirty::SharedVector::new(Logger::sub(&logger,"enabled_dirty"),());
        let data          = default();
        let handles       = default();
        let initialized   = default();
        Self {logger,data,handles,current_dirty,enabled_dirty,initialized}
    }

    /// Return a theme of the given name.
    pub fn get(&self, name:&str) -> Option<Theme> {
        self.data.borrow().get(name).cloned()
    }

    /// Registers a new theme.
    pub fn register<T:Into<Theme>>(&self, name:impl Str, theme:T) {
        let name   = name.into();
        let theme  = theme.into();
        let dirty  = self.current_dirty.clone_ref();
        let handle = theme.on_mut(move || dirty.set());
        self.data.borrow_mut().register(&name,theme);
        self.handles.borrow_mut().insert(name,handle);
    }

    /// Sets a new set of enabled themes.
    pub fn set_enabled<N>(&self, names:N)
    where N:IntoIterator, N::Item:ToString {
        self.enabled_dirty.unset_all();
        for name in names {
            self.enabled_dirty.set(name.to_string())
        }
        // TODO[WD]: This impl should be uncommented and the `self.update()` line removed,
        //   but now it causes project name to be red (to be investigated).
        // // First theme set can skip lazy change, as this is normally done on app startup.
        // // It will also make the startup faster, as the theme will not be updated on the next
        // // frame, which would make all shaders re-compile.
        // if self.initialized.get() {
        //     self.initialized.set(true);
        //     self.update()
        // }
        self.update()
    }

    /// Update the theme manager. This should be done once per an animation frame.
    pub fn update(&self) {
        if self.enabled_dirty.check_all() {
            self.current_dirty.take();
            let names = self.enabled_dirty.take().vec;
            self.data.borrow_mut().set_enabled(&names);
        } else if self.current_dirty.take().check() {
            self.data.borrow_mut().refresh()
        }
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&style::Sheet> for Manager {
    fn from(style_sheet:&style::Sheet) -> Self {
        let mut this = Self::default();
        this.data    = Rc::new(RefCell::new(style_sheet.into()));
        this.handles = default();
        this
    }
}

impl AsRef<Manager> for Manager {
    fn as_ref(&self) -> &Manager {
        self
    }
}



// ============
// === Test ===
// ============

/// Test interactive usage. To be removed in the future.
pub fn test() {
    let theme_manager = Manager::new();

    let theme1 = Theme::new();
    theme1.set("application.background.color", color::Rgba::new(1.0,0.0,0.0,1.0));
    theme1.set("animation.duration", 0.5);
    theme1.set("graph.node.shadow.color", 5.0);
    theme1.set("graph.node.shadow.size", 5.0);
    theme1.set("mouse.pointer.color", color::Rgba::new(0.3,0.3,0.3,1.0));

    let theme2 = Theme::new();
    theme2.set("application.background.color", color::Rgba::new(1.0,0.0,0.0,1.0));
    theme2.set("animation.duration", 0.7);
    theme2.set("graph.node.shadow.color", 5.0);
    theme2.set("graph.node.shadow.size", 5.0);
    theme2.set("mouse.pointer.color", color::Rgba::new(0.3,0.3,0.3,1.0));

    theme_manager.register("theme1",theme1);
    theme_manager.register("theme2",theme2);

    theme_manager.set_enabled(&["theme1".to_string()]);
    println!("-------------------");
    theme_manager.set_enabled(&["theme1","theme2"]);
}
