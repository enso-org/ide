//! This module contains all structures related to Searcher Controller.
pub mod action;

use crate::prelude::*;

use crate::controller::graph::FailedToCreateNode;
use crate::controller::graph::NewNodeInfo;
use crate::double_representation::graph::GraphInfo;
use crate::double_representation::graph::LocationHint;
use crate::double_representation::module::QualifiedName;
use crate::double_representation::node::NodeInfo;
use crate::model::module::MethodId;
use crate::model::module::NodeMetadata;
use crate::model::module::Position;
use crate::notification;

use data::text::TextLocation;
use enso_protocol::language_server;
use flo_stream::Subscriber;
use parser::Parser;

pub use action::Action;




// ==============
// === Errors ===
// ==============


#[allow(missing_docs)]
#[fail(display="No action entry with the index {}.", index)]
#[derive(Copy,Clone,Debug,Fail)]
pub struct NoSuchAction {
    index : usize,
}

#[allow(missing_docs)]
#[fail(display="Action entry with index {} is not a suggestion.", index)]
#[derive(Copy,Clone,Debug,Fail)]
pub struct NotASuggestion {
    index : usize,
}

#[allow(missing_docs)]
#[fail(display="An action cannot be executed when searcher is in \"edit node\" mode.")]
#[derive(Copy,Clone,Debug,Fail)]
pub struct CannotExecuteWhenEditingNode;


// =====================
// === Notifications ===
// =====================

/// The notification emitted by Searcher Controller
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Notification {
    /// A new Suggestion list is available.
    NewActionList
}



// ===================
// === Suggestions ===
// ===================

/// List of actions available in Searcher.
#[derive(Clone,CloneRef,Debug)]
pub enum Actions {
    /// The action list is still gathering suggestions from the Language Server.
    Loading,
    /// The action list is ready.
    #[allow(missing_docs)]
    Loaded {
        list : Rc<action::List>
    },
    /// Loading suggestions from the Language Server resulted in error.
    Error(Rc<failure::Error>)
}

impl Actions {
    /// Check if list is still loading.
    pub fn is_loading(&self) -> bool {
        matches!(self,Self::Loading)
    }

    /// Check if retrieving suggestion list was unsuccessful
    pub fn is_error(&self) -> bool {
        matches!(self,Self::Error(_))
    }

    /// Get the list of actions. Returns None if still loading or error was returned.
    pub fn list(&self) -> Option<&action::List> {
        match self {
            Self::Loaded {list} => Some(list),
            _                   => None,
        }
    }
}

impl Default for Actions {
    fn default() -> Self {
        Self::Loading
    }
}



// ===================
// === Input Parts ===
// ===================

/// An identification of input fragment filled by picking suggestion.
///
/// Essentially, this is a crumb for ParsedInput's expression.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
#[allow(missing_docs)]
pub enum CompletedFragmentId {
    /// The called "function" part, defined as a `func` element in Prefix Chain
    /// (see `ast::prefix::Chain`).
    Function,
    /// The `id`th argument of the called function.
    Argument{index:usize}
}

/// The enum summarizing what user is currently doing basing on the searcher input.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum UserAction {
    /// User is about to type/chose function (the input is likely empty).
    StartingTypingFunction,
    /// User is about to type next argument.
    StartingTypingArgument,
    /// User is in the middle of typing function.
    TypingFunction,
    /// User is in the middle of typing argument.
    TypingArgument,
}

/// A Searcher Input which is parsed to the _expression_ and _pattern_ parts.
///
/// We parse the input for better understanding what user wants to add.
#[derive(Clone,Debug,Default)]
pub struct ParsedInput {
    /// The part of input which is treated as completed function and some set of arguments.
    ///
    /// The expression is kept as prefix chain, as it allows us to easily determine what kind of
    /// entity we can put at this moment (is it a function or argument? What type of the argument?).
    pub expression : Option<ast::Shifted<ast::prefix::Chain>>,
    /// An offset between expression and pattern.
    pub pattern_offset : usize,
    /// The part of input being a function/argument which is still typed by user. It is used
    /// for filtering actions.
    pub pattern : String,
}

impl ParsedInput {
    /// Constructor from the plain input.
    fn new(input:impl Into<String>, parser:&Parser) -> FallibleResult<Self> {
        let mut input      = input.into();
        let leading_spaces = input.chars().take_while(|c| *c == ' ').count();
        // To properly guess what is "still typed argument" we simulate type of one letter by user.
        // This letter will be added to the last argument (or function if there is no argument), or
        // will be a new argument (so the user starts filling a new argument).
        //
        // See also `parsed_input` test to see all cases we want to cover.
        input.push('a');
        let ast        = parser.parse_line(input.trim_start())?;
        let mut prefix = ast::prefix::Chain::from_ast_non_strict(&ast);
        if let Some(last_arg) = prefix.args.pop() {
            let mut last_arg_repr = last_arg.sast.wrapped.repr();
            last_arg_repr.pop();
            Ok(ParsedInput {
                expression     : Some(ast::Shifted::new(leading_spaces,prefix)),
                pattern_offset : last_arg.sast.off,
                pattern        : last_arg_repr,
            })
        } else {
            let mut func_repr = prefix.func.repr();
            func_repr.pop();
            Ok(ParsedInput {
                expression     : None,
                pattern_offset : leading_spaces,
                pattern        : func_repr
            })
        }
    }

    fn new_from_ast(ast:&Ast) -> Self {
        let prefix = ast::prefix::Chain::from_ast_non_strict(&ast);
        ParsedInput {
            expression     : Some(ast::Shifted::new(default(),prefix)),
            pattern_offset : 0,
            pattern        : default(),
        }
    }

    /// Returns the id of the next fragment potentially filled by picking completion suggestion.
    fn next_completion_id(&self) -> CompletedFragmentId {
        match &self.expression {
            None             => CompletedFragmentId::Function,
            Some(expression) => CompletedFragmentId::Argument {index:expression.args.len()},
        }
    }

    /// Get the picked fragment from the Searcher's input.
    pub fn completed_fragment(&self,fragment:CompletedFragmentId) -> Option<String> {
        use CompletedFragmentId::*;
        match (fragment,&self.expression) {
            (_              ,None)       => None,
            (Function       ,Some(expr)) => Some(expr.func.repr()),
            (Argument{index},Some(expr)) => Some(expr.args.get(index)?.sast.wrapped.repr()),
        }
    }

    /// Get the user action basing of this input (see `UserAction` docs).
    pub fn user_action(&self) -> UserAction {
        use UserAction::*;
        let empty_pattern = self.pattern.is_empty();
        match self.next_completion_id() {
            CompletedFragmentId::Function      if empty_pattern => StartingTypingFunction,
            CompletedFragmentId::Function                       => TypingFunction,
            CompletedFragmentId::Argument {..} if empty_pattern => StartingTypingArgument,
            CompletedFragmentId::Argument {..}                  => TypingArgument,
        }
    }

    /// Convert the current input to Prefix Chain representation.
    pub fn as_prefix_chain
    (&self, parser:&Parser) -> Option<ast::Shifted<ast::prefix::Chain>> {
        let parsed_pattern = parser.parse_line(&self.pattern).ok();
        let pattern_sast   = parsed_pattern.map(|p| ast::Shifted::new(self.pattern_offset,p));
        // If there is an expression part of input, we add current pattern as the last argument.
        if let Some(chain) = &self.expression {
            let mut chain = chain.clone();
            if let Some(sast) = pattern_sast {
                let prefix_id = None;
                let argument  = ast::prefix::Argument {sast,prefix_id};
                chain.wrapped.args.push(argument);
            }
            Some(chain)
        // If there isn't any expression part, the pattern is the whole input.
        } else if let Some(sast) = pattern_sast {
            let chain = ast::prefix::Chain::from_ast_non_strict(&sast.wrapped);
            Some(ast::Shifted::new(self.pattern_offset,chain))
        } else {
            None
        }
    }
}

impl HasRepr for ParsedInput {
    fn repr(&self) -> String {
        let mut repr = self.expression.as_ref().map_or("".to_string(), HasRepr::repr);
        repr.extend(itertools::repeat_n(' ',self.pattern_offset));
        repr.push_str(&self.pattern);
        repr
    }
}

impl Display for ParsedInput {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.repr())
    }
}



// ================
// === ThisNode ===
// ================

