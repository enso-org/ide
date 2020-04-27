//! Graph Controller.
//!
//! This controller provides access to a specific graph. It lives under a module controller, as
//! each graph belongs to some module.


use crate::prelude::*;

pub use crate::double_representation::graph::Id;
use crate::double_representation::graph::GraphInfo;
pub use crate::double_representation::graph::LocationHint;
use crate::double_representation::definition;
use crate::double_representation::node;
use crate::model::module::NodeMetadata;
use crate::notification;

use ast::crumbs::InfixCrumb;
use parser::Parser;
use span_tree::{SpanTree, SplitCrumbs};
use span_tree::action::{Implementation, Actions, Action};
use crate::double_representation::node::NodeInfo;
use crate::double_representation::definition::DefinitionName;
use crate::double_representation::alias_analysis::{NormalizedName, LocatedName};



// ==============
// === Errors ===
// ==============

/// Error raised when node with given Id was not found in the graph's body.
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Node with Id {} was not found.", _0)]
pub struct NodeNotFound(ast::Id);

/// Error raised when an attempt to set node's expression to a binding has been made.
#[derive(Clone,Debug,Fail)]
#[fail(display="Illegal string `{}` given for node expression. It must not be a binding.", _0)]
pub struct BindingExpressionNotAllowed(String);

/// Expression AST cannot be used to produce a node. Means a bug in parser and id-giving code.
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Internal error: failed to create a new node.")]
pub struct FailedToCreateNode;



// ============
// === Node ===
// ============

/// Description of the node with all information available to the graph controller.
#[derive(Clone,Debug)]
pub struct Node {
    /// Information based on AST, from double_representation module.
    pub info : double_representation::node::NodeInfo,
    /// Information about this node stored in the module's metadata.
    pub metadata : Option<NodeMetadata>,
}



// ===================
// === NewNodeInfo ===
// ===================

/// Describes the node to be added.
#[derive(Clone,Debug)]
pub struct NewNodeInfo {
    /// Expression to be placed on the node
    pub expression : String,
    /// Visual node position in the graph scene.
    pub metadata : Option<NodeMetadata>,
    /// ID to be given to the node.
    pub id : Option<ast::Id>,
    /// Where line created by adding this node should appear.
    pub location_hint : LocationHint
}

impl NewNodeInfo {
    /// New node with given expression added at the end of the graph's blocks.
    pub fn new_pushed_back(expression:impl Str) -> NewNodeInfo {
        NewNodeInfo {
            expression    : expression.into(),
            metadata      : default(),
            id            : default(),
            location_hint : LocationHint::End,
        }
    }
}



// ===================
// === Connections ===
// ===================


// === Endpoint ===

/// Connection endpoint - a port on a node, described using span-tree crumbs.
#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub struct Endpoint {
    pub node   : double_representation::node::Id,
    pub crumbs : span_tree::SplitCrumbs,
}


// === Connection ===

/// Connection described using span-tree crumbs.
#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub struct Connection {
    pub source      : Endpoint,
    pub destination : Endpoint
}


// === Connections ===

pub struct Trees {
    trees: HashMap<node::Id,NodeTrees>
}

impl Trees {

}

/// Describes connections in the graph. For convenience also includes information about port
/// structure of the involved nodes.
#[derive(Clone,Debug,Default)]
pub struct Connections {
    /// Span trees for all nodes that have connections.
    pub trees       : HashMap<node::Id,NodeTrees>,
    /// The connections between nodes in the graph.
    pub connections : Vec<Connection>,
}

impl Connections {
    /// Describes a connection for given double representation graph.
    pub fn new(graph:&GraphInfo) -> Connections {
        let trees = graph.nodes().iter().flat_map(|node| {
            Some((node.id(), NodeTrees::new(node)?))
        }).collect();

        let mut ret = Connections {trees, connections:default()};
        let connections = graph.connections().into_iter().flat_map(|c|
            ret.convert_connection(&c)
        ).collect();
        ret.connections = connections;
        ret
    }

    /// Converts Endpoint from double representation to the span tree crumbs.
    pub fn convert_endpoint
    (&self, endpoint:&double_representation::connection::Endpoint) -> Option<Endpoint> {
        let tree = self.trees.get(&endpoint.node)?;
        Some(Endpoint{
            node  : endpoint.node,
            crumbs: tree.convert_crumbs(&endpoint.crumbs)?,
        })
    }

