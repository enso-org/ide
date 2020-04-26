//! This module defines a cascading style sheet registry and related style management utilities.

use crate::prelude::*;
use crate::data::HashMapTree;
use crate::data::Index;
use crate::data::OptVec;

pub use super::data::Data;
pub use super::data::data;
pub use super::path::Path;



// ===========
// === Var ===
// ===========

/// Data of a style variable. Variables are associated with a style path like 'panel.button.size'
/// and are automatically bound to the most specific style sheet as soon as it gets defined. By
/// most specific, we mean the one with the longest path. For example, the 'panel.button.size' var
/// will be bound to one of 'panel.button.size', 'button.size', or 'size' if defined, in that order.
#[derive(Debug)]
pub struct Var {
    /// Index of the var in the style var map.
    index : Index<Var>,
    /// Set of all `Sheet` indexes which are potential matches of this var. For example, for a var
    /// 'panel.button.size', all of the following sheets will be included here: 'panel.button.size',
    /// 'button.size', and 'size'.
    matches : Vec<Index<Sheet>>,
    /// Index of the most specific `Sheet` from `matches` which has a defined value if any.
    binding : Option<Index<Sheet>>,
    /// List of all `Sheet`s which use this var in their expressions.
    usages : HashSet<Index<Sheet>>,
}

impl Var {
    /// Constructor.
    pub fn new(index:Index<Var>) -> Self {
        let matches = default();
        let binding = default();
        let usages  = default();
        Self {index,matches,binding,usages}
    }
}



// =============
// === Sheet ===
// =============

/// A node in the style sheet tree. Style sheets are associated with a style path like
/// 'panel.button.size' and each node keeps a `Data` value. The value can either be set explicitly,
/// or computed automatically if the style sheet is defined with a `BoundExpression`. Please note
/// that although `Sheet` contains a single value, it is in fact a node in a tree defined in
/// `RegistryData`, so it can be interpreted as a set of hierarchical values instead.
#[derive(Debug)]
pub struct Sheet {
    /// Index of the style sheet in the style sheet map.
    index : Index<Sheet>,
    /// Value of this style sheet node. Style sheets without value behave like if they do not exist.
    value : Option<Data>,
    /// Expression used to update the value.
    expr : Option<BoundExpression>,
    /// Indexes of all `Var`s that are potential matches with this style sheet.
    matches : HashSet<Index<Var>>,
    /// Indexes of all `Var`s that are bound (best matches) with this style sheet.
    bindings : HashSet<Index<Var>>,
}

impl Sheet {
    /// Constructor.
    pub fn new(index:Index<Sheet>) -> Self {
        let value    = default();
        let expr     = default();
        let matches  = default();
        let bindings = default();
        Self {index,value,expr,matches,bindings}
    }

    /// Checks whether the style sheet exist. Style sheets without value are considered templates
    /// and are kept in the graph for optimization purposes only.
    pub fn exists(&self) -> bool {
        self.value.is_some()
    }
}


// =======================
// === BoundExpression ===
// =======================

/// Expression of a style sheet bound to specific variable indexes.
#[derive(Derivative)]
#[derivative(Clone,Debug)]
pub struct BoundExpression {
    /// Indexes of all vars which are used as sources to the function of this expression.
    args : Vec<Index<Var>>,
    /// Function used to compute the new value of the style sheet.
    #[derivative(Debug="ignore")]
    function : Rc<dyn Fn(&[&Data])->Data>
}

impl BoundExpression {
    pub fn new(args:Vec<Index<Var>>, function:Rc<dyn Fn(&[&Data])->Data>) -> Self {
        Self {args,function}
    }
}



// ==================
// === Expression ===
// ==================

#[derive(Clone)]
pub struct Expression {
    pub args     : Vec<Path>,
    pub function : Rc<dyn Fn(&[&Data])->Data>
}

impl Expression {
    pub fn new(args:Vec<Path>, function:Rc<dyn Fn(&[&Data])->Data>) -> Self {
        Self {args,function}
    }
}

impl Debug for Expression {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Expression")
    }
}

