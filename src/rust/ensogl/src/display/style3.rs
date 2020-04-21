
use crate::prelude::*;
use crate::data::OptVec;

use std::fmt::Write;


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





// =======================
// === HierarchicalMap ===
// =======================

#[derive(Derivative)]
#[derivative(Debug   (bound="K:Eq+Hash+Debug , T:Debug"))]
#[derivative(Default (bound="K:Eq+Hash       , T:Default"))]
pub struct HierarchicalMap<K,T> {
    value    : T,
    children : HashMap<K,HierarchicalMap<K,T>>
}

impl<K,T> HierarchicalMap<K,T>
where K:Eq+Hash {
    pub fn new_with(value:T) -> Self {
        let children = default();
        Self {value,children}
    }

    pub fn focus<P,I>(&mut self, path:P) -> &mut HierarchicalMap<K,T>
    where P:IntoIterator<Item=I>, T:Default, I:Into<K> {
        self.focus_with(path,default)
    }


    pub fn focus_with<P,I,F>(&mut self, path:P, mut f:F) -> &mut HierarchicalMap<K,T>
    where P:IntoIterator<Item=I>, I:Into<K>, F:FnMut()->T {
        path.into_iter().fold(self,|map,t| {
            map.children.entry(t.into()).or_insert_with(|| HierarchicalMap::new_with(f()))
        })
    }

    pub fn focus_map_with<P,I,F,M>(&mut self, path:P, mut f:F, mut m:M) -> &mut HierarchicalMap<K,T>
    where P:IntoIterator<Item=I>, I:Into<K>, F:FnMut()->T, M:FnMut(&mut HierarchicalMap<K,T>) {
        path.into_iter().fold(self,|map,t| {
            let t = map.children.entry(t.into()).or_insert_with(|| HierarchicalMap::new_with(f()));
            m(t);
            t
        })
    }
}



// ==========
// === Id ===
// ==========

#[derive(Clone,Copy,Debug,Deref,Eq,From,Hash,Into,PartialEq)]
pub struct VarId(usize);

#[derive(Clone,Copy,Debug,Deref,Eq,From,Hash,Into,PartialEq)]
pub struct SheetId(usize);

impl Display for SheetId {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

impl Display for VarId {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}



// ===========
// === Var ===
// ===========

#[derive(Debug)]
pub struct Var {
    id      : VarId,
    matches : Vec<SheetId>,
    binding : Option<SheetId>,
    usages  : HashSet<SheetId>,
}

impl Var {
    pub fn new(id:VarId) -> Self {
        let matches = default();
        let binding = default();
        let usages  = default();
        Self {id,matches,binding,usages}
    }
}



// ============
// === Expr ===
// ============

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Expr {
    sources : Vec<VarId>,
    #[derivative(Debug="ignore")]
    function : Box<dyn Fn(&[&Data])->Data>
}


// =============
// === Sheet ===
// =============

#[derive(Debug)]
pub struct Sheet {
    id       : SheetId,
    value    : Option<Data>,
    expr     : Option<Expr>,
    matches  : HashSet<VarId>,
    bindings : HashSet<VarId>,
}

impl Sheet {
    pub fn new(id:SheetId) -> Self {
        let value    = default();
        let expr     = default();
        let matches  = default();
        let bindings = default();
        Self {id,value,expr,matches,bindings}
    }
}



// =============
// === Style ===
// =============

pub type VarMap   = HierarchicalMap<String,Option<VarId>>;
pub type SheetMap = HierarchicalMap<String,SheetId>;

#[derive(Debug,Default)]
pub struct VarRegistry {
    vec : OptVec<Var>
}

impl VarRegistry {
    pub fn new_instance(&mut self) -> VarId {
        self.vec.insert_with_ix(|ix| Var::new(ix.into())).into()
    }
}


#[derive(Debug,Default)]
pub struct SheetRegistry {
    vec : OptVec<Sheet>
}

impl SheetRegistry {
    pub fn new_instance(&mut self) -> SheetId {
        self.vec.insert_with_ix(|ix| Sheet::new(ix.into())).into()
    }
}




#[derive(Debug)]
pub struct Style {
    vars      : VarRegistry,
    sheets    : SheetRegistry,
    var_map   : VarMap,
    sheet_map : SheetMap,
}

impl Style {
    pub fn new() -> Self {
        let vars          = default();
        let mut sheets    = SheetRegistry::default();
        let var_map       = default();
        let root_sheet_id = sheets.new_instance();
        let sheet_map     = SheetMap::new_with(root_sheet_id);
        Self {vars,sheets,var_map,sheet_map}
    }

    pub fn var(&mut self, path:&[String]) -> VarId {
        let path_rev     = || path.iter().rev();
        let var_map_node = self.var_map.focus(path_rev());
        let var_id = match var_map_node.value {
            Some(t) => t,
            None => {
                let var_id = self.vars.new_instance();
                var_map_node.value = Some(var_id);
                var_id
            }
        };

        let mut var_matches = Vec::new();
        let sheets          = &mut self.sheets;
        let sheet_map_node  = self.sheet_map.focus_map_with(path_rev(),|| {sheets.new_instance()},
            |node| var_matches.push(node.value)
        );
        var_matches.reverse();

        let sheet_id = sheet_map_node.value;

        for sheet_id in &var_matches {
            self.sheets.vec[**sheet_id].matches.insert(var_id);
        }

        self.vars.vec[*var_id].matches = var_matches;
        self.rebind_var(var_id);
        var_id
    }

