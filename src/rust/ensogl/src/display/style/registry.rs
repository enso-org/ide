//! This module defines a cascading style sheet registry and related style management utilities.

use crate::prelude::*;

use crate::control::callback;
use crate::data::HashMapTree;
use crate::data::Index;
use crate::data::OptVec;

pub use super::data::Data;
pub use super::data::data;
pub use super::path::Path;



// =============
// === Query ===
// =============

/// Pointer to a sheet. Always bound to the most specific style sheet matching the query. For
/// example, the query 'panel.button.size' will be bound to 'panel.button.size', 'button.size', or
/// 'size' in that order.
///
/// # Implementation Details
/// Each query keeps a list of all style sheets which match this query (`matches`). For example,
/// the query 'panel.button.size' contains three matches - 'panel.button.size', 'button.size', and
/// 'size'. The longest match with a defined value is considered the best match and is remembered
/// (`binding`). Moreover, each query keeps list of all sheets which use this query in their
/// expressions (`usages`). A query is considered unused and can be safely removed from the graph
/// if no sheets use it in their expressions and it is not referred by an external, user code
/// (`external_count`).
#[derive(Debug)]
pub struct Query {
    path           : Path,
    index          : Index<Query>,
    matches        : Vec<Index<Sheet>>,
    binding        : Option<Index<Sheet>>,
    usages         : HashSet<Index<Sheet>>,
    external_count : usize,
}

impl Query {
    /// Constructor.
    pub fn new(path:Path,index:Index<Query>) -> Self {
        let matches        = default();
        let binding        = default();
        let usages         = default();
        let external_count = default();
        Self {path,index,matches,binding,usages,external_count}
    }

    /// Checks whether the variable is being used. Please note that all external variables are
    /// considered to be used.
    pub fn is_unused(&self) -> bool {
        self.external_count == 0 && self.usages.is_empty()
    }

    pub fn is_external(&self) -> bool {
        self.external_count > 0
    }

    fn inc_external_count(&mut self) {
        self.external_count += 1;
    }

    fn dec_external_count(&mut self) {
        self.external_count -= 1;
    }
}



// =============
// === Sheet ===
// =============

/// A style sheet tree node. Each sheet is associated with a style path like 'panel.button.size' and
/// contains a `Data` value. The value can either be set explicitly, or computed automatically if
/// the sheet is assigned with `Expression`. Please note that although `Sheet` technically contains
/// a single value, it is a node in a style sheet tree defined in `CascadingSheetsData`, and it can be
/// interpreted as a set of hierarchical values instead.
///
/// # Implementation Details
/// Each sheet keeps list of all queries which match this sheet (`matches`). It also keeps list of
/// all queries which were bound to this sheet (`bindings`). To learn more about matches and
/// bindings see the `Query` docs.
#[derive(Debug)]
pub struct Sheet {
    path     : Path,
    index    : Index<Sheet>,
    value    : Option<Data>,
    expr     : Option<BoundExpression>,
    matches  : HashSet<Index<Query>>,
    bindings : HashSet<Index<Query>>,
}

impl Sheet {
    /// Constructor.
    pub fn new(path:Path, index:Index<Sheet>) -> Self {
        let value    = default();
        let expr     = default();
        let matches  = default();
        let bindings = default();
        Self {path,index,value,expr,matches,bindings}
    }

    /// Checks whether the style sheet exist. Style sheets without value are considered templates
    /// and are kept in the graph for optimization purposes only.
    pub fn exists(&self) -> bool {
        self.value.is_some()
    }

    /// Checks whether the sheet is being used.
    pub fn is_unused(&self) -> bool {
        self.matches.is_empty() && self.value.is_none()
    }
}



// ==================
// === Expression ===
// ==================

/// Style sheet expression declaration.
#[derive(Clone)]
pub struct Expression {
    pub args     : Vec<Path>,
    pub function : Rc<dyn Fn(&[&Data])->Data>
}

