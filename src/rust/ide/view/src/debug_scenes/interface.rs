#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use crate::prelude::*;

use crate::graph_editor;
use crate::graph_editor::GraphEditor;
use crate::graph_editor::Type;
use crate::project;

use enso_frp as frp;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::system::web;
use ensogl::application::Application;
use ensogl::display::object::ObjectOps;
use ensogl_text as text;
use ensogl_theme as theme;
use wasm_bindgen::prelude::*;
use parser::Parser;



const STUB_MODULE:&str = "from Base import all\n\nmain = IO.println \"Hello\"\n";


#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_interface() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    run_once_initialized(|| {
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());
        init(&app);
        mem::forget(app);
    });
}


fn _fence<T,Out>(network:&frp::Network, trigger:T) -> (frp::Stream,frp::Stream<bool>)
where T:frp::HasOutput<Output=Out>, T:Into<frp::Stream<Out>>, Out:frp::Data {
    let trigger = trigger.into();
    frp::extend! { network
        def trigger_ = trigger.constant(());
        def runner   = source::<()>();
        def switch   = any_mut();
        switch.attach(&trigger_);
        def triggered = trigger.map(f_!(runner.emit(())));
        switch.attach(&triggered);
        def condition = switch.toggle_true();
    }
    let runner = runner.into();
    (runner,condition)
}



// ==================
// === Mock Types ===
// ==================

/// Allows the creation of arbitrary unique `Type`s.
#[derive(Clone,Debug,Default)]
struct DummyTypeGenerator {
    type_counter : u32
}

impl DummyTypeGenerator {
    fn get_dummy_type(&mut self) -> Type {
        self.type_counter += 1;
        Type::from(format!("dummy_type_{}",self.type_counter))
    }
}



// ========================
// === Init Application ===
// ========================