/// Information about a node that is used as a `this` argument.
///
/// When searcher is brought up with a node selected, the node will be used as "this". This affects
/// suggestions for the first completion (to include methods of the node's returned value) and the
/// code inserted when the input is committed.
#[derive(Clone,Debug)]
pub struct ThisNode {
    /// Identifier of the node that will be connected if the initial suggestion is picked.
    pub id:double_representation::node::Id,
    /// Name of the variable that will be used to connect with the selected node.
    pub var:String,
    /// If the pattern with variable needs to be introduced on the node.
    pub needs_to_introduce_pattern:bool,
}

impl ThisNode {
    /// Retrieve information about the `self` node. The first selected node will be used for this.
    ///
    /// Returns `None` if the given node's information cannot be retrieved or if the node does not
    /// introduce a variable.
    pub fn new(nodes:Vec<double_representation::node::Id>, graph:&controller::Graph)
    -> Option<Self> {
        let id   = *nodes.first()?;
        let node = graph.node(id).ok()?;
        let (var,needs_to_introduce_pattern) = if let Some(ast) = node.info.pattern() {
            // TODO [mwu]
            //   Here we just require that the whole node's pattern is a single var, like
            //   `var = expr`. This prevents using pattern subpart (like `x` in
            //   `Point x y = get_pos`), or basically any node that doesn't stick to `var = expr`
            //   form. If we wanted to support pattern subparts, the engine would need to send us
            //   value updates for matched pattern pieces. See the issue:
            //   https://github.com/enso-org/enso/issues/1038
            (ast::identifier::as_var(&ast)?.to_owned(),false)
        } else {
            (graph.variable_name_for(&node.info).ok()?.repr(),true)
        };
        Some(ThisNode {id,var,needs_to_introduce_pattern})
    }

    /// Introduce a pattern with variable on the node serving as provider of "this" argument.
    ///
    /// Does nothing if node already has a pattern.
    pub fn introduce_pattern(&self, graph:controller::Graph) -> FallibleResult {
        if self.needs_to_introduce_pattern {
            graph.set_pattern_on(self.id,ast::Ast::var(&self.var))?;
        }
        Ok(())
    }
}



// ===========================
// === Searcher Controller ===
// ===========================

/// Describes how Searcher was brought to screen and how should behave when committing expression.
#[derive(Copy,Clone,Debug)]
#[allow(missing_docs)]
pub enum Mode {
    /// Searcher should add a new node at given position.
    NewNode {position:Option<Position>},
    /// Searcher should edit existing node's expression.
    EditNode {node_id:ast::Id},
}

/// A fragment filled by single picked suggestion.
///
/// We store such information in Searcher to better suggest the potential arguments, and to know
/// what imports should be added when inserting node.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct FragmentAddedByPickingSuggestion {
    pub id                : CompletedFragmentId,
    pub picked_suggestion : action::Suggestion,
}

impl FragmentAddedByPickingSuggestion {
    /// Check if the picked fragment is still unmodified by user.
    fn is_still_unmodified
    (&self, input:&ParsedInput, current_module:&QualifiedName) -> bool {
        let expected_code = &self.picked_suggestion.code_to_insert(Some(current_module));
        input.completed_fragment(self.id).contains(expected_code)
    }
}

/// A controller state.
#[derive(Clone,Debug,Default)]
pub struct Data {
    /// The current searcher's input.
    pub input:ParsedInput,
    /// The action list which should be displayed.
    pub actions:Actions,
    /// All fragments of input which were added by picking suggestions. If the fragment will be
    /// changed by user, it will be removed from this list.
    pub fragments_added_by_picking:Vec<FragmentAddedByPickingSuggestion>,
}

impl Data {
    /// Initialize Searcher state when editing node.
    ///
    /// When searcher is brought by editing node, the input should be an expression of this node.
    /// Committing node will then edit the exiting node's expression instead of adding a new one.
    /// Additionally searcher should restore information about intended method, so we will be able
    /// to suggest arguments.
    fn new_with_edited_node
    ( project_name   : &str
    , graph          : &controller::Graph
    , database       : &model::SuggestionDatabase
    , edited_node_id : ast::Id
    ) -> FallibleResult<Self> {
        let edited_node      = graph.node(edited_node_id)?;
        let current_module   = graph.module.path().qualified_module_name(project_name);
        let input            = ParsedInput::new_from_ast(edited_node.info.expression());
        let suggestions      = default();
        let intended_method  = edited_node.metadata.and_then(|md| md.intended_method);
        let initial_entry    = intended_method.and_then(|m| database.lookup_method(m));
        let initial_fragment = initial_entry.and_then(|entry| {
            let fragment = FragmentAddedByPickingSuggestion {
                id                : CompletedFragmentId::Function,
                picked_suggestion : entry
            };
            // This is meant to work with single function calls (without "this" argument).
            // In other case we should know what the function is from the engine, as the function
            // should be resolved.
            fragment.is_still_unmodified(&input,&current_module).and_option(Some(fragment))
        });
        let mut fragments_added_by_picking = Vec::<FragmentAddedByPickingSuggestion>::new();
        initial_fragment.for_each(|f| fragments_added_by_picking.push(f));
        Ok(Data {input, actions: suggestions,fragments_added_by_picking})
    }

    fn find_picked_fragment(&self, id:CompletedFragmentId)
    -> Option<&FragmentAddedByPickingSuggestion> {
        self.fragments_added_by_picking.iter().find(|fragment| fragment.id == id)
    }
}

/// Searcher Controller.
///
/// This is an object providing all required functionalities for Searcher View: mainly it is the
/// action list to display depending on the searcher input, and actions of picking suggestion and
/// executing entry.
///
/// For description of different actions, see `Action` type.
///
/// Each action can be _executed_, and some (like suggestions of code) may be _used as a
/// suggestion_. The exact outcome depends on action type.
///
/// Additionally user can accept the current searcher input as a new node (or new expression of
/// existing node).
#[derive(Clone,CloneRef,Debug)]
pub struct Searcher {
    logger           : Logger,
    data             : Rc<RefCell<Data>>,
    notifier         : notification::Publisher<Notification>,
    graph            : controller::ExecutedGraph,
    mode             : Immutable<Mode>,
    database         : Rc<model::SuggestionDatabase>,
    language_server  : Rc<language_server::Connection>,
    parser           : Parser,
    this_arg         : Rc<Option<ThisNode>>,
    position_in_code : Immutable<TextLocation>,
    project_name     : ImString,
}

impl Searcher {
    /// Create new Searcher Controller.
    pub async fn new
    ( parent         : impl AnyLogger
    , project        : &model::Project
    , method         : language_server::MethodPointer
    , mode           : Mode
    , selected_nodes : Vec<double_representation::node::Id>
    ) -> FallibleResult<Self> {
        let graph = controller::ExecutedGraph::new(&parent,project.clone_ref(),method).await?;
        Self::new_from_graph_controller(parent,project,graph,mode,selected_nodes)
    }

    /// Create new Searcher Controller, when you have Executed Graph Controller handy.
    pub fn new_from_graph_controller
    ( parent         : impl AnyLogger
    , project        : &model::Project
    , graph          : controller::ExecutedGraph
    , mode           : Mode
    , selected_nodes : Vec<double_representation::node::Id>
    ) -> FallibleResult<Self> {
        let logger   = Logger::sub(parent,"Searcher Controller");
        let database = project.suggestion_db();
        let data     = if let Mode::EditNode{node_id} = mode {
            Data::new_with_edited_node(&*project.name(),&graph.graph(),&*database,node_id)?
        } else {
            default()
        };
        let module_ast = graph.graph().module.ast();
        let def_id     = graph.graph().id;
        let def_span   = double_representation::module::definition_span(&module_ast,&def_id)?;
        let position   = TextLocation::convert_span(module_ast.repr(),&def_span).end;
        let this_arg   = Rc::new(matches!(mode, Mode::NewNode{..}).and_option_from(|| ThisNode::new(selected_nodes,&graph.graph())));
        let ret        = Self {
            logger,graph,this_arg,
            data             : Rc::new(RefCell::new(data)),
            notifier         : default(),
            mode             : Immutable(mode),
            database         : project.suggestion_db(),
            language_server  : project.json_rpc(),
            parser           : project.parser(),
            position_in_code : Immutable(position),
            project_name     : project.name(),
        };
        ret.reload_list();
        Ok(ret)
    }

    /// Subscribe to controller's notifications.
    pub fn subscribe(&self) -> Subscriber<Notification> {
        self.notifier.subscribe()
    }

    /// Get the current action list.
    pub fn actions(&self) -> Actions {
        self.data.borrow().actions.clone_ref()
    }

