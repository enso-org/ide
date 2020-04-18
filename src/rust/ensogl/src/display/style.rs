
use crate::prelude::*;

use crate::frp;



//// ============
//// === Data ===
//// ============
//
//#[derive(Debug,Clone)]
//pub enum Data {
//    Invalid(String),
//    Number(f32)
//}
//
//pub fn data<T:Into<Data>>(t:T) -> Data {
//    t.into()
//}
//
//impl From<f32> for Data {
//    fn from(t:f32) -> Data {
//        Data::Number(t)
//    }
//}



//#[derive(Debug,Clone,Eq,Hash,PartialEq)]
//pub enum Selector {
//    Always,
//    Class  (String),
//    And    (Box<Selector>,Box<Selector>),
//    Nested (Box<Selector>,Box<Selector>)
//}
//
//
//
//impl From<&str> for Selector {
//    fn from(s:&str) -> Self {
//        s.split(".")
//            .map(|s| Self::Class(s.into()))
//            .fold(Self::Always, |t,s| Self::Nested(Box::new(t),Box::new(s)))
//    }
//}
//
//
//
//
//#[derive(Debug,Clone)]
//pub enum Source {
//    Resolved {
//        selector : Selector,
//        style    : WeakStyle
//    },
//    Unresolved {
//        selector : Selector
//    }
//}
//
//
//
//// =============
//// === Style ===
//// =============
//
//#[derive(Debug,Clone)]
//pub struct Style {
//    rc : Rc<StyleData>
//}
//
//#[derive(Debug,Clone)]
//pub struct WeakStyle {
//    rc : Weak<StyleData>
//}
//
//pub struct StyleData {
//    sheet   : WeakStyleSheet,
//    data    : RefCell<Data>,
//    sources : RefCell<Vec<Source>>,
//    targets : RefCell<Vec<WeakStyle>>,
//    func    : RefCell<Box<dyn Fn(&Data) -> Data>>
//}
//
//impl Style {
//    pub fn new<S,D,F>(sheet:S, data:D, func:F) -> Self
//    where S:Into<WeakStyleSheet>, D:Into<Data>, F:'static+Fn(&Data)->Data {
//        let sheet   = sheet.into();
//        let data    = RefCell::new(data.into());
//        let func    = RefCell::new(Box::new(func));
//        let sources = default();
//        let targets = default();
//        let rc      = Rc::new(StyleData {sheet,data,sources,targets,func});
//        Self {rc}
//    }
//}
//
//impl Debug for StyleData {
//    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
//        write!(f,"{:?}",self.data)
//    }
//}
//
//#[derive(Debug,Clone,CloneRef,Default,Deref)]
//pub struct StyleSheet {
//    rc : Rc<StyleSheetData>
//}
//
//#[derive(Debug,Clone,CloneRef,Default)]
//pub struct WeakStyleSheet {
//    rc : Weak<StyleSheetData>
//}
//
//#[derive(Debug,Clone,Default)]
//pub struct StyleSheetData {
//    pub styles : RefCell<HashMap<Selector,Style>>,
//}
//
//impl StyleSheet {
//    pub fn new() -> Self {
//        default()
//    }
//
//    pub fn downgrade(&self) -> WeakStyleSheet {
//        let rc = Rc::downgrade(&self.rc);
//        WeakStyleSheet {rc}
//    }
//
//    pub fn insert_var<T,D>(&self, selector:T, data:D) where
//        T:Into<Selector>, D:Into<Data> {
//        let selector = selector.into();
//        let data     = data.into();
//        let style    = Style::new(self.downgrade(),data,|x|{x.clone()});
//        self.styles.borrow_mut().insert(selector,style);
//    }
//
////    pub fn insert_var<T,D>(&self, selector:T, data:D) where
////        T:Into<Selector>, D:Into<Data> {
////        let selector = selector.into();
////        let data     = data.into();
////        let style    = Style::new(self.downgrade(),data,|x|{x.clone()});
////        self.styles.borrow_mut().insert(selector,style);
////    }
//}


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

#[derive(Clone,CloneRef,Debug,Deref)]
pub struct Style {
    rc : Rc<StyleData>
}

#[derive(Debug)]
pub enum StyleData {
    Value(Data)
}

