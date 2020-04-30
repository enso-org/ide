//! Graph Controller.
//!
//! This controller provides access to a specific graph. It lives under a module controller, as
//! each graph belongs to some module.

use crate::prelude::*;

use crate::double_representation::alias_analysis::NormalizedName;
use crate::double_representation::alias_analysis::LocatedName;
use crate::double_representation::definition;
pub use crate::double_representation::graph::Id;
use crate::double_representation::graph::GraphInfo;
pub use crate::double_representation::graph::LocationHint;
use crate::double_representation::node;
use crate::double_representation::node::NodeInfo;
use crate::model::module::NodeMetadata;
use crate::notification;

use parser::Parser;
use span_tree::action::Actions;
use span_tree::action::Action;
use span_tree::SpanTree;
use ast::crumbs::InfixCrumb;



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

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Source node {} has no pattern, so it cannot form connections.",node)]
pub struct NoPatternOnNode {
    pub node : node::Id,
}


// ============
// === Node ===
// ============

/// Description of the node with all information available to the graph controller.
#[derive(Clone,Debug)]
pub struct Node {
    /// Information based on AST, from double_representation module.
    pub info : NodeInfo,
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

/// Identifier for ports.
pub type PortId = Vec<span_tree::node::Crumb>;

/// Reference to the port (i.e. the span tree node).
pub type PortRef<'a> = span_tree::node::Ref<'a>;


// === Endpoint

/// Connection endpoint - a port on a node, described using span-tree crumbs.
#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub struct Endpoint {
    pub node : double_representation::node::Id,
    pub port : PortId,
    /// Crumbs which locate the Var in the `port` ast node.
    ///
    /// In normal case this is an empty crumb (which means that the whole span of `port` is the
    /// mentioned Var. However, span tree does not covers all the possible ast of node expression
    /// (e.g. it does not decompose Blocks), but still we want to pass information about connection
    /// to such port and be able to remove it.
    pub var_crumbs: ast::Crumbs,
}

impl Endpoint {
    /// Create endpoint with empty `var_crumbs`.
    pub fn new(node:double_representation::node::Id, port: PortId) -> Self {
        let var_crumbs = default();
        Endpoint{node,port,var_crumbs}
    }
}


// === Connection ===

/// Connection described using span-tree crumbs.
#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub struct Connection {
    pub source      : Endpoint,
    pub destination : Endpoint
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
    pub fn new(node:&NodeInfo) -> Option<NodeTrees> {
        let inputs  = SpanTree::new(node.expression()).ok()?;
        let outputs = if let Some(pat) = node.pattern() {
            Some(SpanTree::new(pat).ok()?)
        } else {
            None
        };
        Some(NodeTrees {inputs,outputs})
    }

    /// Converts AST crumbs (as obtained from double rep's connection endpoint) into the
    /// appriopriate span-tree node reference.
    pub fn get_span_tree_node<'a,'b>(&'a self, ast_crumbs:&'b [ast::Crumb])
    -> Option<span_tree::node::NodeFoundByAstCrumbs<'a,'b>> {
        if let Some(outputs) = self.outputs.as_ref() {
            // Node in assignment form. First crumb decides which span tree to use.
            let tree = match ast_crumbs.get(0) {
                Some(ast::crumbs::Crumb::Infix(InfixCrumb::LeftOperand))  => Some(outputs),
                Some(ast::crumbs::Crumb::Infix(InfixCrumb::RightOperand)) => Some(&self.inputs),
                _ => None,
            };
            tree.and_then(|tree| tree.root_ref().get_descendant_by_ast_crumbs(&ast_crumbs[1..]))
        } else {
            // Expression node - there is only inputs span tree.
            self.inputs.root_ref().get_descendant_by_ast_crumbs(ast_crumbs)
        }
    }
}


