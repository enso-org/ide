
use crate::prelude::*;
use crate::data::OptVec;
use crate::data::HashMapTree;

use std::fmt::Write;



// ============
// === Data ===
// ============

#[derive(Debug,Clone,PartialEq)]
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

impl Mul<&Data> for &Data {
    type Output = Data;
    fn mul(self, rhs:&Data) -> Self::Output {
        match(self,rhs) {
            (Data::Invalid(t),_) => Data::Invalid(t.clone()),
            (_,Data::Invalid(t)) => Data::Invalid(t.clone()),
            (Data::Number(lhs),Data::Number(rhs)) => Data::Number(lhs*rhs),
            _ => Data::Invalid("Cannot multiply.".into())
        }
    }
}

impl Add<&Data> for &Data {
    type Output = Data;
    fn add(self, rhs:&Data) -> Self::Output {
        match(self,rhs) {
            (Data::Invalid(t),_) => Data::Invalid(t.clone()),
            (_,Data::Invalid(t)) => Data::Invalid(t.clone()),
            (Data::Number(lhs),Data::Number(rhs)) => Data::Number(lhs+rhs),
            _ => Data::Invalid("Cannot multiply.".into())
        }
    }
}

impl Eq for Data {}




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

pub type VarMap   = HashMapTree<String,Option<VarId>>;
pub type SheetMap = HashMapTree<String,SheetId>;

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
    fn set_value_to<P:Into<Path>>(&mut self, path:P, data:Option<Data>) {
        let path = path.into();
        self.remove_expr(&path);
        let sheet_id = self.sheet(&path);
        let sheet    = &mut self.sheets.vec[*sheet_id];
        sheet.value  = data;
        for var_id in sheet.matches.clone() {
            self.rebind_var(var_id)
        }

        for sheet_id in self.sheet_topo_sort(sheet_id) {
            self.recompute(sheet_id);
        }
    }

    fn set_value<P:Into<Path>>(&mut self, path:P, data:Data) {
        self.set_value_to(path,Some(data))
    }

    fn remove_value<P:Into<Path>>(&mut self, path:P) {
        self.set_value_to(path,None)
    }
}

impl Style {
    pub fn new() -> Self {
        let vars          = default();
        let mut sheets    = SheetRegistry::default();
        let var_map       = default();
        let root_sheet_id = sheets.new_instance();
        let sheet_map     = SheetMap::from_value(root_sheet_id);
        Self {vars,sheets,var_map,sheet_map}
    }

