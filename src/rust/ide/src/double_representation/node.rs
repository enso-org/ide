//! Code for node discovery and other node-related tasks.

use crate::prelude::*;

use ast::Ast;
use ast::crumbs::Crumbable;
use ast::known;
use std::cmp::Ordering;
use ast::macros::DocCommentInfo;
use crate::double_representation::{discern_line, LineKind};
use crate::double_representation::definition::ScopeKind;

/// Node Id is the Ast Id attached to the node's expression.
pub type Id = ast::Id;



// =============
// === Error ===
// =============

#[allow(missing_docs)]
#[derive(Clone,Copy,Fail,Debug)]
#[fail(display="Node with ID {} was not found.", id)]
pub struct IdNotFound {pub id:Id}

/// Indices of lines belonging to a node.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct NodeIndex {
    /// Documentation comment line index, if present.
    pub documentation_line : Option<usize>,
    /// Main line is a line that contains the node's expression.
    pub main_line          : usize,
}

impl PartialOrd for NodeIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.main_line.partial_cmp(&other.main_line)
    }
}

impl NodeIndex {
    /// Index for the first line belonging to the node.
    pub fn first(&self) -> usize {
        self.documentation_line.unwrap_or(self.main_line)
    }

    /// Index for the last line belonging to the node.
    pub fn last(&self) -> usize {
        self.main_line
    }

    /// Inclusive range between first and last node's lines.
    ///
    /// Note that while a node can contain at most two lines, they may be interspersed by a
    /// number of blank lines.
    pub fn range(start:NodeIndex, last:NodeIndex) -> RangeInclusive<usize> {
        start.first() ..= last.last()
    }
}



// ===============
// === General ===
// ===============

/// Information about the node coupled with its location within a block.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct LocatedNode {
    /// Line index in the block. Zero for inline definition nodes.
    pub index : NodeIndex,
    #[shrinkwrap(main_field)]
    /// Information about the node.
    pub node  : NodeInfo,
}

/// Tests if given line contents can be seen as node with a given id
pub fn is_main_line_of(line:&ast::BlockLine<Option<Ast>>, id:Id) -> bool {
    let node_info = ExpressionLine::from_block_line(line);
    node_info.contains_if(|node| node.id() == id)
}

/// Searches for `NodeInfo` with the associated `id` index in `lines`.
///
/// Returns an error if the Id is not found.
pub fn locate<'a>
( lines : impl IntoIterator<Item=&'a ast::BlockLine<Option<Ast>>> + 'a
, id    : Id
) -> FallibleResult<LocatedNode> {
    Ok(locate_many(lines, [id])?.remove(&id).unwrap())
}

/// Obtain located node information for multiple nodes in a single pass.
///
/// If any of the looked for nodes is not found, `Err` is returned.
/// Any `Ok(…)` return value is guaranteed to have length equal to `looked_for` argument.
pub fn locate_many<'a>
( lines      : impl IntoIterator<Item=&'a ast::BlockLine<Option<Ast>>> + 'a
, looked_for : impl IntoIterator<Item=Id>
) -> FallibleResult<HashMap<ast::Id,LocatedNode>> {
    let lines_iter = double_representation::definition::enumerate_non_empty_lines(lines);
    let mut looked_for = looked_for.into_iter().collect::<HashSet<_>>();

    let mut ret = HashMap::new();
    let nodes = NodeIterator {lines_iter};
    for node in nodes {
        if looked_for.remove(&node.id()) {
            ret.insert(node.id(), node);
        }

        if looked_for.is_empty() {
            break
        }
    };

    if let Some(id) = looked_for.into_iter().next() {
        Err(IdNotFound{id}.into())
    } else {
        Ok(ret)
    }

}



// ================
// === NodeInfo ===
// ================

/// Iterator over indexed line ASTs that yields nodes.
#[derive(Clone,Debug)]
pub struct NodeIterator<'a, T:Iterator<Item=(usize, &'a Ast)> + 'a> {
    /// Input iterator that yields pairs (line index, line's Ast).
    pub lines_iter : T
}