impl Expression {
    /// Constructor.
    pub fn new<A,I,F>(args:A, function:F) -> Self
        where A:IntoIterator<Item=I>, I:Into<Path>, F:'static+Fn(&[&Data])->Data {
        let args     = args.into_iter().map(|t|t.into()).collect_vec();
        let function = Rc::new(function);
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



// =======================
// === BoundExpression ===
// =======================

/// Style sheet expression bound to specific queries, being arguments of this expression.
#[derive(Clone)]
pub struct BoundExpression {
    args     : Vec<Index<Query>>,
    function : Rc<dyn Fn(&[&Data])->Data>
}

impl BoundExpression {
    /// Constructor.
    pub fn new(args:Vec<Index<Query>>, function:Rc<dyn Fn(&[&Data])->Data>) -> Self {
        Self {args,function}
    }
}

impl Debug for BoundExpression {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"BoundExpression")
    }
}



// =============
// === Value ===
// =============

/// A style sheet value declaration.
#[derive(Clone,Debug,PartialEq)]
#[allow(missing_docs)]
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



// ==============
// === Change ===
// ==============

/// Defines a change to a style sheet. Style sheets allow bulk-application of changes in order to
/// optimize the amount of necessary computations under the hood.
pub struct Change {
    path  : Path,
    value : Option<Value>
}

impl Change {
    /// Constructor.
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
    pub type QueryVec = OptVec<Query,Index<Query>>;
    pub type SheetVec = OptVec<Sheet,Index<Sheet>>;
    pub type QueryMap = HashMapTree<String,Option<Index<Query>>>;
    pub type SheetMap = HashMapTree<String,Index<Sheet>>;
}
use types::*;









// ===========
// === Var ===
// ===========

#[derive(Clone,CloneRef,Debug)]
pub struct Var {
    rc : Rc<VarData>
}

#[derive(Debug)]
pub struct VarData {
    sheets    : CascadingSheets,
    query_id  : Index<Query>,
    callbacks : callback::SharedRegistryMut,
}

pub trait VarChangeCallback = 'static + FnMut();

impl VarData {
    pub fn new<R>(sheets:R, query_id:Index<Query>, callbacks:callback::SharedRegistryMut) -> Self
    where R:Into<CascadingSheets> {
        let sheets = sheets.into();
        sheets.rc.borrow_mut().queries[query_id].inc_external_count();
        Self {sheets,query_id,callbacks}
    }

    pub fn on_change<F:VarChangeCallback>(&self, callback:F) -> callback::Handle {
        self.callbacks.add(callback)
    }
}

impl Drop for VarData {
    fn drop(&mut self) {
        self.sheets.callbacks.borrow_mut().remove(&self.query_id);
        let sheets_data = &mut *self.sheets.rc.borrow_mut();
        sheets_data.queries[self.query_id].dec_external_count();
        sheets_data.drop_query_if_unused(self.query_id);
    }
}

impl Var {
    pub fn new<R>(sheets:R, query_id:Index<Query>, callbacks:callback::SharedRegistryMut) -> Self
        where R:Into<CascadingSheets> {
        let rc = Rc::new(VarData::new(sheets,query_id,callbacks));
        Self {rc}
    }
}





// =======================
// === CascadingSheets ===
// =======================

#[derive(Clone,CloneRef,Debug,Default)]
pub struct CascadingSheets {
    rc        : Rc<RefCell<CascadingSheetsData>>,
    callbacks : Rc<RefCell<HashMap<Index<Query>,callback::SharedRegistryMut>>>
}

impl CascadingSheets {
    pub fn new() -> Self {
        default()
    }

    pub fn var<P>(&self, path:P) -> Var
    where P:Into<Path> {
        let query_id          = self.rc.borrow_mut().unmanaged_query(path);
        let callback_registry = callback::SharedRegistryMut::default();
        self.callbacks.borrow_mut().insert(query_id,callback_registry.clone_ref());
        Var::new(self,query_id,callback_registry)
    }

//    pub fn run_callbacks_for(&self, query_id:Index<Query>) {
//        if let Some(callbacks) = self.callbacks.borrow().get(&query_id).map(|t| t.clone_ref()) {
//            callbacks.run_all()
//        }
//    }
}



// ===========================
// === CascadingSheetsData ===
// ===========================