// === Connections ===

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
        let span_tree_node = tree.get_span_tree_node(&endpoint.crumbs)?;
        Some(Endpoint{
            node       : endpoint.node,
            port       : span_tree_node.node.crumbs,
            var_crumbs : span_tree_node.ast_crumbs.into(),
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



// =================
// === Utilities ===
// =================

/// Suggests a variable name for storing results of the given expression.
///
/// Name will try to express result of an infix operation (`sum` for `a+b`), kind of literal
/// (`number` for `5`) and target function name for prefix chain.
///
/// The generated name is not unique and might collide with already present identifiers.
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



// ====================
// === EndpointInfo ===
// ====================

/// Helper structure for controller that describes known information about a connection's endpoint.
///
/// Also provides a number of utility functions for connection operations.
#[derive(Clone,Debug)]
pub struct EndpointInfo {
    /// The endpoint descriptor.
    pub endpoint  : Endpoint,
    /// Ast of the relevant node piece (expression or the pattern).
    pub ast       : Ast,
    /// Span tree for the relevant node side (outputs or inputs).
    pub span_tree : SpanTree,
}

impl EndpointInfo {
    /// Construct information about endpoint. Ast must be the node's expression or pattern.
    pub fn new(endpoint:&Endpoint, ast:&Ast) -> FallibleResult<EndpointInfo> {
        Ok(EndpointInfo {
            endpoint  : endpoint.clone(),
            ast       : ast.clone(),
            span_tree : SpanTree::new(ast)?,
        })
    }

    /// Obtains a reference to the port (span tree node) of this endpoint.
    pub fn port(&self) -> FallibleResult<span_tree::node::Ref> {
        self.span_tree.get_node(&self.endpoint.port)
    }

    /// Obtain reference to the parent of the port identified by given crumbs slice.
    pub fn parent_port_of(&self, crumbs:&[span_tree::node::Crumb]) -> Option<PortRef> {
        let parent_crumbs = span_tree::node::parent_crumbs(crumbs);
        parent_crumbs.and_then(|cr| self.span_tree.get_node(cr.iter()).ok())
    }

    /// Iterates over sibling ports located after this endpoint in its chain.
    pub fn chained_ports_after<'a>(&'a self) -> impl Iterator<Item = PortRef> + 'a {
        let parent_port = self.parent_chain_port();
        let ports_after = parent_port.map(move |parent_port|
            parent_port.chain_children_iter().skip_while(move |port|
                port.crumbs != self.endpoint.port
            ).skip(1));

        ports_after.into_iter().flatten()
    }

    /// Obtains parent port. If this port is part of chain, the parent port will be the parent of
    /// the whole chain.
    pub fn parent_chain_port(&self) -> Option<PortRef> {
        // TODO [mwu]
        //  Unpleasant. Likely there should be something in span tree that allows obtaining
        //  sequence of nodes between root and given crumb. Or sth.
        let mut parent_port = self.parent_port_of(&self.endpoint.port);
        while parent_port.contains_if(|p| p.node.kind == span_tree::node::Kind::Chained) {
            parent_port = parent_port.and_then(|p| self.parent_port_of(&p.crumbs));
        }
        parent_port
    }

    /// Ast being the exact endpoint target. Might be more granular than a span tree port.
    pub fn target_ast(&self) -> FallibleResult<&Ast> {
        self.ast.get_traversing(&self.full_ast_crumbs()?)
    }

    /// Full sequence of Ast crumbs identifying endpoint target.
    pub fn full_ast_crumbs(&self) -> FallibleResult<ast::Crumbs> {
        let port       = self.port()?;
        let mut crumbs = port.ast_crumbs;
        crumbs.extend(self.endpoint.var_crumbs.iter().cloned());
        Ok(crumbs)
    }

    /// Sets AST at the given port. Returns new root Ast.
    pub fn set(&self, ast_to_set:Ast) -> FallibleResult<Ast> {
        self.port()?.set(&self.ast,ast_to_set)
    }

    /// Sets AST at the endpoint target. Returns new root Ast. Does not use span tree logic.
    pub fn set_ast(&self, ast_to_set:Ast) -> FallibleResult<Ast> {
        self.ast.set_traversing(&self.full_ast_crumbs()?,ast_to_set)
    }

    /// Erases given port. Returns new root Ast.
    pub fn erase(&self) -> FallibleResult<Ast> {
        self.port()?.erase(&self.ast)
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
    pub fn all_node_infos(&self) -> FallibleResult<Vec<NodeInfo>> {
        let definition = self.graph_definition_info()?;
        let graph      = double_representation::graph::GraphInfo::from_definition(definition);
        Ok(graph.nodes())
    }

    /// Retrieves double rep information about node with given ID.
    pub fn node_info(&self, id:ast::Id) -> FallibleResult<NodeInfo> {
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
    pub fn used_names(&self) -> FallibleResult<Vec<LocatedName>> {
        use double_representation::alias_analysis;
        let def   = self.graph_definition_info()?;
        let body  = def.body();
        let usage = match body.shape() {
            ast::Shape::Block(block) => alias_analysis::analyse_block(&block),
            _ => {
                if let Some(node) = NodeInfo::from_line_ast(&body) {
                    alias_analysis::analyse_node(&node)
                } else {
                    // Generally speaking - impossible. But if there is no node in the definition
                    // body, then there is nothing that could use any symbols, so nothing is used.
                    default()
                }
            }
        };
        Ok(usage.all_identifiers())
    }

    /// Suggests a variable name for storing results of the given node. Name will get a number
    /// appended to avoid conflicts with other identifiers used in the graph.
    pub fn variable_name_for(&self, node:&NodeInfo) -> FallibleResult<ast::known::Var> {
        let base_name   = Self::variable_name_base_for(node);
        let unavailable = self.used_names()?.into_iter().filter_map(|name| {
            let is_relevant = name.item.starts_with(base_name.as_str());
            is_relevant.then(name.item)
        }).collect::<HashSet<_>>();
        let name = (1..).find_map(|i| {
            let candidate              = NormalizedName::new(iformat!("{base_name}{i}"));
            let available              = !unavailable.contains(&candidate);
            available.and_option_from(|| Some(candidate.deref().clone()))
        }).unwrap(); // It always return a value.
        Ok(ast::known::Var::new(ast::Var {name}, None))
    }

    /// Converts node to an assignment, where the whole value is bound to a single identifier.
    /// Modifies the node, discarding any previously set pattern.
    /// Returns the identifier with the node's expression value.
    pub fn introduce_name_on(&self, id:node::Id) -> FallibleResult<ast::known::Var> {
        let node = self.node(id)?;
        let name = self.variable_name_for(&node.info)?;
        self.update_node(id, |mut node| {
            node.set_pattern(name.ast().clone());
            node
        })?;
        Ok(name)
    }

    /// Obtains information for connection's destination endpoint.
    pub fn destination_info(&self, connection:&Connection) -> FallibleResult<EndpointInfo> {
        let destination_node = self.node_info(connection.destination.node)?;
        let target_node_ast  = destination_node.expression();
        EndpointInfo::new(&connection.destination,target_node_ast)
    }

    /// Obtains information about connection's source endpoint.
    pub fn source_info(&self, connection:&Connection) -> FallibleResult<EndpointInfo> {
        let source_node = self.node_info(connection.source.node)?;
        if let Some(pat) = source_node.pattern() {
            EndpointInfo::new(&connection.source,pat)
        } else {
            // For subports we would not have any idea what pattern to introduce. So we fail.
            Err(NoPatternOnNode {node : connection.source.node}.into())
        }
    }

    /// If the node has no pattern, introduces a new pattern with a single variable name.
    pub fn introduce_pattern_if_missing(&self, node:node::Id) -> FallibleResult<Ast> {
        let source_node = self.node_info(node)?;
        if let Some(pat) = source_node.pattern() {
            Ok(pat.clone())
        } else {
            self.introduce_name_on(node).map(|var| var.into())
        }
    }

    /// Create connection in graph.
    pub fn connect(&self, connection:&Connection) -> FallibleResult<()> {
        if connection.source.port.is_empty() {
            // If we create connection from node's expression root, we are able to introduce missing
            // pattern with a new variable.
            self.introduce_pattern_if_missing(connection.source.node)?;
        }

        let source_info              = self.source_info(connection)?;
        let destination_info         = self.destination_info(connection)?;
        let source_identifier        = source_info.target_ast()?.clone();
        let updated_target_node_expr = destination_info.set(source_identifier)?;
        self.set_expression_ast(connection.destination.node, updated_target_node_expr)
    }

    /// Remove the connections from the graph.
    pub fn disconnect(&self, connection:&Connection) -> FallibleResult<()> {
        let info = self.destination_info(connection)?;

        let updated_expression = if connection.destination.var_crumbs.is_empty() {
            let port = info.port()?;
            let only_empty_ports_after = info.chained_ports_after().all(|p| p.node.is_empty());
            if port.is_action_available(Action::Erase) && only_empty_ports_after {
                info.erase()
            } else {
                info.set(Ast::blank())
            }
        } else {
            info.set_ast(Ast::blank())
        }?;

        self.set_expression_ast(connection.destination.node, updated_expression)
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
    use crate::double_representation::node::NodeInfo;
    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::notification;

    use ast::HasRepr;
    use ast::crumbs;
    use data::text::Index;
    use data::text::TextChange;
    use json_rpc::test_util::transport::mock::MockTransport;
    use parser::Parser;
    use utils::test::ExpectTuple;
    use wasm_bindgen_test::wasm_bindgen_test;
    use ast::test_utils::expect_shape;

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
    fn graph_controller_used_names_in_inline_def() {
        let mut test  = GraphControllerFixture::set_up();
        const PROGRAM:&str = r"main = foo";
        test.run_graph_for_main(PROGRAM, "main", |_, graph| async move {
            let expected_name = LocatedName::new_root(NormalizedName::new("foo"));
            let used_names    = graph.used_names().unwrap();
            assert_eq!(used_names, vec![expected_name]);
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

    #[wasm_bindgen_test]
    fn graph_controller_connections_listing() {
        let mut test  = GraphControllerFixture::set_up();
        const PROGRAM:&str = r"
main =
    x,y = get_pos
    print x
    z = print $ foo y
    print z
    foo
        print z";
        test.run_graph_for_main(PROGRAM, "main", |_, graph| async move {
            let connections = graph.connections().unwrap();

            let (node0,node1,node2,node3,node4) = graph.nodes().unwrap().expect_tuple();
            assert_eq!(node0.info.expression().repr(), "get_pos");
            assert_eq!(node1.info.expression().repr(), "print x");
            assert_eq!(node2.info.expression().repr(), "print $ foo y");
            assert_eq!(node3.info.expression().repr(), "print z");

            let c = &connections.connections[0];
            assert_eq!(c.source.node,      node0.info.id());
            assert_eq!(c.source.port,      vec![1]);
            assert_eq!(c.destination.node, node1.info.id());
            assert_eq!(c.destination.port, vec![2]);

            let c = &connections.connections[1];
            assert_eq!(c.source.node     , node0.info.id());
            assert_eq!(c.source.port     , vec![4]);
            assert_eq!(c.destination.node, node2.info.id());
            assert_eq!(c.destination.port, vec![4,2]);

            let c = &connections.connections[2];
            assert_eq!(c.source.node     , node2.info.id());
            assert_eq!(c.source.port     , Vec::<usize>::new());
            assert_eq!(c.destination.node, node3.info.id());
            assert_eq!(c.destination.port, vec![2]);

            use ast::crumbs::*;
            let c = &connections.connections[3];
            assert_eq!(c.source.node     , node2.info.id());
            assert_eq!(c.source.port     , Vec::<usize>::new());
            assert_eq!(c.destination.node, node4.info.id());
            assert_eq!(c.destination.port, vec![2]);
            assert_eq!(c.destination.var_crumbs, crumbs!(BlockCrumb::HeadLine,PrefixCrumb::Arg));
        })
    }

    #[wasm_bindgen_test]
    fn graph_controller_create_connection() {
        /// A case for creating connection test. The field's names are short to be able to write
        /// nice-to-read table of cases without very long lines (see `let cases` below).
        #[derive(Clone,Debug)]
        struct Case {
            /// A pattern (the left side of assignment operator) of source node.
            src      : &'static str,
            /// An expression of destination node.
            dst      : &'static str,
            /// Crumbs of source and destination ports (i.e. SpanTree nodes)
            ports    : (&'static [usize],&'static [usize]),
            /// Expected destination expression after connecting.
            expected : &'static str,
        }

        impl Case {
            fn run(&self) {
                let mut test    = GraphControllerFixture::set_up();
                let main_prefix = format!("main = \n    {} = foo\n    ",self.src);
                let main        = format!("{}{}",main_prefix,self.dst);
                let expected    = format!("{}{}",main_prefix,self.expected);
                let this        = self.clone();

                let (src_port,dst_port) = self.ports;
                let src_port = src_port.to_vec();
                let dst_port = dst_port.to_vec();

                test.run_graph_for_main(main, "main", |_, graph| async move {
                    let (node0,node1) = graph.nodes().unwrap().expect_tuple();
                    let source        = Endpoint::new(node0.info.id(),src_port.to_vec());
                    let destination   = Endpoint::new(node1.info.id(),dst_port.to_vec());
                    let connection    = Connection{source,destination};
                    graph.connect(&connection).unwrap();
                    let new_main = graph.graph_definition_info().unwrap().ast.repr();
                    assert_eq!(new_main,expected,"Case {:?}",this);
                })
            }
        }

        let cases = &
            [ Case {src:"x"      , dst:"foo"      , expected:"x"         , ports:(&[]   ,&[]   )}
            , Case {src:"x,y"    , dst:"foo a"    , expected:"foo y"     , ports:(&[4]  ,&[2]  )}
            , Case {src:"Vec x y", dst:"1 + 2 + 3", expected:"x + 2 + 3" , ports:(&[0,2],&[0,1])}
            ];
        for case in cases {
            case.run()
        }
    }


    #[wasm_bindgen_test]
    fn graph_controller_create_connection_introducing_var() {
        let mut test  = GraphControllerFixture::set_up();
        const PROGRAM:&str = r"main =
    calculate
    print _
    calculate1 = calculate2
    calculate3 calculate5 = calculate5 calculate4";
        // Note: we expect that name `calculate5` will be introduced. There is no conflict with a
        // function argument, as it just shadows outer variable.
        const EXPECTED:&str = r"main =
    calculate5 = calculate
    print calculate5
    calculate1 = calculate2
    calculate3 calculate5 = calculate5 calculate4";
        test.run_graph_for_main(PROGRAM, "main", |_, graph| async move {
            assert!(graph.connections().unwrap().connections.is_empty());
            let (node0,node1,_) = graph.nodes().unwrap().expect_tuple();
            let connection_to_add = Connection {
                source : Endpoint {
                    node      : node0.info.id(),
                    port      : vec![],
                    var_crumbs: vec![]
                },
                destination : Endpoint {
                    node      : node1.info.id(),
                    port      : vec![2], // `_` in `print _`
                    var_crumbs: vec![]
                }
            };
            graph.connect(&connection_to_add).unwrap();
            let new_main = graph.graph_definition_info().unwrap().ast.repr();
            assert_eq!(new_main,EXPECTED);
        })
    }

    #[wasm_bindgen_test]
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
        #[derive(Clone,Debug)]
        struct Case {
            dest_node_expr     : &'static str,
            dest_node_expected : &'static str,
        }

        impl Case {
            fn run(&self) {
                let mut test  = GraphControllerFixture::set_up();
                const MAIN_PREFIX:&str = "main = \n    in = foo\n    ";
                let main     = format!("{}{}",MAIN_PREFIX,self.dest_node_expr);
                let expected = format!("{}{}",MAIN_PREFIX,self.dest_node_expected);
                let this     = self.clone();

                test.run_graph_for_main(main,"main",|_,graph| async move {
                    let connections = graph.connections().unwrap();
                    let connection  = connections.connections.first().unwrap();
                    graph.disconnect(connection).unwrap();
                    let new_main = graph.graph_definition_info().unwrap().ast.repr();
                    assert_eq!(new_main,expected,"Case {:?}",this);
                })
            }
        }

        let cases = &
            [ Case {dest_node_expr:"foo in"             , dest_node_expected:"foo _"              }
            , Case {dest_node_expr:"foo in a"           , dest_node_expected:"foo _ a"            }
            , Case {dest_node_expr:"foo a in"           , dest_node_expected:"foo a"              }
            , Case {dest_node_expr:"in + a"             , dest_node_expected:"_ + a"              }
            , Case {dest_node_expr:"a + in"             , dest_node_expected:"a + _"              }
            , Case {dest_node_expr:"in + b + c"         , dest_node_expected:"_ + b + c"          }
            , Case {dest_node_expr:"a + in + c"         , dest_node_expected:"a + _ + c"          }
            , Case {dest_node_expr:"a + b + in"         , dest_node_expected:"a + b"              }
            , Case {dest_node_expr:"in , a"             , dest_node_expected:"_ , a"              }
            , Case {dest_node_expr:"a , in"             , dest_node_expected:"a , _"              }
            , Case {dest_node_expr:"in , b , c"         , dest_node_expected:"_ , b , c"          }
            , Case {dest_node_expr:"a , in , c"         , dest_node_expected:"a , _ , c"          }
            , Case {dest_node_expr:"a , b , in"         , dest_node_expected:"a , b"              }
            , Case {dest_node_expr:"f\n        bar a in", dest_node_expected: "f\n        bar a _"}
            ];
        for case in cases {
            case.run();
        }
    }
}