impl<'a, T:Iterator<Item=(usize, &'a Ast)> + 'a> Iterator for NodeIterator<'a, T> {
    type Item = LocatedNode;

    fn next(&mut self) -> Option<Self::Item> {
        let mut documentation = None;
        while let Some((index,ast)) = self.lines_iter.next() {
            if let Some(documentation_info) = DocCommentInfo::new(ast) {
                documentation = Some((index,documentation_info));
            } else if let Some(main_line) = ExpressionLine::from_line_ast(ast) {
                let (documentation_line,documentation) = match documentation {
                    Some((index,documentation)) => (Some(index),Some(documentation)),
                    None                        => (None,None)
                };

                let ret = LocatedNode {
                    node  : NodeInfo  {documentation,main_line},
                    index : NodeIndex {
                        main_line : index,
                        documentation_line,
                    }
                };
                return Some(ret);
            } else {
                // Non-node entity consumes any previous documentation.
                documentation = None;
            }
        }
        None
    }
}

/// Information about node, including both its main line (i.e. line with expression) and optionally
/// attached documentation comment.
#[derive(Clone,Debug,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct NodeInfo {
    /// Primary node AST that contains node's expression and optional pattern binding.
    #[shrinkwrap(main_field)]
    pub main_line: ExpressionLine,
    /// If the node has doc comment attached, it will be represented here.
    pub documentation: Option<DocCommentInfo>,
}

impl NodeInfo {
    /// Check if a given non-empty line's AST belongs to this node.
    pub fn contains_line(&self, line_ast:&Ast) -> bool {
        // TODO refactor these two lambdas into methods
        let expression_id_matches = || ExpressionLine::from_line_ast(line_ast)
            .as_ref()
            .map(ExpressionLine::id)
            .contains(&self.id());
        let doc_comment_id_matches = || match (self.doc_comment_id(), line_ast.id) {
            (Some(node_doc_id),Some(line_ast_id)) => node_doc_id == line_ast_id,
            _                                     => false,
        };
        expression_id_matches() || doc_comment_id_matches()
    }

    /// TODO should not be needed as a method here
    pub fn doc_comment_id(&self) -> Option<ast::Id> {
        self.documentation.as_ref().and_then(|comment| comment.ast().id())
    }

    pub fn from_single_line_ast(ast:&Ast) -> Option<Self> {
        ExpressionLine::from_line_ast(ast).map(|ast| Self { main_line: ast, documentation:None})
    }

    pub fn nodes_from_lines<'a>(lines:impl IntoIterator<Item=&'a Ast>) -> Vec<NodeInfo> {
        let mut ret = Vec::new();

        let mut lines = lines.into_iter();
        while let Some(line) = lines.next() {
            // Node is either:
            // * documentation comment line followed by the node expression,
            // * node expression line.
            if let Some(doc_comment) = DocCommentInfo::new(line) {
                if let Some(node) = lines.next().and_then(ExpressionLine::from_line_ast) {
                    ret.push(NodeInfo {
                        main_line: node,
                        documentation: Some(doc_comment),
                    })
                } else {
                    // Dangling documentation comments never apply to the nodes, so we ignore them.
                }
            } else if let Some(node) = ExpressionLine::from_line_ast(line) {
                ret.push(NodeInfo {
                    main_line: node,
                    documentation: None,
                })
            } else {
                WARNING!("Line '{line}' is neither a doc comment nor a node.")
            }
        }
        ret
    }

    /// Obtain documentation text.
    pub fn documentation_text(&self) -> Option<String> {
         self.documentation.as_ref().map(|doc| doc.text())
    }
}

/// Description of the node that consists of all information locally available about node.
/// Nodes are required to bear IDs. This enum should never contain an ast of node without id set.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum ExpressionLine {
    /// Code with assignment, e.g. `foo = 2 + 2`
    Binding { infix: known::Infix },
    /// Code without assignment (no variable binding), e.g. `2 + 2`.
    Expression { ast: Ast },
}

impl ExpressionLine {
    /// Tries to interpret the whole binding as a node. Right-hand side will become node's
    /// expression.
    pub fn new_binding(infix:known::Infix) -> Option<ExpressionLine> {
        infix.rarg.id?;
        Some(ExpressionLine::Binding {infix})
    }