/// Style sheet registry. Could be named "Cascading Style Sheets" but then the name will be
/// confusing with CSS used in web development. Defines a set of cascading style sheets. Each
/// style sheet can be assigned with a value of type `Data` or an expression to compute one. It
/// also allows creating variables which are automatically bound to the most specific style sheet.
/// See `Query` and `Sheet` to learn more.
#[derive(Debug)]
pub struct CascadingSheetsData {
    /// Set of all variables.
    queries : QueryVec,
    /// Set of all style sheets.
    sheets : SheetVec,
    /// Association of a path like 'button' -> 'size' to a variable.
    query_map : QueryMap,
    /// Association of a path like 'button' -> 'size' to a style sheet.
    sheet_map : SheetMap,
}


// === Constructors ===

impl CascadingSheetsData {
    /// Constructor.
    pub fn new() -> Self {
        let queries       = default();
        let mut sheets    = OptVec::<Sheet,Index<Sheet>>::new();
        let query_map       = default();
        let root_sheet_id = sheets.insert_with_ix(|ix| Sheet::new(Path::empty(),ix));
        let sheet_map     = SheetMap::from_value(root_sheet_id);
        Self {queries,sheets,query_map,sheet_map}
    }

    /// Access variable by the given path or create new one if missing.
    ///
    /// # Implementation Notes
    /// Under the hood, a `Sheet` for each sub-path will be created. For
    /// example, when creating "panel.button.size" variable, three sheets will be created as well:
    /// "panel.button.size", "button.size", and "size". This way we keep track of all possible
    /// matches and we can create high-performance value binding algorithms.
    pub fn unmanaged_query<P:Into<Path>>(&mut self, path:P) -> Index<Query> {
        let path         = path.into();
        let queries         = &mut self.queries;
        let sheets       = &mut self.sheets;
        let query_map_node = self.query_map.get_node(&path.rev_segments);

        let mut query_matches = Vec::new();
        self.sheet_map.get_node_path_traversing_with(&path.rev_segments,|p| {
            sheets.insert_with_ix(|ix| Sheet::new(Path::from_rev_segments(p),ix))
        }, |t| {
            query_matches.push(t.value)
        });
        query_matches.reverse();

        let query_id       = *query_map_node.value_or_set_with(|| {
            queries.insert_with_ix(move |ix| Query::new(path,ix))
        });

        for sheet_id in &query_matches {
            self.sheets[*sheet_id].matches.insert(query_id);
        }

        self.queries[query_id].matches = query_matches;
        self.rebind_query(query_id);
        query_id
    }

    /// Access style sheet by the given path or create new one if missing.
    fn sheet<P:Into<Path>>(&mut self, path:P) -> Index<Sheet> {
        let path   = path.into();
        let sheets = &mut self.sheets;
        let node   = self.sheet_map.get_node_path_traversing_with(&path.rev_segments,|p| {
            sheets.insert_with_ix(|ix| Sheet::new(Path::from_rev_segments(p),ix))
        }, |_| {});
        node.value
    }
}


// === Getters ===

impl CascadingSheetsData {
    /// Reads the value of the variable.
    pub fn query_value(&self, query_id:Index<Query>) -> Option<&Data> {
        self.queries.safe_index(query_id).as_ref().and_then(|query| {
            query.binding.and_then(|sheet_id| {
                self.sheets[sheet_id].value.as_ref()
            })
        })
    }

    /// Queries the style sheet for a value of a path like it was a variable. For example,
    /// querying "button.size" will return the value of "size" if no exact match was found.
    pub fn query<P>(&self, path:P) -> Option<&Data>
    where P:Into<Path> {
        let mut path = path.into();
        while path.rev_segments.is_empty() {
            let value = self.value(&path);
            if value.is_some() { return value }
        }
        return None
    }

    /// Reads the value of the style sheet by the exact path provided. If you want to read a value
    /// of a variable binding, use `query` instead.
    pub fn value<P>(&self, path:P) -> Option<&Data>
    where P:Into<Path> {
        let path = path.into();
        let segs = &path.rev_segments;
        self.sheet_map.get_node2(segs).and_then(|t| self.sheets[t.value].value.as_ref())
    }