    /// Converts Connection from double representation to the span tree crumbs.
    pub fn convert_connection
    (&self, connection:&double_representation::connection::Connection) -> Option<Connection> {
        Some(Connection {
            source      : self.convert_endpoint(&connection.source)?,
            destination : self.convert_endpoint(&connection.destination)?,
        })
    }
}


// === NodeTrees ===

/// Stores node's span trees: one for inputs (expression) and optionally another one for inputs
/// (pattern).
#[derive(Clone,Debug)]
pub struct NodeTrees {
    /// Describes node inputs, i.e. its expression.
    pub inputs : SpanTree,
    /// Describes node outputs, i.e. its pattern. `None` if a node is not an assignment.
    pub outputs : Option<SpanTree>,
}

impl NodeTrees {
    #[allow(missing_docs)]
    pub fn new(node:&node::NodeInfo) -> Option<NodeTrees> {
        let inputs  = SpanTree::new(node.expression()).ok()?;
        let outputs = if let Some(pat) = node.pattern() {
            Some(SpanTree::new(pat).ok()?)
        } else {
            None
        };
        Some(NodeTrees {inputs,outputs})
    }

    /// Converts AST crumbs (as obtained from double rep's connection endpoint) into the span-tree
    /// crumbs.
    pub fn convert_crumbs(&self, ast_crumbs:&ast::Crumbs) -> Option<SplitCrumbs> {
        if let Some(outputs) = self.outputs.as_ref() {
            // Node in assignment form. First crumb decides which span tree to use.
            let tree = match ast_crumbs.get(0) {
                Some(ast::crumbs::Crumb::Infix(InfixCrumb::LeftOperand)) => outputs,
                Some(ast::crumbs::Crumb::Infix(InfixCrumb::RightOperand)) => &self.inputs,
                _ => return None,
            };
            Some(tree.convert_from_ast_crumbs(&ast_crumbs[1..]))
        } else {
            // Expression node - there is only inputs span tree.
            Some(self.inputs.convert_from_ast_crumbs(ast_crumbs))
        }
    }
}



// =================
// === Utilities ===
// =================

pub fn name_for_ast(ast:&Ast) -> String {
    use ast::*;
    match ast.shape() {
        Shape::Var          (ident) => ident.name.clone(),
        Shape::Cons         (ident) => ident.name.to_lowercase(),
        Shape::Number       (_)     => "number".into(),
        Shape::DanglingBase (_)     => "number".into(),
        Shape::TextLineRaw  (_)     => "text".into(),
        Shape::TextLineFmt  (_)     => "text".into(),
        Shape::TextBlockRaw (_)     => "text".into(),
        Shape::TextBlockFmt (_)     => "text".into(),
        Shape::TextUnclosed (_)     => "text".into(),
        Shape::Opr          (opr)   => {
            match opr.name.as_ref() {
                "+" => "sum",
                "*" => "product",
                "-" => "difference",
                "/" => "quotient",
                _   => "operator",
            }.into()
        }
        _ => {
            if let Some(infix) = ast::opr::GeneralizedInfix::try_new(ast) {
                name_for_ast(infix.opr.ast())
            } else if let Some(prefix) = ast::prefix::Chain::try_new(ast) {
                name_for_ast(&prefix.func)
            } else {
                "var".into()
            }
        }
    }
}



// ==================
// === Controller ===
// ==================

/// Handle providing graph controller interface.
#[derive(Clone,CloneRef,Debug)]
pub struct Handle {
    /// Model of the module which this graph belongs to.
    pub module : Rc<model::Module>,
    parser : Parser,
    id     : Rc<Id>,
    logger : Logger,
}

impl Handle {

    /// Creates a new controller. Does not check if id is valid.
    ///
    /// Requires global executor to spawn the events relay task.
    pub fn new_unchecked(module:Rc<model::Module>, parser:Parser, id:Id) -> Handle {
        let id = Rc::new(id);
        let logger = Logger::new(format!("Graph Controller {}", id));
        Handle {module,parser,id,logger}
    }

    /// module. Fails if ID cannot be resolved.
    ///
    /// Requires global executor to spawn the events relay task.
    pub fn new(module:Rc<model::Module>, parser:Parser, id:Id) -> FallibleResult<Handle> {
        let ret = Self::new_unchecked(module,parser,id);
        // Get and discard definition info, we are just making sure it can be obtained.
        let _ = ret.graph_definition_info()?;
        Ok(ret)
    }

