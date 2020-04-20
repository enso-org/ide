//
//use crate::prelude::*;
//use crate::data::OptVec;
//
//use std::fmt::Write;
//
//
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
//
//
//
//// ===========
//// === Var ===
//// ===========
//
//#[derive(Debug,Default)]
//pub struct Var {
//    matches : HashSet<SheetId>,
//    binding : Option<SheetId>,
//    users   : HashSet<SheetId>,
//}
//
//
//
//// ===========
//// === Map ===
//// ===========
//
//#[derive(Debug)]
//pub struct Map {
//    var_id   : Option<VarId>,
//    sheet_id : SheetId,
//    children : HashMap<String,Map>,
//}
//
//impl Map {
//    pub fn new(sheet_id:SheetId) -> Self {
//        let var_id   = default();
//        let children = default();
//        Self {var_id,sheet_id,children}
//    }
//
//    pub fn visualize(&self, dot:&mut String) {
//        writeln!(dot,"sheet_{}",self.sheet_id);
//        for (path,map) in &self.children {
//            writeln!(dot,"sheet_{} -> sheet_{} [label=\"{}\"]",self.sheet_id,map.sheet_id,path);
//            map.visualize(dot);
//        }
//    }
//
//    pub fn get_var(&self, path:&[&str]) -> Option<VarId> {
//        let mut map = self;
//        loop {
//            match path {
//                [] => break map.var_id,
//                [head, tail@..] => {
//                    match map.children.get(*head) {
//                        None    => break None,
//                        Some(m) => map = m
//                    }
//                }
//            }
//        }
//    }
//}
//
//
//
//// =============
//// === Style ===
//// =============
//
//#[derive(Clone,Copy,Debug,Deref,Eq,From,Hash,Into,PartialEq)]
//pub struct VarId(usize);
//
//#[derive(Clone,Copy,Debug,Deref,Eq,From,Hash,Into,PartialEq)]
//pub struct SheetId(usize);
//
//impl Display for SheetId {
//    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
//        write!(f,"{}",self.0)
//    }
//}
//
//impl Display for VarId {
//    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
//        write!(f,"{}",self.0)
//    }
//}
//
//#[derive(Debug)]
//pub struct Style {
//    sheets : OptVec<Sheet>,
//    vars   : OptVec<Var>,
//    map    : Map,
//}
//
//
//#[derive(Derivative)]
//#[derivative(Debug)]
//pub struct Sheet {
//    id       : SheetId,
//    value    : Option<Data>,
//    matches  : HashSet<VarId>,
//    bindings : HashSet<VarId>,
//    deps : Vec<VarId>,
//    #[derivative(Debug="ignore")]
//    expr : Option<Box<dyn Fn(&[Option<Data>])->Data>>
//}
//
//impl Sheet {
//    pub fn new(id:SheetId) -> Self {
//        let value = default();
//        let matches = default();
//        let bindings = default();
//        let deps = default();
//        let expr = default();
//        Self {id,value,matches,bindings,deps,expr}
//    }
//}
//
//
//impl Style {
//    pub fn new() -> Self {
//        let mut sheets    = OptVec::<Sheet>::new();
//        let vars          = OptVec::<Var>::new();
//        let root_sheet_id = sheets.insert_with_ix(|ix| Sheet::new(ix.into())).into();
//        let map           = Map::new(root_sheet_id);
//        Self {sheets,vars,map}
//    }
//
//
//    pub fn var2(&self, path:&[&str]) -> VarId {
//        let mut map = self;
//        loop {
//            match path {
//                [] => break map.var_id,
//                [head, tail@..] => {
//                    match map.children.get(*head) {
//                        None    => break None,
//                        Some(m) => map = m
//                    }
//                }
//            }
//        }
//    }
//
//    pub fn var(&mut self, path:&[&str]) -> VarId {
//        let t1 = &mut self.map;
//        let t2 = &mut self.vars;
//        let mut sub_path    = path;
//        let mut var_matches = HashSet::default();
//        let var_id          = VarId(self.vars.insert(default()));
//        loop {
//            match sub_path {
//                [] => break,
//                [head,tail@..] => {
//                    sub_path = tail;
//                    let sheet_id = self.register_var(head,tail,var_id);
//                    var_matches.insert(sheet_id);
//                }
//            }
//        }
//        self.vars[*var_id].matches = var_matches;
//        var_id
//    }
//
//
//    pub fn set_value(&mut self, path:&[&str], value:Data) {
//        match path {
//            [] => todo!(),
//            [head, tail@..] => {
//                let sheet_id  = self.get_nested_sheet_id(head,tail);
//                let sheet_mut = &mut self.sheets[*sheet_id];
//                let new_set   = sheet_mut.value.is_none();
//                sheet_mut.value = Some(value);
//                let sheet = &self.sheets[*sheet_id];
//
//                let matches = sheet.matches.clone();
//
//                if new_set {
//                    for var_id in matches {
//                        // println!("Rebinding : {:?}", var_id);
//                        let var = &mut self.vars[*var_id];
//                        for var_match in &var.matches {
//                            if self.sheets[**var_match].value.is_some() {
//                                self.sheets[**var_match].bindings.insert(var_id);
//                                var.binding = Some(*var_match);
//                                break
//                            }
//                        }
//                    }
//                }
//
//                let mut sheets_to_update = vec![sheet_id];
//
//                loop {
//                    match sheets_to_update.pop() {
//                        None => break,
//                        Some(sheet_id) => {
//                            let sheet = &self.sheets[*sheet_id];
//                            if let Some(f) = &sheet.expr {
//                                let args : Vec<Option<Data>> = sheet.deps.iter().map(|dep| self.value(*dep)).collect();
//                                println!("UPDATING DATA")
//
//                            }
//                            for var_id in &sheet.bindings {
//                                let users = &self.vars[**var_id].users;
//                                for user in users {
//                                    sheets_to_update.push(*user);
//                                }
//                                println!("Users of [{:?}]: {:?}", var_id, users);
//                            }
//                        }
//                    }
//                }
//            }
//        }
//    }
//
////    css.set_expr(&["text","size"],&[&["size"]],|t| t[0]);
//
//    pub fn set_expr<F>(&mut self, path:&[&str], deps:&[VarId], f:F)
//    where F:'static+Fn(&[Option<Data>])->Data {
//        match path {
//            [] => todo!(),
//            [head, tail@..] => {
//                let sheet_id  = self.get_nested_sheet_id(head,tail);
//                let sheet_mut = &mut self.sheets[*sheet_id];
//                for var_id in deps {
//                    self.vars[**var_id].users.insert(sheet_id);
//                }
//                sheet_mut.deps = deps.into();
//                sheet_mut.expr = Some(Box::new(f));
//            }
//        }
//    }
//
//    fn register_var(&mut self, path_head:&str, path_tail:&[&str], var_id:VarId) -> SheetId {
//        let sheet_id = self.get_nested_sheet_id(path_head, path_tail);
//        self.sheets[*sheet_id].matches.insert(var_id);
//        sheet_id
//    }
//
//
//    fn get_nested_sheet_id(&mut self, path_head:&str, path_tail:&[&str]) -> SheetId {
//        todo!()
////        let     sheets   = &mut self.sheets;
////        let mut sub_path = path_tail;
////        let mut map      = Self::insert_sub_map_if_missing(sheets,&mut self.map,path_head);
////
////        loop {
////            match sub_path {
////                [] => break map.sheet_id,
////                [head, tail@..] => {
////                    sub_path = tail;
////                    map      = Self::insert_sub_map_if_missing(sheets,&mut map.children,*head);
////                }
////            }
////        }
//    }
//
//    fn insert_sub_map_if_missing<'t>
//    (sheets:&mut OptVec<Sheet>, map:&'t mut HashMap<String,Map>, item:&str) -> &'t mut Map {
//        let missing = map.get(item).is_none();
//        if missing {
//            let sheet_id = sheets.insert_with_ix(|ix| Sheet::new(ix.into())).into();
//            map.insert(item.into(), Map::new(sheet_id));
//        }
//        map.get_mut(item).unwrap()
//    }
//
//    pub fn value(&self, var_id:VarId) -> Option<Data> {
//        self.vars[*var_id].binding.and_then(
//            |sheet_id| self.sheets[*sheet_id].value.as_ref().cloned()
//        )
//    }
//
//
//    pub fn vizualize(&self) -> String {
//        let mut dot = String::new();
//        self.map.visualize(&mut dot);
//        for sheet in &self.sheets {
//            let sheet_id = sheet.id;
//            if sheet.value.is_some() {
//                writeln!(dot,"sheet_value_{} [label=\"Value\"]",sheet_id);
//                writeln!(dot,"sheet_{} -> sheet_value_{}",sheet_id,sheet_id);
//            }
//            for var_id in &sheet.matches {
//                writeln!(dot,"sheet_{} -> var_{} [style=dashed weight=0]",sheet_id,var_id);
//            }
//
//            for var_id in &sheet.bindings {
//                writeln!(dot,"sheet_{} -> var_{} [color=red weight=0]",sheet_id,var_id);
//            }
//        }
//        dot
//    }
//
//}
//
//
//
//pub fn test() {
//    let mut css = Style::new();
//    let var1 = css.var(&["button","text","size"]);
////    css.set_expr(&["text","size"],&[var1],|t| t[0].as_ref().unwrap().clone());
////
////
////    css.set_value(&["size"],data(2.0));
//
//
//    println!("{}", css.vizualize());
//
//}
//
//
////text {
////    size: 15
////}
////
////graph_editor {
////    text {
////        size: 18
////    }
////}
////button.text.size