    /// Returns the amount of queries used.
    pub fn query_count(&self) -> usize {
        self.queries.len()
    }

    /// Returns the amount of sheets used not including the root sheet.
    pub fn sheet_count(&self) -> usize {
        let root_sheet_count = 1;
        self.sheets.len() - root_sheet_count
    }
}


// === Setters ===

impl CascadingSheetsData {
    /// Sets the value by the given path. Returns indexes of all affected variables.
    pub fn set<P,V>(&mut self, path:P, value:V) -> HashSet::<Index<Query>>
    where P:Into<Path>, V:Into<Value> {
        let value = value.into();
        self.apply_change(Change::new(path,Some(value)))
    }

    /// Removes the value by the given path. Returns indexes of all affected variables.
    pub fn unset<P>(&mut self, path:P) -> HashSet::<Index<Query>>
    where P:Into<Path> {
        self.apply_change(Change::new(path,None))
    }

    /// Changes the value by the given path. Providing `None` as the value means that the value
    /// will be removed. Returns indexes of all affected variables.
    pub fn change<P>(&mut self, path:P, value:Option<Value>) -> HashSet::<Index<Query>>
    where P:Into<Path> {
        self.apply_change(Change::new(path,value))
    }

    /// Apply a `Change`. Returns indexes of all affected variables.
    pub fn apply_change(&mut self, change:Change) -> HashSet::<Index<Query>> {
        self.apply_changes(iter::once(change))
    }