impl From<Data> for Style {
    fn from(t:Data) -> Self {
        let rc = Rc::new(StyleData::Value(t));
        Self {rc}
    }
}


#[derive(Clone,CloneRef,Debug,Default,Deref)]
pub struct StyleSheet {
    rc : Rc<StyleSheetData>
}

#[derive(Clone,CloneRef,Debug,Default,Deref)]
pub struct WeakStyleSheet {
    rc : Weak<StyleSheetData>
}

impl WeakStyleSheet {
    pub fn upgrade(&self) -> Option<StyleSheet> {
        self.rc.upgrade().map(|rc| StyleSheet {rc})
    }
}

#[derive(Debug,Default)]
pub struct StyleSheetData {
    value    : Rc<RefCell<Option<Style>>>,
    bindings : RefCell<Vec<WeakVar>>,
    children : RefCell<HashMap<String,StyleSheet>>
}

impl StyleSheet {
    pub fn var(&self, path:&[&str]) -> Var {
        let var          = Var::default();
        let weak_var     = var.downgrade();
        let mut sub_path = path;
        let mut sources  = Vec::new();
        loop {
            if sub_path.len() < 1 { break }
            let source = self.register_reader(sub_path,&weak_var);
            sources.push(source);
            sub_path = &sub_path[1..];
        }
        var.set_styles(sources);
        var
    }

    fn register_reader(&self, path:&[&str], var:&WeakVar) -> WeakStyleSheet {
        match path {
            [] => {
                self.bindings.borrow_mut().push(var.clone_ref());
                self.downgrade()
            },
            [head,tail @ ..] => {
                self.children
                    .borrow_mut()
                    .entry((*head).into())
                    .or_default()
                    .register_reader(tail,var)
            }
        }
    }

    pub fn set_style<S:Into<Style>>(&self, path:&[&str], style:S) {
        match path {
            [] => {
                let new_set = self.value.borrow().is_none();
                *self.value.borrow_mut() = Some(style.into());
                if new_set {
                    for weak_var in &*self.bindings.borrow() {
                        weak_var.upgrade().for_each(|var| var.rebind());
                    }
                }
            },
            [head,tail @ ..] => {
                self.children
                    .borrow_mut()
                    .entry((*head).into())
                    .or_default()
                    .set_style(tail,style)
            }
        }
    }

    pub fn downgrade(&self) -> WeakStyleSheet {
        let rc = Rc::downgrade(&self.rc);
        WeakStyleSheet {rc}
    }
}

#[derive(Debug,Clone,CloneRef,Default)]
pub struct WeakVar {
    rc : Weak<VarData>
}

impl WeakVar {
    pub fn upgrade(&self) -> Option<Var> {
        self.rc.upgrade().map(|rc| Var {rc})
    }
}

#[derive(Debug,Clone,CloneRef,Default,Deref)]
pub struct Var {
    rc : Rc<VarData>
}

impl Var {
    pub fn downgrade(&self) -> WeakVar {
        let rc = Rc::downgrade(&self.rc);
        WeakVar {rc}
    }
}

#[derive(Debug,Default)]
pub struct VarData {
    value  : RefCell<Rc<RefCell<Option<Style>>>>,
    styles : RefCell<Vec<WeakStyleSheet>>,
}

impl VarData {
    fn set_styles(&self, styles:Vec<WeakStyleSheet>) {
        *self.styles.borrow_mut() = styles;
        self.rebind();
    }

    fn rebind(&self) {
        for weak_style in &*self.styles.borrow() {
            if let Some(style) = weak_style.upgrade() {
                if style.value.borrow().is_some() {
                    *self.value.borrow_mut() = style.value.clone_ref();
                    break
                }
            }
        }
    }
}




pub fn test() {
    let style_sheet = StyleSheet::default();
    let var1 = style_sheet.var(&["button","text","size"]);

    println!("{:#?}", var1);
    println!("------------");
    style_sheet.set_style(&["text","size"],data(2.0));
    println!("{:#?}", var1);
//    style_sheet.insert_var("text.size", data(2.0));
//    style_sheet.insert_expr("text.size2", data(2.0));
    println!("Hello world");
}




// style: text.color -> ...
// query: button.text.color