    /// Retrieves double rep information about definition providing this graph.
    pub fn graph_definition_info
    (&self) -> FallibleResult<double_representation::definition::DefinitionInfo> {
        self.module.find_definition(&self.id)
    }

    /// Returns double rep information about all nodes in the graph.
    pub fn all_node_infos
    (&self) -> FallibleResult<Vec<double_representation::node::NodeInfo>> {
        let definition = self.graph_definition_info()?;
        let graph      = double_representation::graph::GraphInfo::from_definition(definition);
        Ok(graph.nodes())
    }

    /// Retrieves double rep information about node with given ID.
    pub fn node_info
    (&self, id:ast::Id) -> FallibleResult<double_representation::node::NodeInfo> {
        let nodes = self.all_node_infos()?;
        let node  = nodes.into_iter().find(|node_info| node_info.id() == id);
        node.ok_or_else(|| NodeNotFound(id).into())
    }

    /// Gets information about node with given id.
    ///
    /// Note that it is more efficient to use `get_nodes` to obtain all information at once,
    /// rather then repeatedly call this method.
    pub fn node(&self, id:ast::Id) -> FallibleResult<Node> {
        let info     = self.node_info(id)?;
        let metadata = self.module.node_metadata(id).ok();
        Ok(Node {info,metadata})
    }

    /// Returns information about all the nodes currently present in this graph.
    pub fn nodes(&self) -> FallibleResult<Vec<Node>> {
        let node_infos = self.all_node_infos()?;
        let mut nodes  = Vec::new();
        for info in node_infos {
            let metadata = self.module.node_metadata(info.id()).ok();
            nodes.push(Node {info,metadata})
        }
        Ok(nodes)
    }

    /// Returns information about all the connections between graph's nodes.
    pub fn connections(&self) -> FallibleResult<Connections> {
        let definition  = self.graph_definition_info()?;
        let graph       = double_representation::graph::GraphInfo::from_definition(definition);
        Ok(Connections::new(&graph))
    }

    /// Suggests a name for a variable that shall store the node value.
    ///
    /// Analyzes the expression, e.g. result for "a+b" shall be named "sum".
    /// The caller should make sure that obtained name won't collide with any symbol usage before
    /// actually introducing it. See `variable_name_for`.
    pub fn variable_name_base_for(node:&NodeInfo) -> String {
        name_for_ast(node.expression())
    }

    /// Identifiers introduced or referred to in the current graph's scope.
    ///
    /// Introducing identifier not included on this list should have no side-effects on the name
    /// resolution in the code in this graph.
    pub fn used_names(&self) -> FallibleResult<impl Iterator<Item=LocatedName>> {
        let def    = self.graph_definition_info()?;
        let block  = ast::known::Block::try_from(def.body())?;
        let usage  = double_representation::alias_analysis::analyse_block(&block);
        let idents = usage.introduced.into_iter().chain(usage.used.into_iter());
        Ok(idents)
    }

    /// Suggests a variable name for storing results of the given node. Name will get a number
    /// appended to avoid conflicts with other identifiers used in the graph.
    pub fn variable_name_for(&self, node:&NodeInfo) -> ast::known::Var {
        let obtain_name = || -> FallibleResult<String> {
            let base_name   = Self::variable_name_base_for(node);
            let unavailable = self.used_names()?.filter_map(|name| {
                let is_relevant = name.item.starts_with(base_name.as_str());
                is_relevant.then(name.item)
            }).collect::<HashSet<_>>();

            for i in 1.. {
                let candidate = NormalizedName::new(iformat!("{base_name}{i}"));
                if !unavailable.contains(&candidate) {
                    return Ok(candidate.deref().clone())
                }
            }
            panic!("unreachable")
        };

        let name = obtain_name().unwrap_or_else(|_| {
            let u = uuid::Uuid::new_v4();
            let f = u.to_simple().to_string();
            iformat!("var_{u.to_simple()}")
        });


        ast::known::Var::new(ast::Var {name}, None)
    }

    /// Converts node to an assignment, where the whole value is bound to a single identifier.
    /// Modifies the node, discarding any previously set pattern.
    /// Returns the identifier with the node's expression value.
    pub fn introduce_name_on(&self, id:node::Id) -> FallibleResult<ast::known::Var> {
        let mut ret = None;
        self.update_node(id, |mut node| {
            let name = self.variable_name_for(&node);
            node.set_pattern(name.ast().clone());
            ret = Some(name);
            node
        })?;
        // TODO
        Ok(ret.unwrap())
    }