    /// Tries to interpret AST as node, treating whole AST as an expression.
    pub fn new_expression(ast:Ast) -> Option<ExpressionLine> {
        ast.id?;
        // TODO what if we are given an assignment.
        Some(ExpressionLine::Expression {ast})
    }

    /// Tries to interpret AST as node, treating whole AST as an expression.
    pub fn from_line_ast(ast:&Ast) -> Option<ExpressionLine> {
        match discern_line(ast, ScopeKind::NonRoot) {
            Some(LineKind::ExpressionPlain{ast})       => Self::new_expression(ast),
            Some(LineKind::ExpressionAssignment {ast}) => Self::new_binding(ast),
            Some(LineKind::Definition {..}) | None     => None,
        }
    }

    /// Tries to interpret AST as node, treating whole AST as an expression.
    pub fn from_block_line(line:&ast::BlockLine<Option<Ast>>) -> Option<ExpressionLine> {
        Self::from_line_ast(line.elem.as_ref()?)
    }

    /// Node's unique ID.
    pub fn id(&self) -> Id {
        // Panic must not happen, as the only available constructors checks that
        // there is an ID present.
        self.expression().id.expect("Node AST must bear an ID")
    }

    /// Updates the node's AST so the node bears the given ID.
    pub fn set_id(&mut self, new_id:Id) {
        match self {
            ExpressionLine::Binding{ref mut infix} => {
                let new_rarg = infix.rarg.with_id(new_id);
                let set      = infix.set(&ast::crumbs::InfixCrumb::RightOperand.into(),new_rarg);
                *infix = set.expect("Internal error: setting infix operand should always \
                                     succeed.");
            }
            ExpressionLine::Expression{ref mut ast} => {
                *ast = ast.with_id(new_id);
            }
        };
    }

    /// AST of the node's expression.
    pub fn expression(&self) -> &Ast {
        match self {
            ExpressionLine::Binding   {infix} => &infix.rarg,
            ExpressionLine::Expression{ast}   => &ast,
        }
    }

    /// AST of the node's pattern (assignment's left-hand side).
    pub fn pattern(&self) -> Option<&Ast> {
        match self {
            ExpressionLine::Binding   {infix} => Some(&infix.larg),
            ExpressionLine::Expression{..}    => None,
        }
    }

    /// Mutable AST of the node's expression. Maintains ID.
    pub fn set_expression(&mut self, expression:Ast) {
        let id = self.id();
        match self {
            ExpressionLine::Binding{ref mut infix}  =>
                infix.update_shape(|infix| infix.rarg = expression),
            ExpressionLine::Expression{ref mut ast} => *ast = expression,
        };
        // Id might have been overwritten by the AST we have set. Now we restore it.
        self.set_id(id);
    }

    /// The whole AST of node.
    pub fn ast(&self) -> &Ast {
        match self {
            ExpressionLine::Binding   {infix} => infix.into(),
            ExpressionLine::Expression{ast}   => ast,
        }
    }

    /// Set the pattern (left side of assignment) for node. If it is an Expression node, the
    /// assignment infix will be introduced.
    pub fn set_pattern(&mut self, pattern:Ast) {
        match self {
            ExpressionLine::Binding {infix} => {
                // Setting infix operand never fails.
                infix.update_shape(|infix| infix.larg = pattern)
            }
            ExpressionLine::Expression {ast} => {
                let infix = ast::Infix {
                    larg : pattern,
                    loff : 1,
                    opr  : Ast::opr("="),
                    roff : 1,
                    rarg : ast.clone(),
                };
                let infix = known::Infix::new(infix, None);
                *self = ExpressionLine::Binding {infix};
            }
        }

    }

    /// Clear the pattern (left side of assignment) for node.
    ///
    /// If it is already an Expression node, no change is done.
    pub fn clear_pattern(&mut self) {
        match self {
            ExpressionLine::Binding {infix} => {
                *self = ExpressionLine::Expression {ast:infix.rarg.clone_ref()}
            }
            ExpressionLine::Expression {..} => {}
        }

    }
}