impl PartialEq for Expression {
    fn eq(&self, other:&Self) -> bool {
        (self.args == other.args) && Rc::ptr_eq(&self.function,&other.function)
    }
}



// =============
// === Value ===
// =============

#[derive(Clone,Debug,PartialEq)]
pub enum Value {
    Data       (Data),
    Expression (Expression)
}

impl From<Expression> for Value {
    fn from(t:Expression) -> Self {
        Self::Expression(t)
    }
}

impl<T> From<T> for Value
    where T:Into<Data> {
    default fn from(t:T) -> Self {
        Self::Data(t.into())
    }
}

impl Semigroup for Value {
    fn concat_mut(&mut self, other:&Self) {
        *self = other.clone()
    }

    fn concat_mut_take(&mut self, other:Self) {
        *self = other
    }
}



// =============
// === Value ===
// =============

pub struct Change {
    path  : Path,
    value : Option<Value>
}

impl Change {
    pub fn new<P>(path:P, value:Option<Value>) -> Self
    where P:Into<Path> {
        let path = path.into();
        Self {path,value}
    }
}



// =============
// === Types ===
// =============

// === Types ===

#[allow(missing_docs)]
mod types {
    use super::*;
    pub type VarVec   = OptVec<Var,Index<Var>>;
    pub type SheetVec = OptVec<Sheet,Index<Sheet>>;
    pub type VarMap   = HashMapTree<String,Option<Index<Var>>>;
    pub type SheetMap = HashMapTree<String,Index<Sheet>>;
}
use types::*;

trait NewInstance<K> {
    fn new_instance(&mut self) -> K;
}

impl NewInstance<Index<Var>> for VarVec {
    fn new_instance(&mut self) -> Index<Var> {
        self.insert_with_ix(Var::new)
    }
}

impl NewInstance<Index<Sheet>> for SheetVec {
    fn new_instance(&mut self) -> Index<Sheet> {
        self.insert_with_ix(Sheet::new)
    }
}



// ====================
// === RegistryData ===
// ====================

/// Style sheet registry. Could be named "Cascading Style Sheets" but then the name will be
/// confusing with CSS used in web development. Defines a set of cascading style sheets. Each
/// style sheet can be assigned with a value of type `Data` or an expression to compute one. It
/// also allows creating variables which are automatically bound to the most specific style sheet.
/// See `Var` and `Sheet` to learn more.
#[derive(Debug)]
pub struct RegistryData {
    /// Set of all variables.
    vars : VarVec,
    /// Set of all style sheets.
    sheets : SheetVec,
    /// Association of a path like 'button' -> 'size' to a variable.
    var_map : VarMap,
    /// Association of a path like 'button' -> 'size' to a style sheet.
    sheet_map : SheetMap,
}


// === Constructors ===

impl RegistryData {
    /// Constructor.
    pub fn new() -> Self {
        let vars          = default();
        let mut sheets    = OptVec::<Sheet,Index<Sheet>>::new();
        let var_map       = default();
        let root_sheet_id = sheets.new_instance();
        let sheet_map     = SheetMap::from_value(root_sheet_id);
        Self {vars,sheets,var_map,sheet_map}
    }

    /// Access variable by the given path or create new one if missing.
    ///
    /// Implementation note: under the hood, a `Sheet` for each sub-path will be created. For
    /// example, when creating "panel.button.size" variable, three sheets will be created as well:
    /// "panel.button.size", "button.size", and "size". This way we keep track of all possible
    /// matches and we can create high-performance value binding algorithms.
    pub fn unmanaged_var<P:Into<Path>>(&mut self, path:P) -> Index<Var> {
        let path         = path.into();
        let vars         = &mut self.vars;
        let sheets       = &mut self.sheets;
        let var_map_node = self.var_map.get_node(&path.rev_segments);
        let var_id       = *var_map_node.value_or_set_with(||vars.new_instance());

        let mut var_matches = Vec::new();
        self.sheet_map.get_node_traversing_with(&path.rev_segments,||{sheets.new_instance()}, |t| {
            var_matches.push(t.value)
        });
        var_matches.reverse();

        for sheet_id in &var_matches {
            self.sheets[*sheet_id].matches.insert(var_id);
        }

        self.vars[var_id].matches = var_matches;
        self.rebind_var(var_id);
        var_id
    }