    /// Apply a set of `Change`s. Returns indexes of all affected variables.
    pub fn apply_changes<I>(&mut self, changes:I) -> HashSet::<Index<Query>>
    where I:IntoIterator<Item=Change> {
        let mut changed          = HashSet::<Index<Query>>::new();
        let mut possible_orphans = Vec::<Index<Sheet>>::new();
        let sheets_iter = changes.into_iter().map(|change| {
            let sheet_id = self.sheet(change.path);
            let sheet    = &mut self.sheets[sheet_id];

            // Remove expression bindings.
            let opt_expr = mem::take(&mut sheet.expr);
            if let Some(expr) = opt_expr {
                for query_id in expr.args {
                    self.queries[query_id].usages.remove(&sheet_id);
                    self.drop_query_if_unused(query_id);
                }
            }

            // Set new value and rebind variables.
            let sheet = &mut self.sheets[sheet_id];
            match change.value {
                None => {
                    let needs_rebind = sheet.value.is_some();
                    if needs_rebind {
                        sheet.value = None;
                        for query_id in sheet.bindings.clone() {
                            if self.rebind_query(query_id) {
                                changed.insert(query_id);
                            }
                        }
                        possible_orphans.push(sheet_id);
                    }
                },
                Some(value) => {
                    let needs_rebind = sheet.value.is_none();
                    match value {
                        Value::Data(data) => sheet.value = Some(data),
                        Value::Expression(expr) => {
                            let queries = expr.args.iter().map(|path| self.unmanaged_query(path)).collect_vec();
                            for query_id in &queries {
                                self.queries[*query_id].usages.insert(sheet_id);
                            }
                            let bound_expr = BoundExpression::new(queries,expr.function);
                            let sheet      = &mut self.sheets[sheet_id];
                            sheet.expr     = Some(bound_expr);
                            self.recompute(sheet_id);
                        }
                    }
                    if needs_rebind {
                        let sheet = &self.sheets[sheet_id];
                        for query_id in sheet.matches.clone() {
                            if self.rebind_query(query_id) {
                                changed.insert(query_id);
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

        for sheet_id in possible_orphans {
            self.drop_sheet_if_unused(sheet_id);
        }

        changed
    }
}


// === Utils ===

impl CascadingSheetsData {
    /// Check all potential candidates (sheets) this variable matches to and choose the most
    /// specific one from those which exist (have a value). Returns true if the var was rebound.
    fn rebind_query(&mut self, query_id:Index<Query>) -> bool {
        let mut rebound = false;
        let mut found   = false;
        let query       = &self.queries[query_id];
        for sheet_id in query.matches.clone() {
            let sheet = &self.sheets[sheet_id];
            if sheet.exists() {
                if let Some(sheet_id) = query.binding {
                    self.sheets[sheet_id].bindings.remove(&query_id);
                }
                let query       = &mut self.queries[query_id];
                let sheet       = &mut self.sheets[sheet_id];
                let new_binding = Some(sheet_id);
                rebound         = query.binding != new_binding;
                query.binding   = new_binding;
                sheet.bindings.insert(query_id);
                found = true;
                break
            }
        }
        if found { rebound } else { self.unbind_query(query_id) }
    }

    fn drop_query_if_unused(&mut self, query_id:Index<Query>) {
        let query_ref = &self.queries[query_id];
        if query_ref.is_unused() {
            if let Some(query) = self.queries.remove(query_id) {
                let node = self.query_map.get_node(&query.path.rev_segments);
                node.value = None;
                for sheet_id in query.matches {
                    let sheet = &mut self.sheets[sheet_id];
                    sheet.matches.remove(&query_id);
                    sheet.bindings.remove(&query_id);
                    self.drop_sheet_if_unused(sheet_id);
                }
            }
        }
    }

    fn drop_sheet_if_unused(&mut self, sheet_id:Index<Sheet>) {
        let mut segments = self.sheets[sheet_id].path.rev_segments.clone();
        loop {
            if segments.is_empty() { break }
            if let Some(node) = self.sheet_map.get_node2(&segments) {
                let no_children = node.branches.is_empty();
                let sheet_id    = node.value;
                let unused      = self.sheets[sheet_id].is_unused();
                if no_children && unused {
                    self.sheets.remove(sheet_id);
                    self.sheet_map.remove(&segments);
                    segments.pop();
                } else {
                    break;
                }
            }
        }
    }


    /// Removes all binding information from var and related style sheets. Returns true if var
    /// needed rebound.
    fn unbind_query(&mut self, query_id:Index<Query>) -> bool {
        let query = &mut self.queries[query_id];
        match query.binding {
            None => false,
            Some(sheet_id) => {
                self.sheets[sheet_id].bindings.remove(&query_id);
                query.binding = None;
                true
            }
        }
    }

    /// Recomputes the value of the provided sheet if the sheet was assigned with an expression.
    fn recompute(&mut self, sheet_id:Index<Sheet>) {
        let sheet = &self.sheets[sheet_id];
        let value = sheet.expr.as_ref().and_then(|expr| {
            let mut opt_args : Vec<Option<&Data>> = Vec::new();
            for query_id in &expr.args {
                opt_args.push(self.query_value(*query_id));
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
                for query_id in &sheet.bindings {
                    let query = &self.queries[*query_id];
                    for sheet_id in &query.usages {
                        callback(*sheet_id);
                        sheets_to_visit.push(*sheet_id);
                    }
                }
            }
        }
    }
}


// === Debug ===

impl CascadingSheetsData {
    /// Visualizes the network in the GraphViz Dot language. Use `visualize` to automatically
    /// display it in a new browser tab.
    pub fn to_graphviz(&self) -> String {
        let mut dot = String::new();
        self.sheet_map_to_graphviz(&mut dot,&self.sheet_map);
        self.query_map_to_graphviz(&mut dot,&mut vec![],&self.query_map);
        let s = &mut dot;
        for query in &self.queries {
            for sheet in &query.matches {Self::query_sheet_link(s,query.index,*sheet,"[style=dashed]")}
            for sheet in &query.binding {Self::query_sheet_link(s,query.index,*sheet,"[color=red]")}
            for sheet in &query.usages  {Self::query_sheet_link(s,query.index,*sheet,"[color=blue]")}
        }
        for sheet in &self.sheets {
            for query  in &sheet.matches  {Self::sheet_query_link(s,sheet.index,*query,"[style=dashed]")}
            for query  in &sheet.bindings {Self::sheet_query_link(s,sheet.index,*query,"[color=red]")}
            for expr in &sheet.expr {
                for query in &expr.args {Self::sheet_query_link(s,sheet.index,*query,"[color=blue]")}
            }
        }
        format!("digraph G {{\nnode [shape=box style=rounded]\n{}\n}}",dot)
    }

    fn sheet_map_to_graphviz(&self, dot:&mut String, sheet_map:&SheetMap) {
        let sheet_id = sheet_map.value;
        let sheet    = &self.sheets[sheet_id];
        let value    = format!("{:?}",sheet.value);
        dot.push_str(&iformat!("sheet_{sheet_id} [label=\"sheet_{sheet_id}({value})\"]\n"));
        for (path,child) in &sheet_map.branches {
            Self::sheet_sheet_link(dot,sheet_id,child.value,iformat!("[label=\"{path}\"]"));
            self.sheet_map_to_graphviz(dot,child);
        }
    }

    fn query_map_to_graphviz(&self, dot:&mut String, path:&mut Vec<String>, query_map:&QueryMap) {
        query_map.value.for_each(|query_id| {
            let query       = &self.queries[query_id];
            let scope     = if query.is_external() { "External" } else { "Internal" };
            let real_path = path.iter().rev().join(".");
            dot.push_str(&iformat!("query_{query_id} [label=\"{scope} Query({real_path})\"]\n"));
        });
        for (segment,child) in &query_map.branches {
            path.push(segment.into());
            self.query_map_to_graphviz(dot,path,child);
            path.pop();
        }
    }

    fn query_sheet_link<S>(dot:&mut String, query_id:Index<Query>, sheet_id:Index<Sheet>, s:S)
    where S:Into<String> {
        Self::link(dot,"query","sheet",query_id,sheet_id,s)
    }

    fn sheet_query_link<S>(dot:&mut String, sheet_id:Index<Sheet>, query_id:Index<Query>, s:S)
    where S:Into<String> {
        Self::link(dot,"sheet","query",sheet_id,query_id,s)
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

impl Default for CascadingSheetsData {
    fn default() -> Self {
        Self::new()
    }
}





// =============
// === Tests ===
// =============

/// Interactive testing utility. To be removed in the future.
pub fn test() {
    let mut style = CascadingSheetsData::new();

//    let var_size              = style.unmanaged_query("size");
//    let var_button_size       = style.unmanaged_query("button.size");
//    let var_graph_button_size = style.unmanaged_query("graph.button.size");

//    assert!(style.value(var_graph_button_size).is_none());
//    style.set("size",data(1.0));
//    style.set("graph.button.size",Expression::new(&["button.size"], |args| args[0] + &data(100.0)));
//    style.set("button.size",Expression::new(&["size"], |args| args[0] + &data(10.0)));
//    style.set("button.size",data(3.0));

    style.set(&["size"],data(1.0));
    style.set(&["button.size"],data(2.0));
    style.set(&["circle.radius"],data(3.0));

    println!("-----------");
//    style.remove_value("graph.button.size");


    println!("{}",style.to_graphviz());
//    println!("{:?}", style.value(var_graph_button_size));
//    println!("{:?}", style.value(var_button_size));
//    println!("{:?}", style.queries[var_graph_button_size]);
//    println!("{:?}", style.sheets[style.queries[var_graph_button_size].binding.unwrap()]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_query_sheet_count(style:&CascadingSheetsData, query_count:usize, sheet_count:usize) {
        assert_eq!(style.query_count(),query_count);
        assert_eq!(style.sheet_count(),sheet_count);
    }

    #[test]
    pub fn memory_management_for_single_value() {
        let mut style = CascadingSheetsData::new();
        style.set("size",data(1.0));
        assert_query_sheet_count(&style,0,1);
        style.unset("size");
        assert_query_sheet_count(&style,0,0);
    }

    #[test]
    pub fn memory_management_for_multiple_values() {
        let mut style = CascadingSheetsData::new();
        style.set("size",data(1.0));
        style.set("button.size",data(2.0));
        style.set("circle.radius",data(3.0));
        assert_query_sheet_count(&style,0,4);
        style.unset("size");
        assert_query_sheet_count(&style,0,4);
        style.unset("button.size");
        assert_query_sheet_count(&style,0,2);
        style.unset("circle.radius");
        assert_query_sheet_count(&style,0,0);
    }

    #[test]
    pub fn memory_management_for_single_expression() {
        let mut style = CascadingSheetsData::new();
        style.set("button.size",data(1.0));
        assert_query_sheet_count(&style,0,2);
        style.set("circle.radius",Expression::new(&["button.size"], |args| args[0] + &data(10.0)));
        assert_query_sheet_count(&style,1,4);
        assert_eq!(style.value("circle.radius"),Some(&data(11.0)));
        style.unset("button.size");
        assert_query_sheet_count(&style,1,4);
        assert_eq!(style.value("circle.radius"),Some(&data(11.0))); // Impossible to update.
        style.set("button.size",data(2.0));
        assert_query_sheet_count(&style,1,4);
        assert_eq!(style.value("circle.radius"),Some(&data(12.0)));
        style.set("circle.radius",data(3.0));
        assert_query_sheet_count(&style,0,4);
        style.unset("button.size");
        assert_query_sheet_count(&style,0,2);
        style.unset("circle.radius");
        assert_query_sheet_count(&style,0,0);
    }

    #[test]
    pub fn simple_query_binding_1() {
        let mut style = CascadingSheetsData::new();
        let query1      = style.unmanaged_query("size");
        assert!(style.query_value(query1).is_none());
        style.set("size",data(1.0));
        assert_eq!(style.query_value(query1),Some(&data(1.0)));
    }

    #[test]
    pub fn simple_query_binding_2() {
        let mut style = CascadingSheetsData::new();
        style.set("size",data(1.0));
        let query1 = style.unmanaged_query("size");
        assert_eq!(style.query_value(query1),Some(&data(1.0)));
    }

    #[test]
    pub fn hierarchical_query_binding() {
        let mut style = CascadingSheetsData::new();
        let query1      = style.unmanaged_query("graph.button.size");
        assert!(style.query_value(query1).is_none());
        style.set("size",data(1.0));
        assert_eq!(style.query_value(query1),Some(&data(1.0)));
        style.set("button.size",data(2.0));
        assert_eq!(style.query_value(query1),Some(&data(2.0)));
        style.set("graph.button.size",data(3.0));
        assert_eq!(style.query_value(query1),Some(&data(3.0)));
        style.unset("graph.button.size");
        assert_eq!(style.query_value(query1),Some(&data(2.0)));
        style.unset("button.size");
        assert_eq!(style.query_value(query1),Some(&data(1.0)));
        style.unset("size");
        assert_eq!(style.query_value(query1),None);
    }

    #[test]
    pub fn expr_bindings_1() {
        let mut style = CascadingSheetsData::new();

        let query_size              = style.unmanaged_query("size");
        let query_button_size       = style.unmanaged_query("button.size");
        let query_graph_button_size = style.unmanaged_query("graph.button.size");

        assert!(style.query_value(query_graph_button_size).is_none());
        style.set("size",data(1.0));
        assert_eq!(style.query_value(query_graph_button_size),Some(&data(1.0)));
        style.set("graph.button.size",Expression::new(&["button.size"], |args|args[0]+&data(10.0)));
        assert_eq!(style.query_value(query_graph_button_size),Some(&data(11.0)));
        style.set("button.size",Expression::new(&["size"],|args| args[0] + &data(100.0)));
        assert_eq!(style.query_value(query_graph_button_size),Some(&data(111.0)));
        style.set("size",data(2.0));
        assert_eq!(style.query_value(query_graph_button_size),Some(&data(112.0)));
        style.set("button.size",data(3.0));
        assert_eq!(style.query_value(query_graph_button_size),Some(&data(13.0)));
        style.set("button.size",data(4.0));
        assert_eq!(style.query_value(query_graph_button_size),Some(&data(14.0)));
    }

    #[test]
    pub fn expr_circular() {
        let mut style = CascadingSheetsData::new();
        style.set("a",Expression::new(&["b"], |args| args[0].clone()));
        style.set("b",Expression::new(&["a"], |args| args[0].clone()));
        assert!(style.value("a").is_none());
        assert!(style.value("b").is_none());
    }
}
