
use crate::prelude::*;

use crate::frp;



// ============
// === Data ===
// ============

#[derive(Debug,Clone)]
pub enum Data {
    Invalid(String),
    Number(f32)
}

pub fn data<T:Into<Data>>(t:T) -> Data {
    t.into()
}

impl From<f32> for Data {
    fn from(t:f32) -> Data {
        Data::Number(t)
    }
}



#[derive(Debug,Clone,Eq,Hash,PartialEq)]
pub enum Selector {
    Always,
    Class  (String),
    And    (Box<Selector>,Box<Selector>),
    Nested (Box<Selector>,Box<Selector>)
}



impl From<&str> for Selector {
    fn from(s:&str) -> Self {
        s.split(".")
            .map(|s| Self::Class(s.into()))
            .fold(Self::Always, |t,s| Self::Nested(Box::new(t),Box::new(s)))
    }
}




#[derive(Debug,Clone)]
pub enum Source {
    Resolved {
        selector : Selector,
        style    : WeakStyle
    },
    Unresolved {
        selector : Selector
    }
}



// =============
// === Style ===
// =============

#[derive(Debug,Clone)]
pub struct Style {
    rc : Rc<StyleData>
}

#[derive(Debug,Clone)]
pub struct WeakStyle {
    rc : Weak<StyleData>
}

pub struct StyleData {
    sheet   : WeakStyleSheet,
    data    : RefCell<Data>,
    sources : RefCell<Vec<Source>>,
    targets : RefCell<Vec<WeakStyle>>,
    func    : RefCell<Box<dyn Fn(&Data) -> Data>>
}

impl Style {
    pub fn new<S,D,F>(sheet:S, data:D, func:F) -> Self
    where S:Into<WeakStyleSheet>, D:Into<Data>, F:'static+Fn(&Data)->Data {
        let sheet   = sheet.into();
        let data    = RefCell::new(data.into());
        let func    = RefCell::new(Box::new(func));
        let sources = default();
        let targets = default();
        let rc      = Rc::new(StyleData {sheet,data,sources,targets,func});
        Self {rc}
    }
}

impl Debug for StyleData {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:?}",self.data)
    }
}

#[derive(Debug,Clone,CloneRef,Default,Deref)]
pub struct StyleSheet {
    rc : Rc<StyleSheetData>
}

#[derive(Debug,Clone,CloneRef,Default)]
pub struct WeakStyleSheet {
    rc : Weak<StyleSheetData>
}

#[derive(Debug,Clone,Default)]
pub struct StyleSheetData {
    pub styles : RefCell<HashMap<Selector,Style>>,
}

impl StyleSheet {
    pub fn new() -> Self {
        default()
    }

    pub fn downgrade(&self) -> WeakStyleSheet {
        let rc = Rc::downgrade(&self.rc);
        WeakStyleSheet {rc}
    }

    pub fn insert_var<T,D>(&self, selector:T, data:D) where
        T:Into<Selector>, D:Into<Data> {
        let selector = selector.into();
        let data     = data.into();
        let style    = Style::new(self.downgrade(),data,|x|{x.clone()});
        self.styles.borrow_mut().insert(selector,style);
    }

//    pub fn insert_var<T,D>(&self, selector:T, data:D) where
//        T:Into<Selector>, D:Into<Data> {
//        let selector = selector.into();
//        let data     = data.into();
//        let style    = Style::new(self.downgrade(),data,|x|{x.clone()});
//        self.styles.borrow_mut().insert(selector,style);
//    }
}



pub fn test() {
    let style_sheet = StyleSheet::new();
    style_sheet.insert_var("text.size", data(2.0));
    style_sheet.insert_expr("text.size2", data(2.0));
    println!("Hello world");
}




// style: text.color -> ...
// query: button.text.color