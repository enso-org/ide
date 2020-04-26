
use crate::prelude::*;

use crate::data::HashMapTree;
use crate::data::color;

use super::data::Data;
use super::registry::Path;
use super::registry as style;



// ==================
// === Expression ===
// ==================

#[derive(Clone)]
pub struct Expression {
    pub sources  : Vec<Path>,
    pub function : Rc<dyn Fn(&[&Data])->Data>
}

impl Debug for Expression {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Expression")
    }
}

impl PartialEq for Expression {
    fn eq(&self, other:&Self) -> bool {
        (self.sources == other.sources) && Rc::ptr_eq(&self.function,&other.function)
    }
}



// =============
// === Entry ===
// =============

#[derive(Clone,Debug,PartialEq)]
pub enum Entry {
    Value      (Data),
    Expression (Expression)
}

impl From<Expression> for Entry {
    fn from(t:Expression) -> Self {
        Self::Expression(t)
    }
}

impl<T> From<T> for Entry
where T:Into<Data> {
    default fn from(t:T) -> Self {
        Self::Value(t.into())
    }
}

impl Semigroup for Entry {
    fn concat_mut(&mut self, other:&Self) {
        *self = other.clone()
    }

    fn concat_mut_take(&mut self, other:Self) {
        *self = other
    }
}



// =============
// === Theme ===
// =============

#[derive(Clone,Debug,Default)]
pub struct Theme {
    tree : HashMapTree<String,Option<Entry>>
}

impl Theme {
    pub fn new() -> Self {
        default()
    }

    pub fn insert<P,E>(&mut self, path:P, entry:E)
    where P:Into<Path>, E:Into<Entry> {
        let path  = path.into();
        let entry = entry.into();
        self.tree.set(&path.rev_segments,Some(entry));
    }
}

impl Semigroup for Theme {
    fn concat_mut(&mut self, other:&Self) {
        self.tree.concat_mut(&other.tree);
    }
}



// ===============
// === Manager ===
// ===============

#[derive(Debug,Default)]
pub struct Manager {
    all      : HashMap<String,Theme>,
    active   : Vec<String>,
    combined : Theme,
    style    : style::Registry,
}

impl Manager {
    pub fn new() -> Self {
        default()
    }

    pub fn set_active<N>(&mut self, names:N)
    where N:IntoIterator, N::Item:ToString {
        let mut combined = Theme::new();
        self.active = names.into_iter().map(|name| name.to_string()).collect();
        for name in &self.active {
            if let Some(theme) = self.all.get(name) {
                combined.concat_mut(theme);
            }
        };
        let diff = self.combined.tree.zip_clone(&combined.tree);
        for (path,values) in &diff {
            if !values.same() {
                match values.second() {
                    Some(None) => {},
                    None       => self.style.remove_value(path),
                    Some(Some(entry)) => match entry {
                        Entry::Value(data) => self.style.set_value(path,data.clone()),
                        Entry::Expression(expr) => {
                            let vars = expr.sources.iter().map(|source| self.style.var(source)).collect::<Vec<_>>();
                            self.style.set_expression(path,&vars,expr.function.clone());
                        }
                    }
                }
            }
        }
        self.combined = combined;
    }

    pub fn register<T:Into<Theme>>(&mut self, name:impl Str, theme:T) {
        let name  = name.into();
        let theme = theme.into();
        self.all.insert(name,theme);
    }
}



// ============
// === Test ===
// ============

pub fn test() {
    let mut registry = Manager::new();

    let mut theme1 = Theme::new();
    theme1.insert("application.background.color", color::Srgba::new(1.0,0.0,0.0,1.0));
    theme1.insert("animation.duration", 0.5);
    theme1.insert("graph.node.shadow.color", 5.0);
    theme1.insert("graph.node.shadow.size", 5.0);
    theme1.insert("mouse.pointer.color", color::Srgba::new(0.3,0.3,0.3,1.0));

    let mut theme2 = Theme::new();
    theme2.insert("application.background.color", color::Srgba::new(1.0,0.0,0.0,1.0));
    theme2.insert("animation.duration", 0.7);
    theme2.insert("graph.node.shadow.color", 5.0);
    theme2.insert("graph.node.shadow.size", 5.0);
    theme2.insert("mouse.pointer.color", color::Srgba::new(0.3,0.3,0.3,1.0));

    registry.register("theme1",theme1);
    registry.register("theme2",theme2);

    registry.set_active(&["theme1".to_string()]);
    println!("-------------------");
    registry.set_active(&["theme1","theme2"]);
}