    /// Access style sheet by the given path or create new one if missing.
    fn sheet<P:Into<Path>>(&mut self, path:P) -> Index<Sheet> {
        let path   = path.into();
        let sheets = &mut self.sheets;
        let node   = self.sheet_map.get_node_with(&path.rev_segments,|| sheets.new_instance());
        node.value
    }
}


// === Getters ===

impl RegistryData {
    /// Reads the value of the variable.
    pub fn value(&self, var_id:Index<Var>) -> Option<&Data> {
        self.vars.safe_index(var_id).as_ref().and_then(|var| {
            var.binding.and_then(|sheet_id| {
                self.sheets[sheet_id].value.as_ref()
            })
        })
    }
}


// === Setters ===

impl RegistryData {
    /// Set a style sheet value. Please note that it will remove expression assigned to the target
    /// style sheet if any. Returns indexes of all affected variables.
    pub fn set_value<P:Into<Path>>(&mut self, path:P, data:Data) -> HashSet::<Index<Var>> {
        self.set_value_to(path,Some(data))
    }

    /// Set a style sheet expression which will be used to automatically compute values whenever any
    /// of the provided dependencies will change. Returns indexes of all affected variables.
    pub fn set_expression<P>
    (&mut self, path:P, args:&[&str], function:Rc<dyn Fn(&[&Data])->Data>)
    -> HashSet::<Index<Var>>
    where P:Into<Path> {
        let args     = args.iter().map(|t|(*t).into()).collect_vec();
        let value    = Value::Expression(Expression {args,function});
        let changes  = vec![Change::new(path,Some(value))];
        self.change_values(changes)
    }

    /// Removes a style sheet value. Please note that it will remove expression assigned to the
    /// target style sheet if any. Returns indexes of all affected variables.
    pub fn remove_value<P:Into<Path>>(&mut self, path:P) -> HashSet::<Index<Var>> {
        self.set_value_to(path,None)
    }

    /// Set or unset a style sheet value. Please note that it will remove expression assigned to the
    /// target style sheet if any. Returns indexes of all affected variables.
    pub fn set_value_to<P>(&mut self, path:P, data:Option<Data>) -> HashSet::<Index<Var>>
    where P:Into<Path> {
        self.change_value(Change::new(path,data.map(Value::Data)))
    }

    /// Set or remove a single style sheet value. Returns indexes of all affected variables.
    pub fn change_value(&mut self, change:Change) -> HashSet::<Index<Var>> {
        self.change_values(iter::once(change))
    }

    /// Set or remove a several style sheet values. Returns indexes of all affected variables.
    pub fn change_values<I>(&mut self, changes:I) -> HashSet::<Index<Var>>
    where I:IntoIterator<Item=Change> {
        let mut changed = HashSet::<Index<Var>>::new();
        let sheets_iter = changes.into_iter().map(|change| {
            let sheet_id = self.sheet(change.path);
            let sheet    = &mut self.sheets[sheet_id];

            // Remove expression bindings.
            let opt_expr = mem::take(&mut sheet.expr);
            if let Some(expr) = opt_expr {
                for var_id in expr.args {
                    self.vars[var_id].usages.remove(&sheet_id);
                }
            }

            // Set new value and rebind variables.
            match change.value {
                None => {
                    let needs_rebind = sheet.value.is_some();
                    sheet.value      = None;
                    if needs_rebind {
                        for var_id in sheet.bindings.clone() {
                            if self.rebind_var(var_id) {
                                changed.insert(var_id);
                            }
                        }
                    }
                },
                Some(value) => {
                    let needs_rebind = sheet.value.is_none();
                    match value {
                        Value::Data(data) => sheet.value = Some(data),
                        Value::Expression(expr) => {
                            let vars = expr.args.iter().map(|path| self.unmanaged_var(path)).collect_vec();
                            for var_id in &vars {
                                self.vars[*var_id].usages.insert(sheet_id);
                            }
                            let bound_expr = BoundExpression::new(vars,expr.function);
                            let sheet      = &mut self.sheets[sheet_id];
                            sheet.expr     = Some(bound_expr);
                            self.recompute(sheet_id);
                        }
                    }
                    if needs_rebind {
                        let sheet = &self.sheets[sheet_id];
                        for var_id in sheet.matches.clone() {
                            if self.rebind_var(var_id) {
                                changed.insert(var_id);
                            }
                        }
                    }
                }
            };
            sheet_id
        });

        // Recompute values in the whole graph.
        let sheets = sheets_iter.collect_vec();
        for sheet_id in self.sheet_topo_sort(sheets) {
            let sheet = &self.sheets[sheet_id];
            changed.extend(&sheet.bindings);
            self.recompute(sheet_id);
        }

        changed
    }
}