    /// Set the Searcher Input.
    ///
    /// This function should be called each time user modifies Searcher input in view. It may result
    /// in a new action list (the appropriate notification will be emitted).
    pub fn set_input(&self, new_input:String) -> FallibleResult {
        debug!(self.logger, "Manually setting input to {new_input}.");
        let parsed_input = ParsedInput::new(new_input,&self.parser)?;
        let old_expr     = self.data.borrow().input.expression.repr();
        let new_expr     = parsed_input.expression.repr();

        self.data.borrow_mut().input = parsed_input;
        self.invalidate_fragments_added_by_picking();
        if old_expr != new_expr {
            debug!(self.logger, "Reloading list.");
            self.reload_list();
        } else if let Actions::Loaded {list} = self.data.borrow().actions.clone_ref() {
            debug!(self.logger, "Update filtering.");
            list.update_filtering(&self.data.borrow().input.pattern);
            executor::global::spawn(self.notifier.publish(Notification::NewActionList));
        }
        Ok(())
    }

    fn this_var(&self) -> Option<&str> {
        self.this_arg.deref().as_ref().map(|this| this.var.as_ref())
    }

    /// Code that will be inserted by expanding given suggestion at given location.
    ///
    /// Code depends on the location, as the first fragment can introduce `this` variable access,
    /// and then we don't want to put any module name.
    fn code_to_insert
    (&self, suggestion:&action::Suggestion, loc:CompletedFragmentId) -> String {
        let current_module = self.module_qualified_name();
        if loc == CompletedFragmentId::Function && self.this_arg.is_some() {
            suggestion.code_to_insert_skip_module()
        } else {
            suggestion.code_to_insert(Some(&current_module))
        }
    }

    /// Pick a completion suggestion.
    ///
    /// This function should be called when user do the _use as suggestion_ action as a code
    /// suggestion (see struct documentation). The picked suggestion will be remembered, and the
    /// searcher's input will be updated and returned by this function.
    pub fn use_suggestion
    (&self, picked_suggestion: action::Suggestion) -> FallibleResult<String> {
        info!(self.logger, "Picking suggestion: {picked_suggestion:?}");
        let id                = self.data.borrow().input.next_completion_id();
        let code_to_insert    = self.code_to_insert(&picked_suggestion,id);
        let added_ast         = self.parser.parse_line(&code_to_insert)?;
        let picked_completion = FragmentAddedByPickingSuggestion {id,picked_suggestion};
        let pattern_offset    = self.data.borrow().input.pattern_offset;
        let new_expression    = match self.data.borrow_mut().input.expression.take() {
            None => {
                let ast = ast::prefix::Chain::from_ast_non_strict(&added_ast);
                ast::Shifted::new(pattern_offset,ast)
            },
            Some(mut expression) => {
                let new_argument = ast::prefix::Argument {
                    sast      : ast::Shifted::new(pattern_offset.max(1),added_ast),
                    prefix_id : default(),
                };
                expression.args.push(new_argument);
                expression
            }
        };
        let new_parsed_input = ParsedInput {
            expression     : Some(new_expression),
            pattern_offset : 1,
            pattern        : "".to_string()
        };
        let new_input = new_parsed_input.repr();
        self.data.borrow_mut().input = new_parsed_input;
        self.data.borrow_mut().fragments_added_by_picking.push(picked_completion);
        self.reload_list();
        Ok(new_input)
    }

    /// Use action at given index as a suggestion. The exact outcome depends on the action's type.
    pub fn use_as_suggestion(&self, index:usize) -> FallibleResult<String> {
        let error      = || NoSuchAction {index};
        let suggestion = {
            let data = self.data.borrow();
            let list = data.actions.list().ok_or_else(error)?;
            list.get_cloned(index).ok_or_else(error)?.action
        };
        match suggestion {
            Action::Suggestion(suggestion) => self.use_suggestion(suggestion),
            _ => Err(NotASuggestion {index}.into())
        }
    }

    /// Execute given action.
    ///
    /// If the action results in adding new node to the graph, or changing an exiting node, its id
    /// will be returned by this function.
    pub fn execute_action(&self, action:Action) -> FallibleResult<Option<ast::Id>> {
        match action {
            Action::Suggestion(suggestion) => {
                self.use_suggestion(suggestion)?;
                self.commit_node().map(Some)
            }
            Action::Example(example) => match *self.mode {
                Mode::NewNode {position} => self.add_example(&example,position).map(Some),
                _                        => Err(CannotExecuteWhenEditingNode.into())
            }
        }
    }

    /// See `execute_action` documentation.
    pub fn execute_action_by_index(&self, index:usize) -> FallibleResult<Option<ast::Id>> {
        let error      = || NoSuchAction {index};
        let action = {
            let data = self.data.borrow();
            let list = data.actions.list().ok_or_else(error)?;
            list.get_cloned(index).ok_or_else(error)?.action
        };
        self.execute_action(action.clone_ref())
    }

    /// Check if the first fragment in the input (i.e. the one representing the called function)
    /// is still unmodified.
    ///
    /// False if it was modified after picking or if it wasn't picked at all.
    pub fn is_function_fragment_unmodified(&self) -> bool {
        let current_module = self.module_qualified_name();
        with(self.data.borrow(), |data| {
            data.fragments_added_by_picking.first().contains_if(|frag| {
                let is_function = frag.id == CompletedFragmentId::Function;
                is_function && frag.is_still_unmodified(&data.input,&current_module)
            })
        })
    }

    /// Commit the current input as a new node expression.
    ///
    /// If the searcher was brought by editing existing node, the input is set as a new node
    /// expression, otherwise a new node is added. This will also add all imports required by
    /// picked suggestions.
    pub fn commit_node(&self) -> FallibleResult<ast::Id> {
        let input_chain = self.data.borrow().input.as_prefix_chain(&self.parser);

        let expression = match (self.this_var(),input_chain) {
            (Some(this_var),Some(input)) =>
                apply_this_argument(this_var,&input.wrapped.into_ast()).repr(),
            (None,Some(input)) => input.wrapped.into_ast().repr(),
            (_,None)           => "".to_owned(),
        };
        let intended_method = self.intended_method();

        let id = match *self.mode {
            Mode::NewNode {position} => {
                let mut new_node  = NewNodeInfo::new_pushed_back(expression);
                new_node.metadata = Some(NodeMetadata {position,intended_method});
                let graph         = self.graph.graph();
                if let Some(this) = self.this_arg.deref().as_ref() {
                    this.introduce_pattern(graph.clone_ref())?;
                }
                graph.add_node(new_node)?
            },
            Mode::EditNode {node_id} => {
                self.graph.graph().set_expression(node_id,expression)?;
                self.graph.graph().module.with_node_metadata(node_id,Box::new(|md| {
                    md.intended_method = intended_method
                }))?;
                node_id
            }
        };
        self.add_required_imports()?;
        Ok(id)
    }

    /// Adds an example to the graph.
    ///
    /// The example piece of code will be inserted as a new function definition, and in current
    /// graph the node calling this function will appear.
    pub fn add_example
    (&self, example:&action::Example, position:Option<Position>) -> FallibleResult<ast::Id> {
        // === Add new function definition ===
        let graph                 = self.graph.graph();
        let mut module            = double_representation::module::Info{ast:graph.module.ast()};
        let graph_definition_name = graph.graph_info()?.source.name.item;
        let new_definition        = example.definition_to_add(&module, &self.parser)?;
        let new_definition_name   = Ast::var(new_definition.name.name.item.clone());
        let new_definition_place  = double_representation::module::Placement::Before(graph_definition_name);
        module.add_method(new_definition,new_definition_place,&self.parser)?;


        // === Add new node ===
        let here             = Ast::var(constants::keywords::HERE);
        let args             = std::iter::empty();
        let node_expression  = ast::prefix::Chain::new_with_this(new_definition_name,here,args);
        let node_expression  = node_expression.into_ast();
        let node             = NodeInfo::new_expression(node_expression).ok_or(FailedToCreateNode)?;
        let graph_definition = double_representation::module::locate(&module.ast,&self.graph.graph().id)?;
        let mut graph_info   = GraphInfo::from_definition(graph_definition.item);
        graph_info.add_node(node.ast().clone_ref(), LocationHint::End)?;
        module.ast          = module.ast.set_traversing(&graph_definition.crumbs, graph_info.ast())?;
        let intended_method = None;


        // === Add imports ===
        let here = self.module_qualified_name();
        for import in example.imports.iter().map(QualifiedName::from_text).filter_map(Result::ok) {
            module.add_module_import(&here, &self.parser, &import);
        }
        graph.module.update_ast(module.ast)?;
        graph.module.set_node_metadata(node.id(),NodeMetadata {position,intended_method})?;

        Ok(node.id())
    }