    pub fn connect(&self, connection:&Connection) -> FallibleResult<()> {

        assert!(connection.source.crumbs.tail.is_empty()); // TODO lift this assert and properly support
        assert!(connection.destination.crumbs.tail.is_empty()); // TODO lift this assert and properly support

        let source_node = self.node_info(connection.source.node)?;
        let source_ast = if let Some(pat) = source_node.pattern() {
            pat
        } else {
            self.introduce_name_on(connection.source.node)?;
            return self.connect(connection);
        };
        let source_node_outputs = SpanTree::new(source_ast)?;
        let source_crumbs = &connection.source.crumbs;
        let source_port = source_node_outputs.root_ref().traverse_subnode(source_crumbs.head.clone()).expect("failed locate crumb");

        let source_crumbs = &source_port.ast_crumbs; //source_node_outputs.convert_to_ast_crumbs(&connection.source.crumbs).unwrap();
        let source_identifier = source_ast.get_traversing(source_crumbs)?;


        let destination_node = self.node_info(connection.destination.node)?;
        let destination_ast = destination_node.expression();
        let destination_node_inputs = SpanTree::new(destination_ast)?;
        let destination_port = destination_node_inputs.root_ref().traverse_subnode(connection.destination.crumbs.head.clone()).unwrap();
        let destination_crumbs = &destination_port.ast_crumbs; // destination_node_outputs.convert_to_ast_crumbs(&connection.destination.crumbs).unwrap();
        let destination_identifier = destination_ast.get_traversing(destination_crumbs)?;

        let replaced_destination = destination_port.set(destination_ast,source_identifier.clone()).unwrap();
        let new_expression = replaced_destination;


        println!("Connecting from {} to {}", source_identifier.repr(), destination_identifier.repr());
        println!("New expression {}: ", new_expression.repr());

        self.set_expression_ast(destination_node.id(),new_expression)
    }

    pub fn disconnect(&self, connection:&Connection) -> FallibleResult<()> {
        let destination_node = self.node_info(connection.destination.node)?;
        let destination_ast = destination_node.expression();
        let destination_node_inputs = SpanTree::new(destination_ast)?;
        let destination_port = destination_node_inputs.root_ref().traverse_subnode(connection.destination.crumbs.head.clone()).unwrap();

        let placeholder = Ast::var("x");
        let replaced_destination = if connection.destination.crumbs.tail.is_empty() {
            if destination_port.is_action_available(Action::Erase) {
                destination_port.erase(destination_ast)
            } else {
                destination_port.set(destination_ast,placeholder)
            }
        } else {
            let crumbs = destination_port.ast_crumbs.iter().chain(connection.destination.crumbs.tail.iter()).cloned().collect_vec();
            destination_ast.set_traversing(&crumbs,placeholder)
        }?;

        self.set_expression_ast(destination_node.id(),replaced_destination)
    }

    /// Updates the AST of the definition of this graph.
    pub fn update_definition_ast<F>(&self, f:F) -> FallibleResult<()>
    where F:FnOnce(definition::DefinitionInfo) -> FallibleResult<definition::DefinitionInfo> {
        let ast_so_far     = self.module.ast();
        let definition     = definition::locate(&ast_so_far, &self.id)?;
        let new_definition = f(definition.item)?;
        trace!(self.logger, "Applying graph changes onto definition");
        let new_ast    = new_definition.ast.into();
        let new_module = ast_so_far.set_traversing(&definition.crumbs,new_ast)?;
        self.module.update_ast(new_module);
        Ok(())
    }

    /// Parses given text as a node expression.
    pub fn parse_node_expression
    (&self, expression_text:impl Str) -> FallibleResult<Ast> {
        let node_ast      = self.parser.parse_line(expression_text.as_ref())?;
        if ast::opr::is_assignment(&node_ast) {
            Err(BindingExpressionNotAllowed(expression_text.into()).into())
        } else {
            Ok(node_ast)
        }
    }