impl ast::HasTokens for ExpressionLine {
    fn feed_to(&self, consumer:&mut impl ast::TokenConsumer) {
        self.ast().feed_to(consumer)
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use ast::opr::predefined::ASSIGNMENT;

    fn expect_node(ast:Ast, expression_text:&str, id:Id) {
        let node_info = NodeInfo::from_single_line_ast(&ast).expect("expected a node");
        assert_eq!(node_info.expression().repr(),expression_text);
        assert_eq!(node_info.id(), id);
    }

    #[test]
    fn expression_node_test() {
        // expression: `4`
        let id = Id::new_v4();
        let ast = Ast::new(ast::Number { base:None, int: "4".into()}, Some(id));
        expect_node(ast,"4",id);
    }

    #[test]
    fn binding_node_test() {
        // expression: `foo = 4`
        let id = Id::new_v4();
        let number = ast::Number { base:None, int: "4".into()};
        let larg   = Ast::var("foo");
        let rarg   = Ast::new(number, Some(id));
        let ast    = Ast::infix(larg,ASSIGNMENT,rarg);
        expect_node(ast,"4",id);
    }

    #[test]
    fn set_expression_binding() {
        let ast = Ast::infix(Ast::var("foo"),"=",Ast::number(4).with_new_id());
        assert_eq!(ast.repr(), "foo = 4");

        let mut node = NodeInfo::from_single_line_ast(&ast).expect("expected a node");
        let id       = node.id();
        node.set_expression(Ast::var("bar"));
        assert_eq!(node.expression().repr(), "bar");
        assert_eq!(node.ast().repr(), "foo = bar");
        assert_eq!(node.id(), id);
    }

    #[test]
    fn set_expression_plain() {
        let ast = Ast::number(4).with_new_id();
        assert_eq!(ast.repr(), "4");

        let mut node = NodeInfo::from_single_line_ast(&ast).expect("expected a node");
        let id       = node.id();
        node.set_expression(Ast::var("bar"));
        assert_eq!(node.expression().repr(), "bar");
        assert_eq!(node.ast().repr(), "bar");
        assert_eq!(node.id(), id);
    }

    #[test]
    fn clearing_pattern_test() {
        // expression: `foo = 4`
        let id = Id::new_v4();
        let number = ast::Number { base:None, int: "4".into()};
        let larg   = Ast::var("foo");
        let rarg   = Ast::new(number, Some(id));
        let ast    = Ast::infix(larg,ASSIGNMENT,rarg);

        let mut node = NodeInfo::from_single_line_ast(&ast).unwrap();
        assert_eq!(node.repr(),"foo = 4");
        assert_eq!(node.id(),id);
        node.clear_pattern();
        assert_eq!(node.repr(),"4");
        assert_eq!(node.id(),id);
        node.clear_pattern();
        assert_eq!(node.repr(),"4");
        assert_eq!(node.id(),id);
    }

    #[test]
    fn setting_pattern_on_expression_node_test() {
        let id       = uuid::Uuid::new_v4();
        let line_ast = Ast::number(2).with_id(id);
        let mut node = NodeInfo::from_single_line_ast(&line_ast).unwrap();
        assert_eq!(node.repr(), "2");
        assert_eq!(node.id(),id);

        node.set_pattern(Ast::var("foo"));

        assert_eq!(node.repr(), "foo = 2");
        assert_eq!(node.id(),id);
    }

    #[test]
    fn setting_pattern_on_binding_node_test() {
        let id       = uuid::Uuid::new_v4();
        let larg     = Ast::var("foo");
        let rarg     = Ast::var("bar").with_id(id);
        let line_ast = Ast::infix(larg,ASSIGNMENT,rarg);
        let mut node = NodeInfo::from_single_line_ast(&line_ast).unwrap();

        assert_eq!(node.repr(), "foo = bar");
        assert_eq!(node.id(),id);

        node.set_pattern(Ast::var("baz"));

        assert_eq!(node.repr(), "baz = bar");
        assert_eq!(node.id(),id);
    }
}