    fn invalidate_fragments_added_by_picking(&self) {
        let mut data       = self.data.borrow_mut();
        let data           = data.deref_mut();
        let input          = &data.input;
        let current_module = self.module_qualified_name();
        data.fragments_added_by_picking.drain_filter(
            |frag| !frag.is_still_unmodified(input,&current_module)
        );
    }

    fn add_required_imports(&self) -> FallibleResult {
        let data_borrowed = self.data.borrow();
        let fragments     = data_borrowed.fragments_added_by_picking.iter();
        let imports       = fragments.map(|frag| &frag.picked_suggestion.module);
        let mut module    = self.module();
        let here          = self.module_qualified_name();
        for import in imports {
            module.add_module_import(&here, &self.parser, &import);
        }
        self.graph.graph().module.update_ast(module.ast)
    }

    /// Reload Action List.
    ///
    /// The current list will be set as "Loading" and Language Server will be requested for a new
    /// list - once it be retrieved, the new list will be set and notification will be emitted.
    fn reload_list(&self) {
        let this_type    = self.this_arg_type_for_next_completion();
        let return_types = match self.data.borrow().input.next_completion_id() {
            CompletedFragmentId::Function         => vec![],
            CompletedFragmentId::Argument {index} =>
                self.return_types_for_argument_completion(index),
        };
        self.gather_actions_from_engine(this_type,return_types,None);
        self.data.borrow_mut().actions = Actions::Loading;
        executor::global::spawn(self.notifier.publish(Notification::NewActionList));
    }

    /// Get the typename of "this" value for current completion context. Returns `Future`, as the
    /// type information might not have came yet from the Language Server.
    fn this_arg_type_for_next_completion(&self) -> impl Future<Output=Option<String>> {
        let next_id = self.data.borrow().input.next_completion_id();
        let logger  = self.logger.clone_ref();
        let graph   = self.graph.clone_ref();
        let this    = self.this_arg.clone_ref();
        async move {
            let is_function_fragment = next_id == CompletedFragmentId::Function;
            is_function_fragment.then(())?;
            let ThisNode {id,..} = this.deref().as_ref()?;
            let opt_type         = graph.expression_type(*id).await.map(Into::into);
            opt_type.map_none(move || error!(logger, "Failed to obtain type for this node."))
        }
    }

    /// Get the type that suggestions for the next completion should return.
    ///
    /// Generally this corresponds to the type of the currently filled function argument. Returns
    /// empty list if no type could be determined.
    fn return_types_for_argument_completion(&self, arg_index:usize) -> Vec<String> {
        let suggestions = if let Some(intended) = self.intended_function_suggestion() {
            std::iter::once(intended).collect()
        } else {
            self.possible_function_calls()
        };
        suggestions.into_iter().filter_map(|suggestion| {
            let arg_index = arg_index + if self.this_arg.is_some()  { 1 } else { 0 };
            let arg_index = arg_index + if self.has_this_argument() { 1 } else { 0 };
            let parameter = suggestion.arguments.get(arg_index)?;
            Some(parameter.repr_type.clone())
        }).collect()
    }

    fn gather_actions_from_engine
    ( &self
    , this_type    : impl Future<Output=Option<String>> + 'static
    , return_types : impl IntoIterator<Item=String>
    , tags         : Option<Vec<language_server::SuggestionEntryType>>
    ) {
        let ls               = self.language_server.clone_ref();
        let graph            = self.graph.graph();
        let position         = self.position_in_code.deref().into();
        let this             = self.clone_ref();
        let mut return_types = return_types.into_iter().map(Some).collect_vec();
        if return_types.is_empty() {
            return_types.push(None)
        }
        executor::global::spawn(async move {
            let this_type = this_type.await;
            info!(this.logger,"Requesting new suggestion list. Type of `this` is {this_type:?}.");
            let requests  = return_types.into_iter().map(|return_type| {
                info!(this.logger, "Requesting suggestions for returnType {return_type:?}.");
                let file = graph.module.path().file_path();
                ls.completion(file,&position,&this_type,&return_type,&tags)
            });
            let responses = futures::future::join_all(requests).await;
            info!(this.logger,"Received suggestions from Language Server.");
            let new_list = match this.make_action_list(responses) {
                Ok(list)   => Actions::Loaded {list:Rc::new(list)},
                Err(error) => Actions::Error(Rc::new(error))
            };
            this.data.borrow_mut().actions = new_list;
            this.notifier.publish(Notification::NewActionList).await;
        });
    }

    /// Process multiple completion responses from the engine into a single list of suggestion.
    fn make_action_list
    (&self, completion_responses:Vec<json_rpc::Result<language_server::response::Completion>>)
     -> FallibleResult<action::List> {
        let actions = action::List::new();
        if matches!(self.mode.deref(), Mode::NewNode{..}) && self.this_arg.is_none() {
            actions.extend(self.database.iterate_examples().map(Action::Example));
        }
        for response in completion_responses {
            let response = response?;
            let entries  = response.results.iter().filter_map(|id| {
                self.database.lookup(*id)
                    .map(Action::Suggestion)
                    .handle_err(|e| {
                        error!(self.logger,"Response provided a suggestion ID that cannot be \
                        resolved: {e}.")
                    })
            });
            actions.extend(entries);
        }
        actions.update_filtering(&self.data.borrow().input.pattern);
        Ok(actions)
    }

    fn possible_function_calls(&self) -> Vec<action::Suggestion> {
        let opt_result = || {
            let call_ast = self.data.borrow().input.expression.as_ref()?.func.clone_ref();
            let call     = SimpleFunctionCall::try_new(&call_ast)?;
            if let Some(module) = self.module_whose_method_is_called(&call) {
                let name  = call.function_name;
                let entry = self.database.lookup_module_method(name,&module);
                Some(entry.into_iter().collect())
            } else {
                let name     = &call.function_name;
                let module   = self.module_qualified_name();
                let location = *self.position_in_code;
                Some(self.database.lookup_by_name_and_location(name,&module,location))
            }
        };
        opt_result().unwrap_or_default()
    }

    /// For the simple function call checks if the function is called on the module (if it can be
    /// easily determined) and returns the module's qualified name if it is.
    fn module_whose_method_is_called(&self, call:&SimpleFunctionCall) -> Option<QualifiedName> {
        let position        = *self.position_in_code;
        let this_name       = ast::identifier::name(call.this_argument.as_ref()?)?;
        let module_name     = self.module_qualified_name();
        let matching_locals = self.database.lookup_locals_by_name_and_location
            (this_name,&module_name,position);
        let not_local_name = matching_locals.is_empty();
        not_local_name.and_option_from(|| {
            if this_name == constants::keywords::HERE || this_name == module_name.name().deref() {
                Some(module_name)
            } else {
                self.module().iter_imports().find_map(|import| {
                    import.qualified_name().ok().filter(|module| module.name().deref() == this_name)
                })
            }
        })
    }

    /// Get the suggestion that was selected by the user into the function.
    ///
    /// This suggestion shall be used to request better suggestions from the engine.
    pub fn intended_function_suggestion(&self) -> Option<action::Suggestion> {
        let id       = CompletedFragmentId::Function;
        let fragment = self.data.borrow().find_picked_fragment(id).cloned();
        fragment.map(|f| f.picked_suggestion.clone_ref())
    }

    /// Returns the Id of method user intends to be called in this node.
    ///
    /// The method may be picked by user from suggestion, but there are many methods with the same
    /// name.
    fn intended_method(&self) -> Option<MethodId> {
        self.intended_function_suggestion()?.method_id()
    }

    /// If the function fragment has been filled and also contains the initial "this" argument.
    ///
    /// While we maintain chain consisting of target and applied arguments, the target itself might
    /// need to count for a one argument, as it may of form `this.method`.
    fn has_this_argument(&self) -> bool {
        self.data.borrow().input.expression.as_ref()
            .and_then(|expr| ast::opr::as_access_chain(&expr.func))
            .is_some()
    }

    fn module(&self) -> double_representation::module::Info {
        double_representation::module::Info {ast:self.graph.graph().module.ast()}
    }

    fn module_qualified_name(&self) -> QualifiedName {
        self.graph.graph().module.path().qualified_module_name(&*self.project_name)
    }