fn init(app:&Application) {

    let _bg = app.display.scene().style_sheet.var(theme::application::background);

    let world     = &app.display;
    let scene     = world.scene();
    let camera    = scene.camera();
    let navigator = Navigator::new(&scene,&camera);

    app.views.register::<project::View>();
    app.views.register::<text::Area>();
    app.views.register::<GraphEditor>();
    let project_view = app.new_view::<project::View>();
    let graph_editor = project_view.graph();
    let code_editor  = project_view.code_editor();
    world.add_child(&project_view);

    code_editor.text_area().set_content(STUB_MODULE.to_owned());


    // === Nodes ===

    let node1_id = graph_editor.add_node();
    let node2_id = graph_editor.add_node();
    let node3_id = graph_editor.add_node();

    graph_editor.frp.set_node_position.emit((node1_id,Vector2(-150.0,50.0)));
    graph_editor.frp.set_node_position.emit((node2_id,Vector2(50.0,50.0)));
    graph_editor.frp.set_node_position.emit((node3_id,Vector2(150.0,250.0)));


    let expression_1 = expression_mock();
    graph_editor.frp.set_node_expression.emit((node1_id,expression_1.clone()));
    let expression_2 = expression_mock3();
    graph_editor.frp.set_node_expression.emit((node2_id,expression_2.clone()));

    let expression_3 = expression_mock2();
    graph_editor.frp.set_node_expression.emit((node3_id,expression_3));
    let error = "Runtime Error".to_string().into();
    graph_editor.frp.set_node_error_status.emit((node3_id,Some(error)));


    // === Connections ===

    let src = graph_editor::EdgeEndpoint::new(node1_id,span_tree::Crumbs::new(default()));
    let tgt = graph_editor::EdgeEndpoint::new(node2_id,span_tree::Crumbs::new(vec![0,0,0,0,1]));
    graph_editor.frp.connect_nodes.emit((src,tgt));


    // === Types (Port Coloring) ===

    let mut dummy_type_generator = DummyTypeGenerator::default();
    expression_1.input_span_tree.root_ref().leaf_iter().for_each(|node|{
        if let Some(expr_id) = node.ast_id {
            let dummy_type = Some(dummy_type_generator.get_dummy_type());
            graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,dummy_type));
        }
    });
    expression_1.output_span_tree.root_ref().leaf_iter().for_each(|node|{
        if let Some(expr_id) = node.ast_id {
            let dummy_type = Some(dummy_type_generator.get_dummy_type());
            graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,dummy_type));
        }
    });

    expression_2.input_span_tree.root_ref().leaf_iter().for_each(|node|{
        if let Some(expr_id) = node.ast_id {
            let dummy_type = Some(dummy_type_generator.get_dummy_type());
            graph_editor.frp.set_expression_usage_type.emit((node2_id,expr_id,dummy_type));
        }
    });
    expression_2.output_span_tree.root_ref().leaf_iter().for_each(|node|{
        if let Some(expr_id) = node.ast_id {
            let dummy_type = Some(dummy_type_generator.get_dummy_type());
            graph_editor.frp.set_expression_usage_type.emit((node2_id,expr_id,dummy_type));
        }
    });


    // let tgt_type = dummy_type_generator.get_dummy_type();
    let mut was_rendered = false;
    let mut loader_hidden = false;
    let mut i = 100;
    // let mut j = 3;
    world.on_frame(move |_| {
        let _keep_alive = &navigator;
        let _keep_alive = &project_view;
        let graph_editor = project_view.graph();

        if i > 0 { i -= 1 } else {
            println!("CHANGING TYPES OF EXPRESSIONS");
            i = 10000;
            graph_editor.frp.set_node_expression.emit((node2_id,expression_2.clone()));
            // expression_1.input_span_tree.root_ref().leaf_iter().for_each(|node|{
            //     if let Some(expr_id) = node.ast_id {
            //         let dummy_type = Some(tgt_type.clone());
            //         // if j != 0 {
            //         //     j -= 1;
            //         println!("----\n");
            //             graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,dummy_type));
            //         // } else {
            //         //     println!(">> null change");
            //             // j = 3;
            //             // graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,None));
            //             // graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,dummy_type));
            //         // };
            //     }
            // });
        }

        // Temporary code removing the web-loader instance.
        // To be changed in the future.
        if was_rendered && !loader_hidden {
            web::get_element_by_id("loader").map(|t| {
                t.parent_node().map(|p| {
                    p.remove_child(&t).unwrap()
                })
            }).ok();
            loader_hidden = true;
        }
        was_rendered = true;
    }).forget();

    depth_test();
}



// =============
// === Mocks ===
// =============

use crate::graph_editor::component::node::Expression;

use ast::crumbs::*;
use ast::crumbs::PatternMatchCrumb::*;
use enso_protocol::prelude::Uuid;
use ensogl_text_msdf_sys::run_once_initialized;
use span_tree::traits::*;


pub fn expression_mock() -> Expression {
    let pattern    = Some("var1".to_string());
    let code       = "[1,2,3]".to_string();
    let parser     = Parser::new_or_panic();
    let this_param = span_tree::ArgumentInfo {
        name : Some("this".to_owned()),
        tp   : Some("Text".to_owned()),
    };
    let parameters       = vec![this_param];
    let ast              = parser.parse_line(&code).unwrap();
    let invocation_info  = span_tree::generate::context::CalledMethodInfo {parameters};
    let ctx              = span_tree::generate::MockContext::new_single(ast.id.unwrap(),invocation_info);
    let output_span_tree = span_tree::SpanTree::default();
    let input_span_tree  = span_tree::SpanTree::new(&ast,&ctx).unwrap();
    Expression {pattern,code,input_span_tree,output_span_tree}
}