    /// Adds a new node to the graph and returns information about created node.
    pub fn add_node(&self, node:NewNodeInfo) -> FallibleResult<ast::Id> {
        trace!(self.logger, "Adding node with expression `{node.expression}`");
        let ast           = self.parse_node_expression(&node.expression)?;
        let mut node_info = node::NodeInfo::from_line_ast(&ast).ok_or(FailedToCreateNode)?;
        if let Some(desired_id) = node.id {
            node_info.set_id(desired_id)
        }

        self.update_definition_ast(|definition| {
            let mut graph = GraphInfo::from_definition(definition);
            let node_ast  = node_info.ast().clone();
            graph.add_node(node_ast,node.location_hint)?;
            Ok(graph.source)
        })?;

        if let Some(initial_metadata) = node.metadata {
            self.module.set_node_metadata(node_info.id(),initial_metadata);
        }

        Ok(node_info.id())
    }

    /// Removes the node with given Id.
    pub fn remove_node(&self, id:ast::Id) -> FallibleResult<()> {
        trace!(self.logger, "Removing node {id}");
        self.update_definition_ast(|definition| {
            let mut graph = GraphInfo::from_definition(definition);
            graph.remove_node(id)?;
            Ok(graph.source)
        })?;

        // It's fine if there were no metadata.
        let _ = self.module.remove_node_metadata(id);
        Ok(())
    }

    /// Sets the given's node expression.
    pub fn set_expression(&self, id:ast::Id, expression_text:impl Str) -> FallibleResult<()> {
        //trace!(self.logger, "Setting node {id} expression to `{expression_text.as_ref()}`");
        let new_expression_ast = self.parse_node_expression(expression_text)?;
        self.set_expression_ast(id,new_expression_ast)
    }

    /// Sets the given's node expression.
    pub fn set_expression_ast(&self, id:ast::Id, expression:Ast) -> FallibleResult<()> {
        trace!(self.logger, "Setting node {id} expression to `{expression.repr()}`");
        self.update_definition_ast(|definition| {
            let mut graph = GraphInfo::from_definition(definition);
            graph.edit_node(id,expression)?;
            Ok(graph.source)
        })?;
        Ok(())
    }

    /// Updates the given node in the definition.
    ///
    /// The function `F` is called with the information with the state of the node so far and
    pub fn update_node<F>(&self, id:ast::Id, f:F) -> FallibleResult<()>
    where F : FnOnce(NodeInfo) -> NodeInfo {
        self.update_definition_ast(|definition| {
            let mut graph = GraphInfo::from_definition(definition);
            graph.update_node(id,|node| {
                let new_node = f(node);
                trace!(self.logger, "Setting node {id} line to `{new_node.repr()}`");
                Some(new_node)
            })?;
            Ok(graph.source)
        })?;
        Ok(())
    }