    /// Get the user action basing of current input (see `UserAction` docs).
    pub fn current_user_action(&self) -> UserAction {
        self.data.borrow().input.user_action()
    }
}

/// A simple function call is an AST where function is a single identifier with optional
/// argument applied by `ACCESS` operator (dot).
struct SimpleFunctionCall {
    this_argument : Option<Ast>,
    function_name : String,
}

impl SimpleFunctionCall {
    fn try_new(call:&Ast) -> Option<Self> {
        if let Some(name) = ast::identifier::name(call) {
            Some(Self {
                this_argument : None,
                function_name : name.to_owned(),
            })
        } else {
            let infix = ast::opr::to_access(call)?;
            let name  = ast::identifier::name(&infix.rarg)?;
            Some(Self {
                this_argument : Some(infix.larg.clone_ref()),
                function_name : name.to_owned(),
            })
        }
    }
}

fn apply_this_argument(this_var:&str, ast:&Ast) -> Ast {
    if let Ok(opr) = ast::known::Opr::try_from(ast) {
        let shape = ast::SectionLeft {
            arg: Ast::var(this_var),
            off: 1,
            opr: opr.into()
        };
        Ast::new(shape,None)
    } else if let Some(mut infix_chain) = ast::opr::Chain::try_new(ast) {
        if let Some(ref mut target) = &mut infix_chain.target {
             target.arg = apply_this_argument(this_var,&target.arg);
        } else {
            infix_chain.target = Some(ast::opr::ArgWithOffset {arg:Ast::var(this_var), offset:1});
        }
        infix_chain.into_ast()
    } else if let Some(mut prefix_chain) = ast::prefix::Chain::from_ast(ast) {
        prefix_chain.func = apply_this_argument(this_var, &prefix_chain.func);
        prefix_chain.into_ast()
    } else {
        let shape = ast::Infix {
            larg : Ast::var(this_var),
            loff : 0,
            opr  : Ast::opr(ast::opr::predefined::ACCESS),
            roff : 0,
            rarg : ast.clone_ref(),
        };
        Ast::new(shape,None)
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::model::suggestion_database::entry::Argument;
    use crate::model::suggestion_database::entry::Kind;
    use crate::model::suggestion_database::entry::Scope;
    use crate::test::mock::data::MAIN_FINISH;
    use crate::test::mock::data::MODULE_NAME;
    use crate::test::mock::data::PROJECT_NAME;

    use enso_protocol::language_server::types::test::value_update_with_type;
    use json_rpc::expect_call;
    use utils::test::traits::*;
    use enso_protocol::language_server::SuggestionId;
    use crate::model::SuggestionDatabase;

    pub fn completion_response(results:&[SuggestionId]) -> language_server::response::Completion {
        language_server::response::Completion {
            results         : results.to_vec(),
            current_version : default(),
        }
    }

    pub fn expect_completion(client:&mut language_server::MockClient, results:&[SuggestionId]) {
        let response = completion_response(results);
        client.expect.completion(|_,_,_,_,_| Ok(response))
    }

    #[derive(Debug,Derivative)]
    #[derivative(Default)]
    struct MockData {
        graph         : controller::graph::executed::tests::MockData,
        /// If the node in `main` function will be selected while opening searcher.
        selected_node : bool,
        #[derivative(Default(value="MAIN_FINISH"))]
        code_location : enso_protocol::language_server::Position,
    }

    impl MockData {
        fn change_main_body(&mut self, line:&str) {
            let code     = dbg!(crate::test::mock::main_from_lines(&[line]));
            let location = data::text::TextLocation::at_document_end(&code);
            // TODO [mwu] Not nice that we ended up with duplicated mock data for code.
            self.graph.module.code = code.clone();
            self.graph.graph.code  = code;
            self.code_location     = location.into();
        }

        fn expect_completion
        ( &self
        , client      : &mut language_server::MockClient
        , self_type   : Option<&str>
        , return_type : Option<&str>
        , result      : &[SuggestionId]
        ) {
            let completion_response = completion_response(result);
            expect_call!(client.completion(
                module      = self.graph.module.path.file_path().clone(),
                position    = self.code_location,
                self_type   = self_type.map(Into::into),
                return_type = return_type.map(Into::into),
                tag         = None
            ) => Ok(completion_response));
        }
    }

    struct Fixture {
        #[allow(dead_code)]
        data     : MockData,
        test     : TestWithLocalPoolExecutor,
        searcher : Searcher,
        entry1   : action::Suggestion,
        entry2   : action::Suggestion,
        entry3   : action::Suggestion,
        entry4   : action::Suggestion,
        entry9   : action::Suggestion,
    }

    impl Fixture {
        fn new_custom<F>(client_setup:F) -> Self
        where F : FnOnce(&mut MockData,&mut language_server::MockClient) {
            info!(DefaultDebugLogger::new("Test"),"1");
            let test       = TestWithLocalPoolExecutor::set_up();
            info!(DefaultDebugLogger::new("Test"),"2");
            let mut data   = MockData::default();
            info!(DefaultDebugLogger::new("Test"),"3");
            let mut client = language_server::MockClient::default();
            info!(DefaultDebugLogger::new("Test"),"4");
            client.require_all_calls();
            info!(DefaultDebugLogger::new("Test"),"5");
            client_setup(&mut data,&mut client);
            info!(DefaultDebugLogger::new("Test"),"6");
            let end_of_code = TextLocation::at_document_end(&data.graph.module.code);
            let code_range  = TextLocation::at_document_begin()..=end_of_code;
            info!(DefaultDebugLogger::new("Test"),"7");
            let graph       = data.graph.controller();
            info!(DefaultDebugLogger::new("Test"),"8");
            let node        = &graph.graph().nodes().unwrap()[0];
            info!(DefaultDebugLogger::new("Test"),"9");
            let this        = ThisNode::new(vec![node.info.id()],&graph.graph());
            info!(DefaultDebugLogger::new("Test"),"10");
            let this        = data.selected_node.and_option(this);
            info!(DefaultDebugLogger::new("Test"),"11");
            let logger      = Logger::new("Searcher");// new_empty
            info!(DefaultDebugLogger::new("Test"),"12");
            let database    = Rc::new(SuggestionDatabase::new_empty(&logger));
            info!(DefaultDebugLogger::new("Test"),"13");
            let module_name = QualifiedName::from_segments(PROJECT_NAME, &[MODULE_NAME]).unwrap();
            info!(DefaultDebugLogger::new("Test"),"14");
            let searcher = Searcher {
                graph,logger,database,
                data             : default(),
                notifier         : default(),
                mode             : Immutable(Mode::NewNode {position:default()}),
                language_server  : language_server::Connection::new_mock_rc(client),
                parser           : Parser::new_or_panic(),
                this_arg         : Rc::new(this),
                position_in_code : Immutable(end_of_code),
                project_name     : ImString::new(&data.graph.graph.project_name),
            };
            let entry1 = model::suggestion_database::Entry {
                name          : "testFunction1".to_string(),
                kind          : Kind::Function,
                module        : crate::test::mock::data::module_qualified_name(),
                arguments     : vec![],
                return_type   : "Number".to_string(),
                documentation : default(),
                self_type     : None,
                scope         : Scope::InModule {range:code_range},
            };
            let entry2 = model::suggestion_database::Entry {
                name : "TestVar1".to_string(),
                kind : Kind::Local,
                ..entry1.clone()
            };
            let entry3 = model::suggestion_database::Entry {
                name          : "testMethod1".to_string(),
                kind          : Kind::Method,
                self_type     : Some(module_name.clone().into()),
                scope         : Scope::Everywhere,
                arguments     : vec![
                    Argument {
                        repr_type     : "Any".to_string(),
                        name          : "this".to_string(),
                        has_default   : false,
                        default_value : None,
                        is_suspended  : false,
                    },
                    Argument {
                        repr_type     : "Number".to_string(),
                        name          : "num_arg".to_string(),
                        has_default   : false,
                        default_value : None,
                        is_suspended  : false,
                    }
                ],
                ..entry1.clone()
            };
            let entry4 = model::suggestion_database::Entry {
                self_type : Some("Test.Test".to_owned().try_into().unwrap()),
                module    : "Test.Test".to_owned().try_into().unwrap(),
                arguments : vec![
                    Argument {
                        repr_type     : "Any".to_string(),
                        name          : "this".to_string(),
                        has_default   : false,
                        default_value : None,
                        is_suspended  : false
                    },
                    Argument {
                        repr_type     : "String".to_string(),
                        name          : "num_arg".to_string(),
                        has_default   : false,
                        default_value : None,
                        is_suspended  : false
                    }
                ],
                ..entry3.clone()
            };
            let entry9 = model::suggestion_database::Entry {
                name : "testFunction2".to_string(),
                arguments     : vec![
                    Argument {
                        repr_type     : "Text".to_string(),
                        name          : "text_arg".to_string(),
                        has_default   : false,
                        default_value : None,
                        is_suspended  : false
                    },
                    Argument {
                        repr_type     : "Number".to_string(),
                        name          : "num_arg".to_string(),
                        has_default   : false,
                        default_value : None,
                        is_suspended  : false
                    },
                ],
                ..entry1.clone()
            };

            searcher.database.put_entry(1,entry1);
            let entry1 = searcher.database.lookup(1).unwrap();
            searcher.database.put_entry(2,entry2);
            let entry2 = searcher.database.lookup(2).unwrap();
            searcher.database.put_entry(3,entry3);
            let entry3 = searcher.database.lookup(3).unwrap();
            searcher.database.put_entry(4,entry4);
            let entry4 = searcher.database.lookup(4).unwrap();
            searcher.database.put_entry(9,entry9);
            let entry9 = searcher.database.lookup(9).unwrap();
            Fixture{data,test,searcher,entry1,entry2,entry3,entry4,entry9}
        }

        fn new() -> Self {
            Self::new_custom(|_,_| {})
        }
    }


    /// Test checks that:
    /// 1) if the selected node is assigned to a single variable (or can be assigned), the list is
    ///    not immediately presented;
    /// 2) instead the searcher model obtains the type information for the selected node and uses it
    ///    to query Language Server for the suggestion list;
    /// 3) The query for argument type takes the this-argument presence into consideration.
    #[wasm_bindgen_test]
    fn loading_list_w_self() {
        let mock_type = crate::test::mock::data::TYPE_NAME;

        /// The case is: `main` contains a single, selected node. Searcher is brought up.
        /// Mock `entry1` suggestion is picked twice.
        struct Case {
            /// The single line of the initial `main` body.
            node_line:&'static str,
            /// If the searcher should enter "connect to this" mode at all and wait for type info.
            sets_this:bool,
        };

        let cases = [
            Case {node_line:"2+2",             sets_this:true },
            Case {node_line:"the_sum = 2 + 2", sets_this:true },
            Case {node_line:"[x,y] = 2 + 2",   sets_this:false},
        ];

        for case in &cases {
            let Fixture { mut test, searcher, entry1, entry9,.. } = Fixture::new_custom(|data,client| {
                data.change_main_body(case.node_line);
                data.selected_node = true;
                // We expect following calls:
                // 1) for the function - with the "this" filled (if the test case says so);
                // 2) for subsequent completions - without "this"
                data.expect_completion(client,case.sets_this.as_some(mock_type),None,&[1,5,9]);
                // If we are about to add the self type, then we expect the second argument first,
                // and then none. Otherwise we expect all arguments starting from the first.
                let expected_types = if case.sets_this {[Some("Number"),None          ]}
                                     else              {[Some("Text")  ,Some("Number")]};

                data.expect_completion(client,None,expected_types[0],&[1,5,9]);
                data.expect_completion(client,None,expected_types[1],&[1,5,9]);
            });

            searcher.reload_list();

            // The suggestion list should stall only if we actually use "this" argument.
            if case.sets_this {
                assert!(searcher.actions().is_loading());
                test.run_until_stalled();
                // Nothing appeared, because we wait for type information for this node.
                assert!(searcher.actions().is_loading());

                let this_node_id = searcher.this_arg.deref().as_ref().unwrap().id;
                let update = value_update_with_type(this_node_id,mock_type);
                searcher.graph.computed_value_info_registry().apply_updates(vec![update]);
                assert!(searcher.actions().is_loading());
            }

            test.run_until_stalled();
            assert!(!searcher.actions().is_loading());
            searcher.use_suggestion(entry9.clone_ref()).unwrap();
            searcher.use_suggestion(entry1.clone_ref()).unwrap();
            let expected_input = format!("{} {} ",entry9.name,entry1.name);
            assert_eq!(searcher.data.borrow().input.repr(),expected_input);
        }
    }

    #[wasm_bindgen_test]
    fn arguments_suggestions_for_picked_method() {
        let mut fixture = Fixture::new_custom(|data,client| {
            data.expect_completion(client,None,Some("Number"),&[20]);
        });
        let Fixture{test,searcher,entry3,..} = &mut fixture;
        searcher.use_suggestion(entry3.clone_ref()).unwrap();
        assert!(searcher.actions().is_loading());
        test.run_until_stalled();
        assert!(!searcher.actions().is_loading());
    }

    #[wasm_bindgen_test]
    fn arguments_suggestions_for_picked_function() {
        let mut fixture = Fixture::new_custom(|data,client| {
            data.expect_completion(client,None,Some("Text"),&[]);   // First arg suggestion.
            data.expect_completion(client,None,Some("Number"),&[]); // Second arg suggestion.
            data.expect_completion(client,None,None,&[]);           // Oversaturated arg position.
        });


        let Fixture{searcher,entry9,..} = &mut fixture;
        searcher.use_suggestion(entry9.clone_ref()).unwrap();
        assert_eq!(searcher.data.borrow().input.repr(),"testFunction2 ");
        searcher.set_input("testFunction2 'foo' ".to_owned()).unwrap();
        searcher.set_input("testFunction2 'foo' 10 ".to_owned()).unwrap();
    }

    #[wasm_bindgen_test]
    fn non_picked_function_arg_suggestions() {
        let mut fixture = Fixture::new_custom(|data,client| {
            data.graph.module.code.insert_str(0,"import Test.Test\n\n");
            data.code_location.line += 2;
            data.expect_completion(client,None,Some("String"),&[1]);
            data.expect_completion(client,None,Some("Number"),&[]);
            data.expect_completion(client,None,Some("Number"),&[]);
            data.expect_completion(client,None,Some("Number"),&[]);
            data.expect_completion(client,None,None,&[1,2,3,4,9]);
        });
        let Fixture{searcher,..} = &mut fixture;

        // Known functions cases
        searcher.set_input("Test.testMethod1 ".to_string()).unwrap();
        searcher.set_input("here.testMethod1 ".to_string()).unwrap();
        searcher.set_input(iformat!("{MODULE_NAME}.testMethod1 ")).unwrap();
        searcher.set_input("testFunction2 \"str\" ".to_string()).unwrap();

        // Unknown functions case
        searcher.set_input("unknownFunction ".to_string()).unwrap();
    }

    #[wasm_bindgen_test]
    fn non_picked_function_arg_suggestion_ambiguous() {
        fn run_case(input:impl Str, setup:impl FnOnce(&mut Fixture)) {
            // In each case we expect that we can pick two methods with the same name, but different
            // second argument, so the controller should call Engine for each type.
            const EXPECTED_REQUESTS:usize                            = 2;
            let requested_types:Rc<RefCell<HashSet<Option<String>>>> = default();
            let requested_types2                                     = requested_types.clone();
            let mut fixture = Fixture::new_custom(move |data,client| {
                data.graph.module.code.insert_str(0,"import Test.Test\n\n");
                data.code_location.line += 2;
                for _ in 0..EXPECTED_REQUESTS {
                    let requested_types = requested_types2.clone();
                    client.expect.completion(move |_path,_position,_self_type,return_type,_tags| {
                        iprintln!("Requested {return_type:?}.");
                        requested_types.borrow_mut().insert(return_type.clone());
                        Ok(completion_response(&[]))
                    });
                }
            });
            setup(&mut fixture);
            let Fixture{test,searcher,..} = &mut fixture;
            searcher.set_input(input.into()).unwrap();
            test.run_until_stalled();
            iprintln!(">>>>> {requested_types.borrow().deref():?}");
            assert_eq!(requested_types.borrow().len(),EXPECTED_REQUESTS);
            assert!(requested_types.borrow().contains(&Some("Number".to_string())));
            assert!(requested_types.borrow().contains(&Some("String".to_string())));
        }

        run_case("testMethod1 (foo bar) ".to_string(), |_|{});
        run_case("(foo bar).testMethod1 ".to_string(), |_|{});
        // Here the "Test" module is shadowed by local, so the call is ambiguous
        run_case("Test.testMethod1 ".to_string(), |fixture|{
            let shadowing = model::suggestion_database::Entry {
                name : "test".to_string(),
                ..(*fixture.entry2).clone()
            };
            fixture.searcher.database.put_entry(133,shadowing)
        });
    }

    #[wasm_bindgen_test]
    fn loading_list() {
        let Fixture{mut test,searcher,entry1,entry9,..} = Fixture::new_custom(|data,client| {
            data.expect_completion(client,None,None,&[1,5,9]);
        });

        let mut subscriber = searcher.subscribe();
        searcher.reload_list();
        assert!(searcher.actions().is_loading());
        test.run_until_stalled();
        let expected_list = vec![Action::Suggestion(entry1), Action::Suggestion(entry9)];
        assert_eq!(searcher.actions().list().unwrap().to_action_vec(), expected_list);
        let notification = subscriber.next().boxed_local().expect_ready();
        assert_eq!(notification, Some(Notification::NewActionList));
    }

    #[wasm_bindgen_test]
    fn parsed_input() {
        let parser = Parser::new_or_panic();

        fn args_reprs(prefix:&ast::prefix::Chain) -> Vec<String> {
            prefix.args.iter().map(|arg| arg.repr()).collect()
        }

        let input  = "";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        assert!(parsed.expression.is_none());
        assert_eq!(parsed.pattern.as_str(), "");

        let input  = "foo";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        assert!(parsed.expression.is_none());
        assert_eq!(parsed.pattern.as_str(), "foo");

        let input  = " foo";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        assert!(parsed.expression.is_none());
        assert_eq!(parsed.pattern_offset,   1);
        assert_eq!(parsed.pattern.as_str(), "foo");

        let input  = "foo  ";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        let expression = parsed.expression.unwrap();
        assert_eq!(expression.off         , 0);
        assert_eq!(expression.func.repr() , "foo");
        assert_eq!(args_reprs(&expression) , Vec::<String>::new());
        assert_eq!(parsed.pattern_offset,   2);
        assert_eq!(parsed.pattern.as_str(), "");

        let input  = "foo bar";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        let expression = parsed.expression.unwrap();
        assert_eq!(expression.off         , 0);
        assert_eq!(expression.func.repr() , "foo");
        assert_eq!(args_reprs(&expression) , Vec::<String>::new());
        assert_eq!(parsed.pattern_offset,   1);
        assert_eq!(parsed.pattern.as_str(), "bar");

        let input  = "foo  bar  baz";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        let expression = parsed.expression.unwrap();
        assert_eq!(expression.off         , 0);
        assert_eq!(expression.func.repr() , "foo");
        assert_eq!(args_reprs(&expression) , vec!["  bar".to_string()]);
        assert_eq!(parsed.pattern_offset  , 2);
        assert_eq!(parsed.pattern.as_str(), "baz");

        let input  = "  foo bar baz ";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        let expression = parsed.expression.unwrap();
        assert_eq!(expression.off         , 2);
        assert_eq!(expression.func.repr() , "foo");
        assert_eq!(args_reprs(&expression), vec![" bar".to_string()," baz".to_string()]);
        assert_eq!(parsed.pattern_offset,   1);
        assert_eq!(parsed.pattern.as_str(), "");

        let input  = "foo bar (baz ";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        let expression = parsed.expression.unwrap();
        assert_eq!(expression.off         , 0);
        assert_eq!(expression.func.repr() , "foo");
        assert_eq!(args_reprs(&expression) , vec![" bar".to_string()]);
        assert_eq!(parsed.pattern_offset,   1);
        assert_eq!(parsed.pattern.as_str(), "(baz ");
    }

    #[wasm_bindgen_test]
    fn picked_completions_list_maintaining() {
        let Fixture{test:_test,searcher,entry1,entry2,..} = Fixture::new_custom(|data,client| {
            data.expect_completion(client,None,None,&[]);
            data.expect_completion(client,None,None,&[]);
            data.expect_completion(client,None,None,&[]);
            data.expect_completion(client,None,None,&[]);
            data.expect_completion(client,None,None,&[]);
            data.expect_completion(client,None,None,&[]);
        });
        let frags_borrow = || Ref::map(searcher.data.borrow(),|d| &d.fragments_added_by_picking);

        // Picking first suggestion.
        let new_input = searcher.use_suggestion(entry1.clone_ref()).unwrap();
        assert_eq!(new_input, "testFunction1 ");
        let (func,) = frags_borrow().iter().cloned().expect_tuple();
        assert_eq!(func.id, CompletedFragmentId::Function);
        assert!(Rc::ptr_eq(&func.picked_suggestion,&entry1));

        // Typing more args by hand.
        searcher.set_input("testFunction1 some_arg pat".to_string()).unwrap();
        let (func,) = frags_borrow().iter().cloned().expect_tuple();
        assert_eq!(func.id, CompletedFragmentId::Function);
        assert!(Rc::ptr_eq(&func.picked_suggestion,&entry1));

        // Picking argument's suggestion.
        let new_input = searcher.use_suggestion(entry2.clone_ref()).unwrap();
        assert_eq!(new_input, "testFunction1 some_arg TestVar1 ");
        let new_input = searcher.use_suggestion(entry2.clone_ref()).unwrap();
        assert_eq!(new_input, "testFunction1 some_arg TestVar1 TestVar1 ");
        let (function,arg1,arg2) = frags_borrow().iter().cloned().expect_tuple();
        assert_eq!(function.id, CompletedFragmentId::Function);
        assert!(Rc::ptr_eq(&function.picked_suggestion,&entry1));
        assert_eq!(arg1.id, CompletedFragmentId::Argument {index:1});
        assert!(Rc::ptr_eq(&arg1.picked_suggestion,&entry2));
        assert_eq!(arg2.id, CompletedFragmentId::Argument {index:2});
        assert!(Rc::ptr_eq(&arg2.picked_suggestion,&entry2));

        // Backspacing back to the second arg.
        searcher.set_input("testFunction1 some_arg TestVar1 TestV".to_string()).unwrap();
        let (picked,arg) = frags_borrow().iter().cloned().expect_tuple();
        assert_eq!(picked.id, CompletedFragmentId::Function);
        assert!(Rc::ptr_eq(&picked.picked_suggestion,&entry1));
        assert_eq!(arg.id, CompletedFragmentId::Argument {index:1});
        assert!(Rc::ptr_eq(&arg.picked_suggestion,&entry2));

        // Editing the picked function.
        searcher.set_input("testFunction2 some_arg TestVar1 TestV".to_string()).unwrap();
        let (arg,) = frags_borrow().iter().cloned().expect_tuple();
        assert_eq!(arg.id, CompletedFragmentId::Argument {index:1});
        assert!(Rc::ptr_eq(&arg.picked_suggestion,&entry2));
    }

    #[wasm_bindgen_test]
    fn applying_this_var() {
        #[derive(Copy,Clone,Debug)]
        struct Case {
            before : &'static str,
            after  : &'static str,
        }

        impl Case {
            fn new(before:&'static str, after:&'static str) -> Self {
                Case{before,after}
            }

            fn run(&self) {
                let parser  = Parser::new_or_panic();
                let ast     = parser.parse_line(self.before).unwrap();
                let new_ast = apply_this_argument("foo",&ast);
                assert_eq!(new_ast.repr(),self.after,"Case {:?} failed: {:?}",self,ast);
            }
        }

        let cases =
            [ Case::new("bar", "foo.bar")
            , Case::new("bar.baz", "foo.bar.baz")
            , Case::new("bar baz", "foo.bar baz")
            , Case::new("+ 2", "foo + 2")
            , Case::new("+ 2 + 3", "foo + 2 + 3")
            , Case::new("+ 2 - 3", "foo + 2 - 3")
            , Case::new("+ bar baz", "foo + bar baz")
            , Case::new("map x-> x.characters.length", "foo.map x-> x.characters.length")
            ];

        for case in &cases { case.run(); }
    }

    #[wasm_bindgen_test]
    fn adding_node_introducing_this_var() {
        struct Case {
            line   : &'static str,
            result : String,
            run    : Box<dyn FnOnce(&mut Fixture)>,
        };

        impl Case {
            fn new(line:&'static str, result:&[&str], run:impl FnOnce(&mut Fixture)+'static )
            -> Self {
                Case {
                    line,
                    result : crate::test::mock::main_from_lines(result),
                    run    : Box::new(run),
                }
            }
        }

        let cases = vec![
            // Completion was picked.
            Case::new("2 + 2",&["sum1 = 2 + 2","sum1.testFunction1"], |f| {
                f.searcher.use_suggestion(f.entry1.clone()).unwrap();
            }),
            // The input was manually written (not picked).
            Case::new("2 + 2",&["sum1 = 2 + 2","sum1.testFunction1"], |f| {
                f.searcher.set_input("testFunction1 ".to_owned()).unwrap();
            }),
            // Completion was picked and edited.
            Case::new("2 + 2",&["sum1 = 2 + 2","sum1.var.testFunction1"], |f| {
                f.searcher.use_suggestion(f.entry1.clone()).unwrap();
                let new_parsed_input = ParsedInput::new("var.testFunction1",&f.searcher.parser);
                f.searcher.data.borrow_mut().input = new_parsed_input.unwrap();
            }),
            // Variable name already present, need to use it. And not break it.
            Case::new("my_var = 2 + 2",&["my_var = 2 + 2","my_var.testFunction1"], |f| {
                f.searcher.use_suggestion(f.entry1.clone()).unwrap();
            }),
            // Variable names unusable (subpatterns are not yet supported).
            // Don't use "this" argument adjustments at all.
            Case::new("[x,y] = 2 + 2",&["[x,y] = 2 + 2","testFunction1"], |f| {
                f.searcher.use_suggestion(f.entry1.clone()).unwrap();
            }),
        ];

        for case in cases.into_iter() {
            let mut fixture = Fixture::new_custom(|data, client| {
                data.selected_node = true;
                data.change_main_body(case.line);
                data.expect_completion(client,None,None,&[]);
            });
            (case.run)(&mut fixture);
            fixture.searcher.commit_node().unwrap();
            let updated_def = fixture.searcher.graph.graph().definition().unwrap().item;
            assert_eq!(updated_def.ast.repr(),case.result);
        }
    }

    #[wasm_bindgen_test]
    fn committing_node() {
        let Fixture{test:_test,mut searcher,entry4,..} = Fixture::new();
        let module                                     = searcher.graph.graph().module.clone_ref();
        // Setup searcher.
        let parser        = Parser::new_or_panic();
        let picked_method = FragmentAddedByPickingSuggestion {
            id                : CompletedFragmentId::Function,
            picked_suggestion : entry4,
        };
        with(searcher.data.borrow_mut(), |mut data| {
            data.fragments_added_by_picking.push(picked_method);
            data.input = ParsedInput::new("Test.testMethod1".to_string(),&parser).unwrap();
        });

        // Add new node.
        let position  = Some(Position::new(4.0, 5.0));
        searcher.mode = Immutable(Mode::NewNode {position});
        searcher.commit_node().unwrap();

        let expected_code = "import Test.Test\nmain = \n    2 + 2\n    Test.testMethod1";
        assert_eq!(module.ast().repr(), expected_code);
        let (node1,node2) = searcher.graph.graph().nodes().unwrap().expect_tuple();
        let expected_intended_method = Some(MethodId {
            module          : "Test.Test".to_string().try_into().unwrap(),
            defined_on_type : "Test.Test".to_string().try_into().unwrap(),
            name            : "testMethod1".to_string(),
        });
        assert_eq!(node2.metadata.unwrap().intended_method, expected_intended_method);

        // Edit existing node.
        searcher.mode = Immutable(Mode::EditNode {node_id:node1.info.id()});
        searcher.commit_node().unwrap();
        let expected_code = "import Test.Test\nmain = \n    Test.testMethod1\n    Test.testMethod1";
        let (node1,_)     = searcher.graph.graph().nodes().unwrap().expect_tuple();
        assert_eq!(node1.metadata.unwrap().intended_method, expected_intended_method);
        assert_eq!(module.ast().repr(), expected_code);
    }

    #[wasm_bindgen_test]
    fn initialized_data_when_editing_node() {
        let Fixture{test:_test,searcher,entry4,..} = Fixture::new();

        let graph    = searcher.graph.graph();
        let (node,)  = graph.nodes().unwrap().expect_tuple();
        let node_id  = node.info.id();
        let database = searcher.database;

        // Node had not intended method.
        let searcher_data = Data::new_with_edited_node(PROJECT_NAME,&graph,&database,node_id).unwrap();
        assert_eq!(searcher_data.input.repr(), node.info.expression().repr());
        assert!(searcher_data.fragments_added_by_picking.is_empty());
        assert!(searcher_data.actions.is_loading());

        // Node had intended method, but it's outdated.
        let intended_method = MethodId {
            module          : "Test.Test".to_string().try_into().unwrap(),
            defined_on_type : "Test.Test".to_string().try_into().unwrap(),
            name            : "testMethod1".to_string()
        };
        graph.module.with_node_metadata(node_id, Box::new(|md| {
            md.intended_method = Some(intended_method);
        })).unwrap();
        let searcher_data = Data::new_with_edited_node(PROJECT_NAME,&graph,&database,node_id).unwrap();
        assert_eq!(searcher_data.input.repr(), node.info.expression().repr());
        assert!(searcher_data.fragments_added_by_picking.is_empty());
        assert!(searcher_data.actions.is_loading());

        // Node had up-to-date intended method.
        graph.set_expression(node_id,"Test.testMethod1 12").unwrap();
        // We set metadata in previous section.
        let searcher_data = Data::new_with_edited_node(PROJECT_NAME,&graph,&database,node_id).unwrap();
        assert_eq!(searcher_data.input.repr(), "Test.testMethod1 12");
        assert!(searcher_data.actions.is_loading());
        let (initial_fragment,) = searcher_data.fragments_added_by_picking.expect_tuple();
        assert!(Rc::ptr_eq(&initial_fragment.picked_suggestion,&entry4))
    }

    #[wasm_bindgen_test]
    fn simple_function_call_parsing() {
        let parser = Parser::new_or_panic();

        let ast  = parser.parse_line("foo").unwrap();
        let call = SimpleFunctionCall::try_new(&ast).expect("Returned None for \"foo\"");
        assert!(call.this_argument.is_none());
        assert_eq!(call.function_name, "foo");

        let ast = parser.parse_line("Main.foo").unwrap();
        let call = SimpleFunctionCall::try_new(&ast).expect("Returned None for \"Main.foo\"");
        assert_eq!(call.this_argument.unwrap().repr(), "Main");
        assert_eq!(call.function_name                , "foo");

        let ast = parser.parse_line("(2 + 3).foo").unwrap();
        let call = SimpleFunctionCall::try_new(&ast).expect("Returned None for \"(2 + 3).foo\"");
        assert_eq!(call.this_argument.unwrap().repr(), "(2 + 3)");
        assert_eq!(call.function_name                , "foo");

        let ast = parser.parse_line("foo + 3").unwrap();
        assert!(SimpleFunctionCall::try_new(&ast).is_none());

        let ast = parser.parse_line("foo bar baz").unwrap();
        assert!(SimpleFunctionCall::try_new(&ast).is_none());

        let ast = parser.parse_line("Main . (foo bar)").unwrap();
        assert!(SimpleFunctionCall::try_new(&ast).is_none());
    }

    #[wasm_bindgen_test]
    fn adding_example() {
        let Fixture{test:_test,searcher,..} = Fixture::new();
        let module                          = searcher.graph.graph().module.clone_ref();
        let example                         = model::suggestion_database::example::Example {
            name          : "Test Example".to_owned(),
            code          : "x = 2 + 2\nx + 4".to_owned(),
            imports       : vec![],
            documentation : "Lorem ipsum".to_owned()
        };
        let expected_code = "test_example1 =\n    x = 2 + 2\n    x + 4\n\n\
            main = \n    2 + 2\n    here.test_example1";
        searcher.add_example(&Rc::new(example),None).unwrap();
        assert_eq!(module.ast().repr(), expected_code);
    }

    #[wasm_bindgen_test]
    fn adding_example_twice() {
        let Fixture{test:_test,searcher,..} = Fixture::new();
        let module                          = searcher.graph.graph().module.clone_ref();
        let example                         = model::suggestion_database::example::Example {
            name          : "Test Example".to_owned(),
            code          : "[1,2,3,4,5]".to_owned(),
            imports       : vec!["Base.Network.Http".to_owned()],
            documentation : "Lorem ipsum".to_owned()
        };
        let expected_code = "import Base.Network.Http\n\
            test_example1 = [1,2,3,4,5]\n\ntest_example2 = [1,2,3,4,5]\n\n\
            main = \n    2 + 2\n    here.test_example1\n    here.test_example2";
        let example = Rc::new(example);
        searcher.add_example(&example,None).unwrap();
        searcher.add_example(&example,None).unwrap();
        assert_eq!(module.ast().repr(), expected_code);
    }
}