pub fn expression_mock2() -> Expression {
    let pattern          = Some("var1".to_string());
    let pattern_cr       = vec![Seq { right: false }, Or, Or, Build];
    let val              = ast::crumbs::SegmentMatchCrumb::Body {val:pattern_cr};
    let parens_cr        = ast::crumbs::MatchCrumb::Segs {val,index:0};
    let code             = "make_maps size (distribution normal)".into();
    let output_span_tree = span_tree::SpanTree::default();
    let input_span_tree  = span_tree::builder::TreeBuilder::new(36)
        .add_child(0,14,span_tree::node::Kind::Chained,PrefixCrumb::Func)
            .add_child(0,9,span_tree::node::Kind::Operation,PrefixCrumb::Func)
                .set_ast_id(Uuid::new_v4())
                .done()
            .add_empty_child(10,span_tree::node::InsertionPointType::BeforeTarget)
            .add_child(10,4,span_tree::node::Kind::this().removable(),PrefixCrumb::Arg)
                .set_ast_id(Uuid::new_v4())
                .done()
            .add_empty_child(14,span_tree::node::InsertionPointType::Append)
            .set_ast_id(Uuid::new_v4())
            .done()
        .add_child(15,21,span_tree::node::Kind::argument().removable(),PrefixCrumb::Arg)
            .set_ast_id(Uuid::new_v4())
            .add_child(1,19,span_tree::node::Kind::argument(),parens_cr)
                .set_ast_id(Uuid::new_v4())
                .add_child(0,12,span_tree::node::Kind::Operation,PrefixCrumb::Func)
                    .set_ast_id(Uuid::new_v4())
                    .done()
                .add_empty_child(13,span_tree::node::InsertionPointType::BeforeTarget)
                .add_child(13,6,span_tree::node::Kind::this(),PrefixCrumb::Arg)
                    .set_ast_id(Uuid::new_v4())
                    .done()
                .add_empty_child(19,span_tree::node::InsertionPointType::Append)
                .done()
            .done()
        .add_empty_child(36,span_tree::node::InsertionPointType::Append)
        .build();
    Expression {pattern,code,input_span_tree,output_span_tree}
}

pub fn expression_mock3() -> Expression {
    let pattern    = Some("Vector x y z".to_string());
    // let code       = "image.blur ((foo   bar) baz)".to_string();
    let code       = "Vector x y z".to_string();
    let parser     = Parser::new_or_panic();
    let this_param = span_tree::ArgumentInfo {
        name : Some("this".to_owned()),
        tp   : Some("Image".to_owned()),
    };
    let param0 = span_tree::ArgumentInfo {
        name : Some("radius".to_owned()),
        tp   : Some("Number".to_owned()),
    };
    let param1 = span_tree::ArgumentInfo {
        name : Some("name".to_owned()),
        tp   : Some("Text".to_owned()),
    };
    let param2 = span_tree::ArgumentInfo {
        name : Some("area".to_owned()),
        tp   : Some("Vector Int".to_owned()),
    };
    let param3 = span_tree::ArgumentInfo {
        name : Some("matrix".to_owned()),
        tp   : Some("Vector String".to_owned()),
    };
    let parameters       = vec![this_param,param0,param1,param2,param3];
    let ast              = parser.parse_line(&code).unwrap();
    let invocation_info  = span_tree::generate::context::CalledMethodInfo {parameters};
    let ctx              = span_tree::generate::MockContext::new_single(ast.id.unwrap(),invocation_info);
    let output_span_tree = span_tree::SpanTree::new(&ast,&ctx).unwrap();//span_tree::SpanTree::default();
    let input_span_tree  = span_tree::SpanTree::new(&ast,&ctx).unwrap();
    Expression {pattern,code,input_span_tree,output_span_tree}
}



// TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO
// TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO
// TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO

// Extract and make use in scene depth sorting.




#[derive(Clone,Copy,Debug,PartialEq,PartialOrd,Eq,Ord)]
pub enum NoEqOrdering { Less, Greater }

impl NoEqOrdering {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Less    => Self::Greater,
            Self::Greater => Self::Less,
        }
    }
}

#[derive(Clone,Copy,Debug,PartialEq,PartialOrd,Eq,Ord)]
pub enum RelationSource {Provided,Inferred}

#[derive(Clone,Copy,Debug,PartialEq,PartialOrd,Eq,Ord)]
pub struct Relation {
    source   : RelationSource,
    ordering : NoEqOrdering,
    count    : usize,
}