// === Utils ===

impl RegistryData {
    /// Check all potential candidates (sheets) this variable matches to and choose the most
    /// specific one from those which exist (have a value). Returns true if the var was rebound.
    fn rebind_var(&mut self, var_id:Index<Var>) -> bool {
        let mut rebound = false;
        let mut found   = false;
        let var         = &self.vars[var_id];
        for sheet_id in var.matches.clone() {
            let sheet = &self.sheets[sheet_id];
            if sheet.exists() {
                if let Some(sheet_id) = var.binding {
                    self.sheets[sheet_id].bindings.remove(&var_id);
                }
                let var         = &mut self.vars[var_id];
                let sheet       = &mut self.sheets[sheet_id];
                let new_binding = Some(sheet_id);
                rebound         = var.binding != new_binding;
                var.binding     = new_binding;
                sheet.bindings.insert(var_id);
                found = true;
                break
            }
        }
        if found { rebound } else { self.unbind_var(var_id) }
    }

    /// Removes all binding information from var and related style sheets. Returns true if var
    /// needed rebound.
    fn unbind_var(&mut self, var_id:Index<Var>) -> bool {
        let var = &mut self.vars[var_id];
        match var.binding {
            None => false,
            Some(sheet_id) => {
                self.sheets[sheet_id].bindings.remove(&var_id);
                var.binding = None;
                true
            }
        }
    }

    /// Recomputes the value of the provided sheet if the sheet was assigned with an expression.
    fn recompute(&mut self, sheet_id:Index<Sheet>) {
        let sheet = &self.sheets[sheet_id];
        let value = sheet.expr.as_ref().and_then(|expr| {
            let mut opt_args : Vec<Option<&Data>> = Vec::new();
            for var_id in &expr.args {
                opt_args.push(self.value(*var_id));
            }
            let args : Option<Vec<&Data>> = opt_args.into_iter().collect();
            args.map(|v| (expr.function)(&v) )
        });
        let sheet_mut = &mut self.sheets[sheet_id];
        value.for_each(|v| sheet_mut.value = Some(v));
    }

    /// Traverses all sheets whose value depend on the value of the provided sheet and sorts them
    /// in a topological order. This is used mainly for efficient implementation of sheet
    /// recomputation mechanism.
    fn sheet_topo_sort<T>(&self, changed_sheets:T) -> Vec<Index<Sheet>>
    where T:Into<Vec<Index<Sheet>>> {
        let changed_sheets      = changed_sheets.into();
        let mut sheet_ref_count = HashMap::<Index<Sheet>,usize>::new();
        let mut sorted_sheets   = changed_sheets.clone();
        self.with_all_sheet_deps(&changed_sheets[..], |sheet_id| {
            *sheet_ref_count.entry(sheet_id).or_default() += 1;
        });
        self.with_all_sheet_deps(changed_sheets, |sheet_id| {
            let ref_count = sheet_ref_count.entry(sheet_id).or_default();
            *ref_count -= 1;
            if *ref_count == 0 {
                sorted_sheets.push(sheet_id);
            }
        });
        sorted_sheets
    }

