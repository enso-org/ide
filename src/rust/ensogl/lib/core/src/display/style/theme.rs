//! Defines `Theme`, a smart style manager on top of style sheets.

use crate::prelude::*;

use crate::data::HashMapTree;
use crate::data::color;

use super::sheet::Path;
use super::sheet::Change;
use super::sheet::Value;
use super::sheet as style;

use crate::control::callback;



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

    /// Insert a new style in the theme.
    pub fn set<P,E>(&self, path:P, entry:E)
    where P:Into<Path>, E:Into<Value> {
        let path  = path.into();
        let entry = entry.into();
        self.tree.borrow_mut().set(&path.rev_segments,Some(entry));
        self.on_mut.run_all();
    }

    /// Add a new callback which will be triggered everytime this theme is modified.
    pub fn on_mut(&self, callback:impl callback::CallbackMutFn) -> callback::Handle {
        self.on_mut.add(callback)
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
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Manager {
    data    : Rc<RefCell<ManagerData>>,
    handles : Rc<RefCell<HashMap<String,callback::Handle>>>,
}

impl Manager {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Return a theme of the given name.
    pub fn get(&self, name:&str) -> Option<Theme> {
        self.data.borrow().get(name).cloned()
    }

    /// Registers a new theme.
    pub fn register<T:Into<Theme>>(&self, name:impl Str, theme:T) {
        let name      = name.into();
        let theme     = theme.into();
        let weak_data = Rc::downgrade(&self.data);
        let handle    = theme.on_mut(move || {
            if let Some(data) = weak_data.upgrade() {
                data.borrow_mut().refresh()
            }
        });
        self.data.borrow_mut().register(&name,theme);
        self.handles.borrow_mut().insert(name,handle);
    }

    /// Sets a new set of enabled themes.
    pub fn set_enabled<N>(&self, names:N)
    where N:IntoIterator, N::Item:ToString {
        self.data.borrow_mut().set_enabled(names)
    }
}

impl From<&style::Sheet> for Manager {
    fn from(style_sheet:&style::Sheet) -> Self {
        let data    = Rc::new(RefCell::new(style_sheet.into()));
        let handles = default();
        Self {data,handles}
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