impl Relation {
    pub fn new(source:RelationSource, ordering:NoEqOrdering, count:usize) -> Self {
        Self {source,ordering,count}
    }

    pub fn provided(ordering:NoEqOrdering) -> Self {
        let source = RelationSource::Provided;
        Self::new(source,ordering,0)
    }

    pub fn inferred(ordering:NoEqOrdering) -> Self {
        let source = RelationSource::Inferred;
        Self::new(source,ordering,0)
    }

    pub fn inc(&mut self) {
        self.count += 1
    }

    pub fn dec(&mut self) {
        self.count -= 1
    }

    pub fn as_provided(&mut self) {
        self.source = RelationSource::Provided;
    }

    pub fn as_inferred(&mut self) {
        self.source = RelationSource::Inferred;
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound="T:Debug+Eq+Hash"))]
#[derivative(Default(bound="T:Eq+Hash"))]
#[derivative(PartialEq(bound="T:Eq+Hash"))]
pub struct OrderingMap<T> {
    map : HashMap<T,HashMap<T,Relation>>
}

impl<T:Eq+Hash+Copy> OrderingMap<T> {
    pub fn insert(&mut self, a:T, b:T, ord:NoEqOrdering) {
        match ord {
            NoEqOrdering::Less    => self.insert_less(a,b),
            NoEqOrdering::Greater => self.insert_less(b,a),
        }
    }

    pub fn insert_less(&mut self, below:T, over:T) {
        self.remove(below,over);
        let top_elems    = self.over(over).copied().collect_vec();
        let bottom_elems = self.below(below).copied().collect_vec();
        self.insert_less_asymmetrical(below,over,&bottom_elems,&top_elems,NoEqOrdering::Greater);
        self.insert_less_asymmetrical(over,below,&top_elems,&bottom_elems,NoEqOrdering::Less);
    }

    pub fn remove(&mut self, below:T, over:T) {
        let top_elems    = self.over(over).copied().collect_vec();
        let bottom_elems = self.below(below).copied().collect_vec();
        self.remove_asymmetrical(below,over,&bottom_elems,&top_elems);
        self.remove_asymmetrical(over,below,&top_elems,&bottom_elems);
    }

    pub fn insert_less_asymmetrical
    (&mut self, a:T, b:T, a_side:&[T], b_side:&[T], rel:NoEqOrdering) {
        let a_rels = self.map.entry(a).or_default();
        a_rels.entry(b).and_modify(|t|t.as_provided()).or_insert(Relation::provided(rel)).inc();
        for b_side_elem in b_side {
            a_rels.entry(*b_side_elem).or_insert(Relation::inferred(rel)).inc();
        }
        for b_side_elem in b_side {
            let rev_rel           = rel.reverse();
            let b_side_elem_entry = self.map.entry(*b_side_elem).or_default();
            b_side_elem_entry.entry(a).or_insert(Relation::inferred(rev_rel)).inc();
            for a_side_elem in a_side {
                b_side_elem_entry.entry(*a_side_elem).or_insert(Relation::inferred(rev_rel)).inc();
            }
        }
    }