    /// Subscribe to updates about changes in this graph.
    pub fn subscribe(&self) -> impl Stream<Item=notification::Graph> {
        use notification::*;
        let module_sub = self.module.subscribe_graph_notifications();
        module_sub.map(|notification| {
            match notification {
                Graphs::Invalidate => Graph::Invalidate
            }
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::double_representation::definition::DefinitionName;
    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::notification;

    use ast::HasRepr;
    use data::text::Index;
    use data::text::TextChange;
    use json_rpc::test_util::transport::mock::MockTransport;
    use parser::Parser;
    use utils::test::ExpectTuple;
    use wasm_bindgen_test::wasm_bindgen_test;
    use ast::test_utils::expect_shape;
    use crate::double_representation::graph::LocationHint::End;

    struct GraphControllerFixture(TestWithLocalPoolExecutor);
    impl GraphControllerFixture {
        pub fn set_up() -> GraphControllerFixture {
            let nested = TestWithLocalPoolExecutor::set_up();
            Self(nested)
        }

        pub fn run_graph_for_main<Test,Fut>
        (&mut self, code:impl Str, function_name:impl Str, test:Test)
        where Test : FnOnce(controller::Module,Handle) -> Fut + 'static,
              Fut  : Future<Output=()> {
            let code     = code.as_ref();
            let fm       = file_manager_client::Handle::new(MockTransport::new());
            let loc      = controller::module::Location::new("Main");
            let parser   = Parser::new_or_panic();
            let module   = controller::Module::new_mock(loc,code,default(),fm,parser).unwrap();
            let graph_id = Id::new_single_crumb(DefinitionName::new_plain(function_name.into()));
            let graph    = module.graph_controller(graph_id).unwrap();
            self.0.run_task(async move {
                test(module,graph).await
            })
        }

        pub fn run_graph_for<Test,Fut>(&mut self, code:impl Str, graph_id:Id, test:Test)
            where Test : FnOnce(controller::Module,Handle) -> Fut + 'static,
                  Fut  : Future<Output=()> {
            let code   = code.as_ref();
            let fm     = file_manager_client::Handle::new(MockTransport::new());
            let loc    = controller::module::Location::new("Main");
            let parser = Parser::new_or_panic();
            let module = controller::Module::new_mock(loc,code,default(),fm,parser).unwrap();
            let graph  = module.graph_controller(graph_id).unwrap();
            self.0.run_task(async move {
                test(module,graph).await
            })
        }

        pub fn run_inline_graph<Test,Fut>(&mut self, definition_body:impl Str, test:Test)
        where Test : FnOnce(controller::Module,Handle) -> Fut + 'static,
              Fut  : Future<Output=()> {
            assert_eq!(definition_body.as_ref().contains('\n'), false);
            let code = format!("main = {}", definition_body.as_ref());
            let name = "main";
            self.run_graph_for_main(code, name, test)
        }
    }

    #[wasm_bindgen_test]
    fn node_operations() {
        TestWithLocalPoolExecutor::set_up().run_task(async {
            let code   = "main = Hello World";
            let module = model::Module::from_code_or_panic(code,default(),default());
            let parser = Parser::new().unwrap();
            let pos    = model::module::Position {vector:Vector2::new(0.0,0.0)};
            let crumbs = vec![DefinitionName::new_plain("main")];
            let id     = Id {crumbs};
            let graph  = Handle::new(module,parser,id).unwrap();
            let uid    = graph.all_node_infos().unwrap()[0].id();

            graph.module.with_node_metadata(uid, |data| data.position = Some(pos));

            assert_eq!(graph.module.node_metadata(uid).unwrap().position, Some(pos));
        })
    }

    #[wasm_bindgen_test]
    fn graph_controller_notification_relay() {
        let mut test = GraphControllerFixture::set_up();
        test.run_graph_for_main("main = 2 + 2", "main", |module, graph| async move {
            let text_change = TextChange::insert(Index::new(12), "2".into());
            module.apply_code_change(&text_change).unwrap();

            let mut sub = graph.subscribe();
            module.apply_code_change(&TextChange::insert(Index::new(1),"2".to_string())).unwrap();
            assert_eq!(Some(notification::Graph::Invalidate), sub.next().await);
        })
    }

    #[wasm_bindgen_test]
    fn graph_controller_inline_definition() {
        let mut test = GraphControllerFixture::set_up();
        const EXPRESSION: &str = "2+2";
        test.run_inline_graph(EXPRESSION, |_,graph| async move {
            let nodes   = graph.nodes().unwrap();
            let (node,) = nodes.expect_tuple();
            assert_eq!(node.info.expression().repr(), EXPRESSION);
            let id   = node.info.id();
            let node = graph.node(id).unwrap();
            assert_eq!(node.info.expression().repr(), EXPRESSION);
        })
    }

    #[wasm_bindgen_test]
    fn graph_controller_block_definition() {
        let mut test  = GraphControllerFixture::set_up();
        let program = r"
main =
    foo = 2
    print foo";
        test.run_graph_for_main(program, "main", |_, graph| async move {
            let nodes   = graph.nodes().unwrap();
            let (node1,node2) = nodes.expect_tuple();
            assert_eq!(node1.info.expression().repr(), "2");
            assert_eq!(node2.info.expression().repr(), "print foo");
        })
    }

    #[wasm_bindgen_test]
    fn graph_controller_parse_expression() {
        let mut test  = GraphControllerFixture::set_up();
        let program = r"main = 0";
        test.run_graph_for_main(program, "main", |_, graph| async move {
            let foo = graph.parse_node_expression("foo").unwrap();
            assert_eq!(expect_shape::<ast::Var>(&foo), &ast::Var {name:"foo".into()});

            assert!(graph.parse_node_expression("Vec").is_ok());
            assert!(graph.parse_node_expression("5").is_ok());
            assert!(graph.parse_node_expression("5+5").is_ok());
            assert!(graph.parse_node_expression("a+5").is_ok());
            assert!(graph.parse_node_expression("a=5").is_err());
        })
    }

    #[wasm_bindgen_test]
    fn graph_controller_nested_definition() {
        let mut test  = GraphControllerFixture::set_up();
        const PROGRAM:&str = r"main =
    foo a =
        bar b = 5
    print foo";
        let definition = definition::Id::new_plain_names(vec!["main","foo"]);
        test.run_graph_for(PROGRAM, definition, |module, graph| async move {
            let expression = "new_node";
            graph.add_node(NewNodeInfo::new_pushed_back(expression)).unwrap();
            let expected_program = r"main =
    foo a =
        bar b = 5
        new_node
    print foo";
            module.expect_code(expected_program);
        })
    }

    #[wasm_bindgen_test]
    fn graph_controller_doubly_nested_definition() {
        // Tests editing nested definition that requires transforming inline expression into
        // into a new block.
        let mut test  = GraphControllerFixture::set_up();
        // Not using multi-line raw string literals, as we don't want IntelliJ to automatically
        // strip the trailing whitespace in the lines.
        const PROGRAM:&str = "main =\n    foo a =\n        bar b = 5\n    print foo";
        let definition = definition::Id::new_plain_names(vec!["main","foo","bar"]);
        test.run_graph_for(PROGRAM, definition, |module, graph| async move {
            let expression = "new_node";
            graph.add_node(NewNodeInfo::new_pushed_back(expression)).unwrap();
            let expected_program = "main =\n    foo a =\n        bar b = \
                                    \n            5\n            new_node\n    print foo";
            module.expect_code(expected_program);
        })
    }

    #[wasm_bindgen_test]
    fn graph_controller_node_operations_node() {
        let mut test  = GraphControllerFixture::set_up();
        const PROGRAM:&str = r"
main =
    foo = 2
    print foo";
        test.run_graph_for_main(PROGRAM, "main", |module, graph| async move {
            // === Initial nodes ===
            let nodes   = graph.nodes().unwrap();
            let (node1,node2) = nodes.expect_tuple();
            assert_eq!(node1.info.expression().repr(), "2");
            assert_eq!(node2.info.expression().repr(), "print foo");


            // === Add node ===
            let id       = ast::Id::new_v4();
            let position = Some(model::module::Position::new(10.0,20.0));
            let metadata = NodeMetadata {position};
            let info     = NewNodeInfo {
                expression    : "a+b".into(),
                metadata      : Some(metadata),
                id            : Some(id),
                location_hint : LocationHint::End,
            };
            graph.add_node(info).unwrap();
            let expected_program = r"
main =
    foo = 2
    print foo
    a+b";
            module.expect_code(expected_program);
            let nodes = graph.nodes().unwrap();
            let (_,_,node3) = nodes.expect_tuple();
            assert_eq!(node3.info.id(),id);
            assert_eq!(node3.info.expression().repr(), "a+b");
            let pos = node3.metadata.unwrap().position;
            assert_eq!(pos, position);
            assert!(graph.module.node_metadata(id).is_ok());


            // === Edit node ===
            graph.set_expression(id, "bar baz").unwrap();
            let (_,_,node3) = graph.nodes().unwrap().expect_tuple();
            assert_eq!(node3.info.id(),id);
            assert_eq!(node3.info.expression().repr(), "bar baz");
            assert_eq!(node3.metadata.unwrap().position, position);


            // === Remove node ===
            graph.remove_node(node3.info.id()).unwrap();
            let nodes = graph.nodes().unwrap();
            let (node1,node2) = nodes.expect_tuple();
            assert_eq!(node1.info.expression().repr(), "2");
            assert_eq!(node2.info.expression().repr(), "print foo");
            assert!(graph.module.node_metadata(id).is_err());

            module.expect_code(PROGRAM);
        })
    }

    #[test]
    fn graph_controller_connections_listing() {
        let mut test  = GraphControllerFixture::set_up();
        const PROGRAM:&str = r"
main =
    x,y = get_pos
    print x
    z = print $ foo y";
        test.run_graph_for_main(PROGRAM, "main", |_, graph| async move {
            let connections = graph.connections().unwrap();

            let (node0,node1,node2) = graph.nodes().unwrap().expect_tuple();
            assert_eq!(node0.info.expression().repr(), "get_pos");
            assert_eq!(node1.info.expression().repr(), "print x");
            assert_eq!(node2.info.expression().repr(), "print $ foo y");

            let c = &connections.connections[0];
            assert_eq!(c.source.node,        node0.info.id());
            assert_eq!(c.source.crumbs,      SplitCrumbs::new_span(vec![0]));
            assert_eq!(c.destination.node,   node1.info.id());
            assert_eq!(c.destination.crumbs, SplitCrumbs::new_span(vec![1]));

            let c = &connections.connections[1];
            assert_eq!(c.source.node, node0.info.id());
            assert_eq!(c.source.crumbs, SplitCrumbs::new_span(vec![2]));
            assert_eq!(c.destination.node, node2.info.id());
            assert_eq!(c.destination.crumbs, SplitCrumbs::new_span(vec![2,1]));
        })
    }

    #[test]
    fn graph_controller_create_connection() {
        let mut test  = GraphControllerFixture::set_up();
        const PROGRAM:&str = r"
main =
    x,y = get_pos
    1+2
    1,2,3";
        test.run_graph_for_main(PROGRAM, "main", |_, graph| async move {
            let connections   = graph.connections().unwrap();
            let (node0,node1,node2) = graph.nodes().unwrap().expect_tuple();

            let tree1 = SpanTree::new(node1.info.expression()).unwrap();
            println!("{:?}", tree1);

            let connection_to_add = Connection {
                source : Endpoint {
                    node : node0.info.id(),
                    crumbs : SplitCrumbs::new_span(vec![0]),
                },
                destination : Endpoint {
                    node : node1.info.id(),
                    crumbs : SplitCrumbs::new_span(vec![2]),
                }
            };
            graph.connect(&connection_to_add);


            let connection_to_add = Connection {
                source : Endpoint {
                    node : node0.info.id(),
                    crumbs : SplitCrumbs::new_span(vec![0]),
                },
                destination : Endpoint {
                    node : node2.info.id(),
                    crumbs : SplitCrumbs::new_span(vec![2,0]), // `2` in `1,2,3`
                }
            };
            graph.connect(&connection_to_add);
        })
    }


    #[test]
    fn graph_controller_create_connection_introducing_var() {
        let mut test  = GraphControllerFixture::set_up();
        const PROGRAM:&str = r"
main =
    calculate
    print _
    calculate1 = calculate2
    calculate5 a calculate4 = calculate4 + calculate3";
        test.run_graph_for_main(PROGRAM, "main", |_, graph| async move {
            assert!(graph.connections().unwrap().connections.is_empty());
            let (node0,node1,_) = graph.nodes().unwrap().expect_tuple();
            let connection_to_add = Connection {
                source : Endpoint {
                    node : node0.info.id(),
                    crumbs : SplitCrumbs::new_span(vec![]),
                },
                destination : Endpoint {
                    node : node1.info.id(),
                    crumbs : SplitCrumbs::new_span(vec![1]), // `_` in `print _`
                }
            };
            graph.connect(&connection_to_add).unwrap();

            println!("=================================\n{}",graph.graph_definition_info().unwrap().ast.repr());
        })
    }

    #[test]
    fn suggested_names() {
        let parser = Parser::new_or_panic();
        let cases = [
            ("a+b",           "sum"),
            ("a-b",           "difference"),
            ("a*b",           "product"),
            ("a/b",           "quotient"),
            ("read 'foo.csv'","read"),
            ("Read 'foo.csv'","read"),
            ("574",           "number"),
            ("'Hello'",       "text"),
            ("'Hello",        "text"),
            ("\"Hello\"",     "text"),
            ("\"Hello",       "text"),
        ];

        for (code,expected_name) in &cases {
            let ast = parser.parse_line(*code).unwrap();
            let node = NodeInfo::from_line_ast(&ast).unwrap();
            let name = Handle::variable_name_base_for(&node);
            assert_eq!(&name,expected_name);
        }
    }

    #[wasm_bindgen_test]
    fn disconnect() {
        let mut test  = GraphControllerFixture::set_up();
        const PROGRAM:&str = r"main =
    from = 3 + 4
    to   = foo from";
        const EXPECTED:&str = r"main =
    from = 3 + 4
    to   = foo x";
        test.run_graph_for_main(PROGRAM, "main", |_, graph| async move {
            let connections = graph.connections().unwrap();
            let connection = connections.connections.first().unwrap();

            graph.disconnect(connection).unwrap();
            assert_eq!(EXPECTED, graph.graph_definition_info().unwrap().ast.repr());
        })
    }
}