    pub fn var<P:Into<Path>>(&mut self, path:P) -> VarId {
        let path = path.into();
        let var_map_node = self.var_map.focus(&path.segments);
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
        let sheet_map_node  = self.sheet_map.focus_map_with(&path.segments,|| {sheets.new_instance()},
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

    fn sheet<P:Into<Path>>(&mut self, path:P) -> SheetId {
        let path   = path.into();
        let sheets = &mut self.sheets;
        let node   = self.sheet_map.focus_with(&path.segments,|| sheets.new_instance());
        node.value
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

    fn set_expr<P,F>(&mut self, path:P, sources:&[VarId], function:F)
    where P:Into<Path>, F:'static+Fn(&[&Data])->Data {
        let sheet_id = self.sheet(path);
        let sheet    = &mut self.sheets.vec[*sheet_id];

        for var_id in sources {
            self.vars.vec[**var_id].usages.insert(sheet_id);
        }
        let sources  = sources.iter().cloned().collect();
        let function = Box::new(function);
        sheet.expr   = Some(Expr {sources,function});

        self.recompute(sheet_id);

        let sheet = &mut self.sheets.vec[*sheet_id];
        for var_id in sheet.matches.clone() {
            self.rebind_var(var_id)
        }

        for sheet_id in self.sheet_topo_sort(sheet_id) {
            self.recompute(sheet_id);
        }
    }

    fn remove_expr<P>(&mut self, path:P)
    where P:Into<Path> {
        let sheet_id = self.sheet(path);
        let sheet    = &mut self.sheets.vec[*sheet_id];

        if sheet.expr.is_some() {
            sheet.value = None;

            let expr = mem::take(&mut sheet.expr);
            expr.for_each(|expr| {
                for var_id in expr.sources {
                    self.vars.vec[*var_id].usages.remove(&sheet_id);
                }
            });

            self.recompute(sheet_id);

            let sheet = &mut self.sheets.vec[*sheet_id];
            for var_id in sheet.matches.clone() {
                self.rebind_var(var_id)
            }

            for sheet_id in self.sheet_topo_sort(sheet_id) {
                self.recompute(sheet_id);
            }
        }
    }

    fn recompute(&mut self, sheet_id:SheetId) {
        let sheet = &self.sheets.vec[*sheet_id];
        let value = sheet.expr.as_ref().and_then(|expr| {
            let mut opt_values : Vec<Option<&Data>> = Vec::new();
            for var_id in &expr.sources {
                opt_values.push(self.value(*var_id));
            }
            let values : Option<Vec<&Data>> = opt_values.into_iter().collect();
            values.map(|v| (expr.function)(&v) )
        });
        let sheet_mut = &mut self.sheets.vec[*sheet_id];
        value.for_each(|v| sheet_mut.value = Some(v));
    }

    fn sheet_topo_sort(&self, changed_sheet_id:SheetId) -> Vec<SheetId> {
        let mut sheet_ref_count = HashMap::<SheetId,usize>::new();
        let mut sorted_sheets   = vec![changed_sheet_id];
        self.with_all_sheet_deps(changed_sheet_id, |sheet_id| {
            *sheet_ref_count.entry(sheet_id).or_default() += 1;
        });
        self.with_all_sheet_deps(changed_sheet_id, |sheet_id| {
            let ref_count = sheet_ref_count.entry(sheet_id).or_default();
            *ref_count -= 1;
            if *ref_count == 0 {
                sorted_sheets.push(sheet_id);
            }
        });
        sorted_sheets
    }

    fn with_all_sheet_deps<F>(&self, target:SheetId, mut callback:F)
    where F:FnMut(SheetId) {
        let mut sheets_to_visit = vec![target];
        loop {
            match sheets_to_visit.pop() {
                None => break,
                Some(current_sheet_id) => {
                    let sheet = &self.sheets.vec[*current_sheet_id];
                    for var_id in &sheet.bindings {
                        let var = &self.vars.vec[**var_id];
                        for sheet_id in &var.usages {
                            callback(*sheet_id);
                            sheets_to_visit.push(*sheet_id);
                        }
                    }
                }
            }
        }
    }

    fn visualize(&self) -> String {
        let mut dot = String::new();
        Self::visualize_sheet_map(&mut dot,&self.sheet_map);
        Self::visualize_var_map(&mut dot,&mut vec![],&self.var_map);
        for var in &self.vars.vec {
            for var_match in &var.matches {
                writeln!(dot,"var_{} -> sheet_{} [style=dashed]",var.id,var_match).ok();
            }
            var.binding.for_each(|sheet_id| {
                writeln!(dot,"var_{} -> sheet_{} [color=red]",var.id,sheet_id).ok();
            });
            for sheet_id in &var.usages {
                writeln!(dot,"var_{} -> sheet_{} [color=blue]",var.id,sheet_id).ok();
            }
        }

        for sheet in &self.sheets.vec {
            for sheet_match in &sheet.matches {
                writeln!(dot,"sheet_{} -> var_{} [style=dashed]",sheet.id,sheet_match).ok();
            }

            for var_id in &sheet.bindings {
                writeln!(dot,"sheet_{} -> var_{} [color=red]",sheet.id,var_id).ok();
            }

            sheet.expr.for_each_ref(|expr| {
                for var_id in &expr.sources {
                    writeln!(dot,"sheet_{} -> var_{} [color=blue]",sheet.id,var_id).ok();
                }
            })
        }
        dot
    }

    fn visualize_sheet_map(dot:&mut String, sheet_map:&SheetMap) {
        let sheet_id = sheet_map.value;
        writeln!(dot,"sheet_{}",sheet_id).ok();
        for (path,child) in sheet_map {
            writeln!(dot,"sheet_{} -> sheet_{} [label=\"{}\"]",sheet_id,child.value,path).ok();
            Self::visualize_sheet_map(dot,child);
        }
    }

    fn visualize_var_map(dot:&mut String, path:&mut Vec<String>, var_map:&VarMap) {
        var_map.value.for_each(|var_id| {
            let real_path = path.iter().rev().join(".");
            writeln!(dot,"var_{} [label=\"Var({})\"]",var_id,real_path).ok();
        });
        for (segment,child) in var_map {
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


#[derive(Clone,Debug)]
pub struct Path {
    pub segments : Vec<String>
}

impl Path {
    pub fn from_segments<'t,T,I,Item>(t:T) -> Self
    where T : IntoIterator<IntoIter=I,Item=&'t Item>,
          I : Iterator<Item=&'t Item>,
          I : std::iter::DoubleEndedIterator,
          Item : 't + Copy + Into<String> {
        let segments = t.into_iter().map(|s|(*s).into()).rev().collect();
        Self {segments}
    }
}

macro_rules! gen_var_path_conversions {
    ($($num:tt),*) => {$(
        impl<T> From<&[T;$num]> for Path
        where T : Copy + Into<String> {
            fn from(t:&[T;$num]) -> Self {
                Self::from_segments(t)
            }
        }
    )*};
}

impl<T> From<&[T]> for Path
    where T : Copy + Into<String> {
    fn from(t:&[T]) -> Self {
        Self::from_segments(t)
    }
}

gen_var_path_conversions!(1,2,3,4,5,6,7,8,9,10);

impl From<&str> for Path {
    fn from(t:&str) -> Self {
        Self::from_segments(&t.split(".").collect::<Vec<_>>())
    }
}

impl From<&Path> for Path {
    fn from(t:&Path) -> Self {
        t.clone()
    }
}


impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}



pub fn test() {

    let mut style = Style::new();

    let var_size              = style.var("size");
    let var_button_size       = style.var("button.size");
    let var_graph_button_size = style.var("graph.button.size");

    assert!(style.value(var_graph_button_size).is_none());
    style.set_value("size",data(1.0));
    style.set_expr("graph.button.size",&[var_button_size],|args| args[0] + &data(100.0));
    style.set_expr("button.size",&[var_size],|args| args[0] + &data(10.0));
    style.set_value("button.size",data(3.0));


    println!("{}",style.visualize());
    println!("{:?}", style.value(var_graph_button_size));
    println!("{:?}", style.value(var_button_size));
    println!("{:?}", style.vars.vec[*var_graph_button_size]);
    println!("{:?}", style.sheets.vec[*style.vars.vec[*var_graph_button_size].binding.unwrap()]);

    tests::simple_var_binding_1();
    tests::simple_var_binding_2();
    tests::hierarchical_var_binding();
    tests::expr_bindings_1();
    tests::expr_circular();
}





//// #[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    pub fn simple_var_binding_1() {
        let mut style = Style::new();
        let var1      = style.var(&["size"]);
        assert!(style.value(var1).is_none());
        style.set_value(&["size"],data(1.0));
        assert_eq!(style.value(var1),Some(&data(1.0)));
    }

    // #[test]
    pub fn simple_var_binding_2() {
        let mut style = Style::new();
        style.set_value(&["size"],data(1.0));
        let var1 = style.var(&["size"]);
        assert_eq!(style.value(var1),Some(&data(1.0)));
    }

    // #[test]
    pub fn hierarchical_var_binding() {
        let mut style = Style::new();
        let var1      = style.var("graph.button.size");
        assert!(style.value(var1).is_none());
        style.set_value("size",data(1.0));
        assert_eq!(style.value(var1),Some(&data(1.0)));
        style.set_value("button.size",data(2.0));
        assert_eq!(style.value(var1),Some(&data(2.0)));
        style.set_value("graph.button.size",data(3.0));
        assert_eq!(style.value(var1),Some(&data(3.0)));
        style.remove_value("graph.button.size");
        assert_eq!(style.value(var1),Some(&data(2.0)));
        style.remove_value("button.size");
        assert_eq!(style.value(var1),Some(&data(1.0)));
        style.remove_value("size");
        assert_eq!(style.value(var1),None);
    }

    // #[test]
    pub fn expr_bindings_1() {
        let mut style = Style::new();

        let var_size              = style.var("size");
        let var_button_size       = style.var("button.size");
        let var_graph_button_size = style.var("graph.button.size");

        assert!(style.value(var_graph_button_size).is_none());
        style.set_value("size",data(1.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(1.0)));
        style.set_expr("graph.button.size",&[var_button_size],|args| args[0] + &data(10.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(11.0)));
        style.set_expr("button.size",&[var_size],|args| args[0] + &data(100.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(111.0)));
        style.set_value("size",data(2.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(112.0)));
        style.set_value("button.size",data(3.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(13.0)));
        style.set_value("button.size",data(4.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(14.0)));
    }

    pub fn expr_circular() {
        let mut style = Style::new();

        let var_a = style.var("a");
        let var_b = style.var("b");

        style.set_expr("a",&[var_b],|args| args[0].clone());
        style.set_expr("b",&[var_a],|args| args[0].clone());
        assert!(style.value(var_a).is_none());
    }
}