    pub fn remove_asymmetrical
    (&mut self, a:T, b:T, a_side:&[T], b_side:&[T]) {
        let a_map        = self.map.get(&a);
        let b_rel        = a_map.and_then(|m|m.get(&b));
        let was_provided = b_rel.map(|t|t.source == RelationSource::Provided).unwrap_or_default();
        if was_provided {
            if let Some(a_rels) = self.map.get_mut(&a) {
                let to_remove = a_rels.get_mut(&b).map(|b_entry| {
                    b_entry.as_inferred();
                    b_entry.dec();
                    b_entry.count == 0
                }).unwrap_or_default();
                if to_remove {
                    a_rels.remove(&b);
                }
                for b_side_elem in b_side {
                    let to_remove = a_rels.get_mut(b_side_elem).map(|b_entry| {
                        b_entry.dec();
                        b_entry.count == 0
                    }).unwrap_or_default();
                    if to_remove {
                        a_rels.remove(b_side_elem);
                    }
                }
                for b_side_elem in b_side {
                    if let Some(b_side_elem_entry) = self.map.get_mut(b_side_elem) {
                        let to_remove = b_side_elem_entry.get_mut(&a).map(|b_entry| {
                            b_entry.dec();
                            b_entry.count == 0
                        }).unwrap_or_default();
                        if to_remove {
                            b_side_elem_entry.remove(&a);
                        }

                        for a_side_elem in a_side {
                            let to_remove = b_side_elem_entry.get_mut(a_side_elem).map(|b_entry| {
                                b_entry.dec();
                                b_entry.count == 0
                            }).unwrap_or_default();
                            if to_remove {
                                b_side_elem_entry.remove(a_side_elem);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn iter_rel(&self, ord:NoEqOrdering, target:T) -> impl Iterator<Item=&T> {
        self.map.get(&target).into_iter().flatten().filter_map(
            move |(elem,rel)| (rel.ordering == ord).as_some(elem)
        )
    }

    pub fn below(&self, target:T) -> impl Iterator<Item=&T> {
        self.iter_rel(NoEqOrdering::Less,target)
    }

    pub fn over(&self, target:T) -> impl Iterator<Item=&T> {
        self.iter_rel(NoEqOrdering::Greater,target)
    }

    pub fn unchecked_from_vec(slice:&[(T,T,Relation)]) -> Self {
        let mut this = OrderingMap::default();
        for (a,b,rel) in slice {
            this.map.entry(*a).or_default().entry(*b).insert(*rel);
        }
        this
    }

    pub fn to_vec(&self) -> Vec<(T,T,Relation)> {
        self.map.iter().map(|(a,m)| m.iter().map(move |(b,r)| (*a,*b,*r))).flatten().collect()
    }
}


pub fn depth_test() {
    println!("DEPTH TEST");
    let mut m = OrderingMap::<usize>::default();
    m.insert_less(1,2);
    m.insert_less(3,4);
    m.insert_less(4,5);
    println!("\n-------");
    m.insert_less(2,3);
    m.insert_less(2,4);
    println!("{:#?}",m);
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use RelationSource::*;
    use NoEqOrdering::*;

    fn pg(count:usize) -> Relation { Relation::new(Provided,Greater,count) }
    fn pl(count:usize) -> Relation { Relation::new(Provided,Less,count) }
    fn ig(count:usize) -> Relation { Relation::new(Inferred,Greater,count) }
    fn il(count:usize) -> Relation { Relation::new(Inferred,Less,count) }

    fn check(m:&OrderingMap<usize>, slice:&[(usize,usize,Relation)]) {
        let mut v1 = m.to_vec();
        let mut v2 = OrderingMap::<usize>::unchecked_from_vec(slice).to_vec();
        v1.sort();
        v2.sort();
        assert_eq!(v1,v2);
    }

    macro_rules! check {
        ($name:ident, $($rules:tt)*) => {
            check(&$name,rules!{[] $($rules)*,});
        };
    }

    macro_rules! rules {
        ([$($x:tt)*] $(,)?)                              => {&[$($x)*]};
        ([$($x:tt)*] $a:tt << $b:tt [$n:tt], $($ts:tt)*) => {rules!{[$($x)* ($a,$b,pg($n)),] $($ts)*}};
        ([$($x:tt)*] $a:tt >> $b:tt [$n:tt], $($ts:tt)*) => {rules!{[$($x)* ($a,$b,pl($n)),] $($ts)*}};
        ([$($x:tt)*] $a:tt <  $b:tt [$n:tt], $($ts:tt)*) => {rules!{[$($x)* ($a,$b,ig($n)),] $($ts)*}};
        ([$($x:tt)*] $a:tt >  $b:tt [$n:tt], $($ts:tt)*) => {rules!{[$($x)* ($a,$b,il($n)),] $($ts)*}};
    }


    #[test]
    fn hierarchy_test() {
        let mut m = OrderingMap::<usize>::default();

        // === Insertion ===

        check!(m,);
        m.insert(1,2,Less);
        check!(m, 1<<2[1], 2>>1[1]);
        m.insert(3,4,Less);
        check!(m, 1<<2[1], 2>>1[1], 3<<4[1], 4>>3[1]);
        m.insert(2,3,Less);
        check!(m, 1<<2[1], 2>>1[1], 3> 1[1], 4> 1[1],
                  1< 3[1], 2<<3[1], 3>>2[1], 4> 2[1],
                  1< 4[1], 2< 4[1], 3<<4[1], 4>>3[1]);
        m.insert(4,5,Less);
        check!(m, 1<<2[1], 2>>1[1], 3> 1[1], 4> 1[1], 5> 1[1],
                  1< 3[1], 2<<3[1], 3>>2[1], 4> 2[1], 5> 2[1],
                  1< 4[1], 2< 4[1], 3<<4[1], 4>>3[1], 5> 3[1],
                  1< 5[1], 2< 5[1], 3< 5[1], 4<<5[1], 5>>4[1]);
        m.insert(2,4,Less);
        check!(m, 1<<2[1], 2>>1[1], 3> 1[1], 4> 1[2], 5> 1[2],
                  1< 3[1], 2<<3[1], 3>>2[1], 4>>2[2], 5> 2[2],
                  1< 4[2], 2<<4[2], 3<<4[1], 4>>3[1], 5> 3[1],
                  1< 5[2], 2< 5[2], 3< 5[1], 4<<5[1], 5>>4[1]);


        // === Removal ===

        m.remove(2,4);
        check!(m, 1<<2[1], 2>>1[1], 3> 1[1], 4> 1[1], 5> 1[1],
                  1< 3[1], 2<<3[1], 3>>2[1], 4> 2[1], 5> 2[1],
                  1< 4[1], 2< 4[1], 3<<4[1], 4>>3[1], 5> 3[1],
                  1< 5[1], 2< 5[1], 3< 5[1], 4<<5[1], 5>>4[1]);
        m.remove(4,5);
        check!(m, 1<<2[1], 2>>1[1], 3> 1[1], 4> 1[1],
                  1< 3[1], 2<<3[1], 3>>2[1], 4> 2[1],
                  1< 4[1], 2< 4[1], 3<<4[1], 4>>3[1]);
        m.remove(2,3);
        check!(m, 1<<2[1], 2>>1[1], 3<<4[1], 4>>3[1]);
        m.remove(3,4);
        check!(m, 1<<2[1], 2>>1[1]);
        m.remove(1,2);
        check!(m,);
    }
}

pub trait DepthId = Copy + Debug + Eq + Hash + Ord;

pub struct DepthItem<T> {
    elem : T,
    ordering : DepthOrdering<T>,
}

impl<T:Debug> Debug for DepthItem<T> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DepthItem({:?})", self.elem)
    }
}

impl<T:PartialEq> PartialEq for DepthItem<T> {
    fn eq(&self, other:&Self) -> bool {
        self.elem == other.elem
    }
}

impl<T:PartialEq> Eq for DepthItem<T> {}

impl<T:DepthId> Ord for DepthItem<T> {
    fn cmp(&self, other:&Self) -> Ordering {
        self.ordering.check(self.elem,other.elem).unwrap_or_else(||self.elem.cmp(&other.elem))
    }
}

impl<T:DepthId> PartialOrd for DepthItem<T> {
    fn partial_cmp(&self, other:&Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(CloneRef)]
#[derive(Derivative)]
#[derivative(Debug(bound="T:DepthId"))]
#[derivative(Clone(bound=""))]
#[derivative(Default(bound="T:DepthId"))]
pub struct DepthOrdering<T> {
    below : Rc<RefCell<HashMap<(T,T),Ordering>>>,
}

impl <T:Copy+Eq+Hash> DepthOrdering<T> {
    fn check(&self, first:T, second:T) -> Option<Ordering> {
        self.below.borrow().get(&(first,second)).copied()
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound="T:DepthId"))]
#[derivative(Default(bound="T:DepthId"))]
pub struct DepthHierarchy<T> {
    ordering  : DepthOrdering<T>,
    hierarchy : BTreeSet<DepthItem<T>>
}

impl<T:DepthId> DepthHierarchy<T> {
    fn add_rule_below(&self, below:T, over:T) {
        let ordering = self.ordering.below.borrow_mut();
        // ordering.insert((below,over))
    }
}




#[allow(clippy::implicit_hasher)]
pub fn depth_sort(ids:&[usize], elem_above_elems:&HashMap<usize,Vec<usize>>) -> Vec<usize> {

    // === Remove from `elem_above_elems` all ids which are not present in `ids` ===

    let mut elem_above_elems : HashMap<usize,Vec<usize>> = elem_above_elems.clone();
    let mut missing = vec![];
    for (elem,above_elems) in &mut elem_above_elems {
        above_elems.retain(|id| ids.contains(id));
        if above_elems.is_empty() {
            missing.push(*elem);
        }
    }
    for id in &missing {
        elem_above_elems.remove(id);
    }


    // === Generate `elem_below_elems` map ===

    let mut elem_below_elems : HashMap<usize,Vec<usize>> = HashMap::new();
    for (above_id,below_ids) in &elem_above_elems {
        for below_id in below_ids {
            elem_below_elems.entry(*below_id).or_default().push(*above_id);
        }
    }


    // === Sort ids ===

    let mut queue        = HashSet::<usize>::new();
    let mut sorted       = vec![];
    let mut newly_sorted = vec![];

    for id in ids {
        if elem_above_elems.get(id).is_some() {
            queue.insert(*id);
        } else {
            newly_sorted.push(*id);
            while !newly_sorted.is_empty() {
                let id = newly_sorted.pop().unwrap();
                sorted.push(id);
                elem_below_elems.remove(&id).for_each(|above_ids| {
                    for above_id in above_ids {
                        if let Some(lst) = elem_above_elems.get_mut(&above_id) {
                            lst.remove_item(&id);
                            if lst.is_empty() && queue.contains(&above_id) {
                                queue.remove(&above_id);
                                newly_sorted.push(above_id);
                            }
                            if lst.is_empty() {
                                elem_above_elems.remove(&above_id);
                            }
                        }
                    }
                })
            }
        }
    }
    sorted
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_with_no_rules() {
        assert_eq!( depth_sort(&vec![]      , &default()) , Vec::<usize>::new() );
        assert_eq!( depth_sort(&vec![1]     , &default()) , vec![1] );
        assert_eq!( depth_sort(&vec![1,3]   , &default()) , vec![1,3] );
        assert_eq!( depth_sort(&vec![1,2,3] , &default()) , vec![1,2,3] );
    }


    #[test]
    fn chained_rules() {
        let mut rules = HashMap::<usize,Vec<usize>>::new();
        rules.insert(1,vec![2]);
        rules.insert(2,vec![3]);
        assert_eq!( depth_sort(&vec![]      , &rules) , Vec::<usize>::new() );
        assert_eq!( depth_sort(&vec![1]     , &rules) , vec![1] );
        assert_eq!( depth_sort(&vec![1,2]   , &rules) , vec![2,1] );
        assert_eq!( depth_sort(&vec![1,2,3] , &rules) , vec![3,2,1] );
    }

    #[test]
    fn order_preserving() {
        let mut rules = HashMap::<usize,Vec<usize>>::new();
        rules.insert(1,vec![2]);
        rules.insert(2,vec![3]);
        assert_eq!( depth_sort(&vec![10,11,12]          , &rules) , vec![10,11,12] );
        assert_eq!( depth_sort(&vec![10,1,11,12]        , &rules) , vec![10,1,11,12] );
        assert_eq!( depth_sort(&vec![10,1,11,2,12]      , &rules) , vec![10,11,2,1,12] );
        assert_eq!( depth_sort(&vec![10,1,11,2,12,3,13] , &rules) , vec![10,11,12,3,2,1,13] );
    }
}