    fn rebind_var(&mut self, var_id:VarId) {
        let mut done = false;
        let var      = &self.vars.vec[*var_id];
        for sheet_id in var.matches.clone() {
            let sheet = &self.sheets.vec[*sheet_id];
            if sheet.value.is_some() {
                var.binding.for_each(|sheet_id| {
                    self.sheets.vec[*sheet_id].bindings.remove(&var_id);
                });
                let var   = &mut self.vars.vec[*var_id];
                let sheet = &mut self.sheets.vec[*sheet_id];
                var.binding = Some(sheet_id);
                sheet.bindings.insert(var_id);
                done = true;
                break
            }
        }
        if !done {
            let var = &self.vars.vec[*var_id];
            var.binding.for_each(|sheet_id| {
                self.sheets.vec[*sheet_id].bindings.remove(&var_id);
            });
            let var = &mut self.vars.vec[*var_id];
            var.binding = None;
        }
    }

    fn set_value_to(&mut self, path:&[String], data:Option<Data>) {
        let sheets     = &mut self.sheets;
        let sheet_node = self.sheet_map.focus_with(path.iter().rev(),|| sheets.new_instance());
        let sheet_id   = sheet_node.value;
        let sheet      = &mut self.sheets.vec[*sheet_id];
        sheet.value    = data;
        for var_id in sheet.matches.clone() {
            self.rebind_var(var_id)
        }
    }

    fn set_value(&mut self, path:&[String], data:Data) {
        self.set_value_to(path,Some(data))
    }

    fn remove_value(&mut self, path:&[String]) {
        self.set_value_to(path,None)
    }


    fn set_expr<F>(&mut self, path:&[String], data:Option<Data>, sources:&[VarId], function:F)
    where F:'static+Fn(&[&Data])->Data {
        let sheets     = &mut self.sheets;
        let sheet_node = self.sheet_map.focus_with(path.iter().rev(),|| sheets.new_instance());
        let sheet_id   = sheet_node.value;
        let sheet      = &mut self.sheets.vec[*sheet_id];
        sheet.value    = data;


        for var_id in sources {
            self.vars.vec[**var_id].usages.insert(sheet_id);
        }
        let sources  = sources.iter().cloned().collect();
        let function = Box::new(function);
        sheet.expr   = Some(Expr {sources,function});

        for var_id in sheet.matches.clone() {
            self.rebind_var(var_id)
        }
    }

    fn visualize(&self) -> String {
        let mut dot = String::new();
        Self::visualize_sheet_map(&mut dot,&self.sheet_map);
        Self::visualize_var_map(&mut dot,&mut vec![],&self.var_map);
        for var in &self.vars.vec {
            for var_match in &var.matches {
                writeln!(dot,"var_{} -> sheet_{} [style=dashed]",var.id,var_match);
            }
            var.binding.for_each(|sheet_id| {
                writeln!(dot,"var_{} -> sheet_{} [color=red]",var.id,sheet_id);
            });
            for sheet_id in &var.usages {
                writeln!(dot,"var_{} -> sheet_{} [color=blue]",var.id,sheet_id);
            }
        }

        for sheet in &self.sheets.vec {
            for sheet_match in &sheet.matches {
                writeln!(dot,"sheet_{} -> var_{} [style=dashed]",sheet.id,sheet_match);
            }

            for var_id in &sheet.bindings {
                writeln!(dot,"sheet_{} -> var_{} [color=red]",sheet.id,var_id);
            }

            sheet.expr.for_each_ref(|expr| {
                for var_id in &expr.sources {
                    writeln!(dot,"sheet_{} -> var_{} [color=blue]",sheet.id,var_id);
                }
            })
        }
        dot
    }

    fn visualize_sheet_map(dot:&mut String, sheet_map:&SheetMap) {
        let sheet_id = sheet_map.value;
        writeln!(dot,"sheet_{}",sheet_id);
        for (path,child) in &sheet_map.children {
            writeln!(dot,"sheet_{} -> sheet_{} [label=\"{}\"]",sheet_id,child.value,path);
            Self::visualize_sheet_map(dot,child);
        }
    }

    fn visualize_var_map(dot:&mut String, path:&mut Vec<String>, var_map:&VarMap) {
        var_map.value.for_each(|var_id| {
            let real_path = path.iter().rev().join(".");
            writeln!(dot,"var_{} [label=\"Var({})\"]",var_id,real_path);
        });
        for (segment,child) in &var_map.children {
            path.push(segment.into());
            Self::visualize_var_map(dot,path,child);
            path.pop();
        }
    }

    pub fn value(&self, var_id:VarId) -> Option<&Data> {
        self.vars.vec.items.index(*var_id).as_ref().and_then(|var| {
            var.binding.and_then(|sheet_id| {
                self.sheets.vec[*sheet_id].value.as_ref()
            })
        })
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}



pub fn test() {
    let mut css = Style::default();
    let var0    = css.var(&["size".into()]);
    let var1    = css.var(&["text".into(),"size".into()]);
    let var2    = css.var(&["button".into(),"text".into(),"size".into()]);

    css.set_value(&["size".into()], data(1.0));

    css.set_expr(&["button".into(),"text".into(),"size".into()], Some(data(2.0)), &[var0], |t| t[0].clone());

    let var3    = css.var(&["circle".into(),"size".into()]);

    let var4    = css.var(&["graph".into(),"background".into(),"color".into()]);
    let var5    = css.var(&["cursor".into(),"movement".into(),"speed".into()]);
    let var6    = css.var(&["animation".into(),"speed".into()]);


//    println!("{:?}", css.value(var3));


//    css.remove_value(&["button".into(),"text".into(),"size".into()]);



    println!("{}",css.visualize());
}