    /// Runs the provided callback with all sheet indexes whose value depend on the values of the
    /// provided sheets.
    fn with_all_sheet_deps<T,F>(&self, targets:T, mut callback:F)
    where T:Into<Vec<Index<Sheet>>>, F:FnMut(Index<Sheet>) {
        let mut sheets_to_visit = targets.into();
        while !sheets_to_visit.is_empty() {
            if let Some(current_sheet_id) = sheets_to_visit.pop() {
                let sheet = &self.sheets[current_sheet_id];
                for var_id in &sheet.bindings {
                    let var = &self.vars[*var_id];
                    for sheet_id in &var.usages {
                        callback(*sheet_id);
                        sheets_to_visit.push(*sheet_id);
                    }
                }
            }
        }
    }
}


// === Debug ===

impl RegistryData {
    /// Visualizes the network in the GraphViz Dot language. Use `visualize` to automatically
    /// display it in a new browser tab.
    pub fn to_graphviz(&self) -> String {
        let mut dot = String::new();
        Self::sheet_map_to_graphviz(&mut dot,&self.sheet_map);
        Self::var_map_to_graphviz(&mut dot,&mut vec![],&self.var_map);
        let s = &mut dot;
        for var in &self.vars {
            for sheet in &var.matches {Self::var_sheet_link(s,var.index,*sheet,"[style=dashed]")}
            for sheet in &var.binding {Self::var_sheet_link(s,var.index,*sheet,"[color=red]")}
            for sheet in &var.usages  {Self::var_sheet_link(s,var.index,*sheet,"[color=blue]")}
        }
        for sheet in &self.sheets {
            for var  in &sheet.matches  {Self::sheet_var_link(s,sheet.index,*var,"[style=dashed]")}
            for var  in &sheet.bindings {Self::sheet_var_link(s,sheet.index,*var,"[color=red]")}
            for expr in &sheet.expr {
                for var in &expr.args {Self::sheet_var_link(s,sheet.index,*var,"[color=blue]")}
            }
        }
        format!("digraph G {{\nnode [shape=box style=rounded]\n{}\n}}",dot)
    }

    fn sheet_map_to_graphviz(dot:&mut String, sheet_map:&SheetMap) {
        let sheet_id = sheet_map.value;
        dot.push_str(&iformat!("sheet_{sheet_id}\n"));
        for (path,child) in &sheet_map.branches {
            Self::sheet_sheet_link(dot,sheet_id,child.value,iformat!("[label=\"{path}\"]"));
            Self::sheet_map_to_graphviz(dot,child);
        }
    }

    fn var_map_to_graphviz(dot:&mut String, path:&mut Vec<String>, var_map:&VarMap) {
        var_map.value.for_each(|var_id| {
            let real_path = path.iter().rev().join(".");
            dot.push_str(&iformat!("var_{var_id} [label=\"Var({real_path})\"]\n"));
        });
        for (segment,child) in &var_map.branches {
            path.push(segment.into());
            Self::var_map_to_graphviz(dot,path,child);
            path.pop();
        }
    }

    fn var_sheet_link<S>(dot:&mut String, var_id:Index<Var>, sheet_id:Index<Sheet>, s:S)
    where S:Into<String> {
        Self::link(dot,"var","sheet",var_id,sheet_id,s)
    }

    fn sheet_var_link<S>(dot:&mut String, sheet_id:Index<Sheet>, var_id:Index<Var>, s:S)
    where S:Into<String> {
        Self::link(dot,"sheet","var",sheet_id,var_id,s)
    }

    fn sheet_sheet_link<S>(dot:&mut String, sheet_id_1:Index<Sheet>, sheet_id_2:Index<Sheet>, s:S)
    where S:Into<String> {
        Self::link(dot,"sheet","sheet",sheet_id_1,sheet_id_2,s)
    }

    fn link<Src,Tgt,S>(dot:&mut String, src_pfx:&str, tgt_pfx:&str, src:Src, tgt:Tgt, s:S)
    where Src:Display, Tgt:Display, S:Into<String> {
        dot.push_str(&format!("{}_{} -> {}_{} {}\n",src_pfx,src,tgt_pfx,tgt,s.into()));
    }
}

// === Impls ===

impl Default for RegistryData {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug,Default)]
pub struct Registry {
    rc : Rc<RefCell<RegistryData>>
}

impl Registry {
    pub fn new() -> Self {
        default()
    }
}



// =============
// === Tests ===
// =============

/// Interactive testing utility. To be removed in the future.
pub fn test() {
    let mut style = RegistryData::new();

    let var_size              = style.unmanaged_var("size");
    let var_button_size       = style.unmanaged_var("button.size");
    let var_graph_button_size = style.unmanaged_var("graph.button.size");
    let _var = style.unmanaged_var("scene.background.color");
    let _var = style.unmanaged_var("application.text.color");
    let _var = style.unmanaged_var("application.text.size");
    let _var = style.unmanaged_var("button.text.size");
    let _var = style.unmanaged_var("node.text.color");
    let _var = style.unmanaged_var("node.text.size");
    let _var = style.unmanaged_var("application.background.color");
    let _var = style.unmanaged_var("node.background.color");

    assert!(style.value(var_graph_button_size).is_none());
    style.set_value("size",data(1.0));
    style.set_expression("graph.button.size",&["button.size"],Rc::new(|args| args[0] + &data(100.0)));
    style.set_expression("button.size",&["size"],Rc::new(|args| args[0] + &data(10.0)));
    style.set_value("button.size",data(3.0));

    println!("{}",style.to_graphviz());
    println!("{:?}", style.value(var_graph_button_size));
    println!("{:?}", style.value(var_button_size));
    println!("{:?}", style.vars[var_graph_button_size]);
    println!("{:?}", style.sheets[style.vars[var_graph_button_size].binding.unwrap()]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn simple_var_binding_1() {
        let mut style = RegistryData::new();
        let var1      = style.unmanaged_var(&["size"]);
        assert!(style.value(var1).is_none());
        style.set_value(&["size"],data(1.0));
        assert_eq!(style.value(var1),Some(&data(1.0)));
    }

    #[test]
    pub fn simple_var_binding_2() {
        let mut style = RegistryData::new();
        style.set_value(&["size"],data(1.0));
        let var1 = style.unmanaged_var(&["size"]);
        assert_eq!(style.value(var1),Some(&data(1.0)));
    }

    #[test]
    pub fn hierarchical_var_binding() {
        let mut style = RegistryData::new();
        let var1      = style.unmanaged_var("graph.button.size");
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

    #[test]
    pub fn expr_bindings_1() {
        let mut style = RegistryData::new();

        let var_size              = style.unmanaged_var("size");
        let var_button_size       = style.unmanaged_var("button.size");
        let var_graph_button_size = style.unmanaged_var("graph.button.size");

        assert!(style.value(var_graph_button_size).is_none());
        style.set_value("size",data(1.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(1.0)));
        style.set_expression("graph.button.size",&["button.size"],Rc::new(|args| args[0] + &data(10.0)));
        assert_eq!(style.value(var_graph_button_size),Some(&data(11.0)));
        style.set_expression("button.size",&["size"],Rc::new(|args| args[0] + &data(100.0)));
        assert_eq!(style.value(var_graph_button_size),Some(&data(111.0)));
        style.set_value("size",data(2.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(112.0)));
        style.set_value("button.size",data(3.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(13.0)));
        style.set_value("button.size",data(4.0));
        assert_eq!(style.value(var_graph_button_size),Some(&data(14.0)));
    }

    #[test]
    pub fn expr_circular() {
        let mut style = RegistryData::new();

        let var_a = style.unmanaged_var("a");
        let var_b = style.unmanaged_var("b");

        style.set_expression("a",&["b"],Rc::new(|args| args[0].clone()));
        style.set_expression("b",&["a"],Rc::new(|args| args[0].clone()));
        assert!(style.value(var_a).is_none());
    }
}



//todo todo todo
//
//1. vary sa tworzone tylko na potrzeby expressionow
//2. jak expression jest usuwany, vary powinny spawdzac czy sa uzywane pzez inne expresiony i sie usuwac
//3. po usunieciu vara powinny usuwac sie sheety
//4. vary powinny miec flage ze sa uzywane przez uzytkownika i jezeli tak, nie usuwac sie dopoki uzytkownik nie przestanie z nich korzystac
