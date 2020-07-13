//! This module contains all structures related to Searcher Controller.

use crate::prelude::*;

use crate::notification;

use data::text::TextLocation;
use enso_protocol::language_server;
use flo_stream::Subscriber;
use parser::Parser;


// =======================
// === Suggestion List ===
// =======================

pub type Completion = Rc<model::suggestion_database::Entry>;

/// A single suggestion on the Searcher suggestion list.
#[derive(Clone,CloneRef,Debug,Eq,PartialEq)]
pub enum Suggestion {
    /// Suggestion for input completion: possible functions, arguments, etc.
    Completion(Completion)
    // In future, other suggestion types will be added (like suggestions of actions, etc.).
}

/// List of suggestions available in Searcher.
#[derive(Clone,CloneRef,Debug)]
pub enum Suggestions {
    /// The suggestion list is still loading from the Language Server.
    Loading,
    /// The suggestion list is loaded.
    #[allow(missing_docs)]
    Loaded {
        list : Rc<Vec<Suggestion>>
    },
    /// Loading suggestion list resulted in error.
    Error(Rc<failure::Error>)
}

impl Suggestions {
    /// Check if suggestion list is still loading.
    pub fn is_loading(&self) -> bool {
        match self {
            Self::Loading => true,
            _             => false,
        }
    }

    /// Check if retrieving suggestion list was unsuccessful
    pub fn is_error(&self) -> bool {
        match self {
            Self::Error(_) => true,
            _              => false,
        }
    }

    /// Get the list of suggestions. Returns None if still loading or error was returned.
    pub fn list(&self) -> Option<&Vec<Suggestion>> {
        match self {
            Self::Loaded {list} => Some(list),
            _                   => None,
        }
    }
}

impl Default for Suggestions {
    fn default() -> Self {
        Self::Loading
    }
}



// =====================
// === Notifications ===
// =====================

/// The notification emitted by Searcher Controller
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Notification {
    /// A new Suggestion list is available.
    NewSuggestionList
}


// ===================
// === Input Parts ===
// ===================

#[derive(Clone,Debug,Default)]
struct ParsedInput {
    expression     : Option<ast::Shifted<ast::prefix::Chain>>,
    pattern_offset : usize,
    pattern        : String,
}

impl ParsedInput {
    fn new(mut input:String, parser:&Parser) -> FallibleResult<Self> {
        let leading_spaces = input.chars().take_while(|c| *c == ' ').count();
        input.push('a');
        let mut prefix = ast::prefix::Chain::new_non_strict(&parser.parse_line(input.trim_start())?);
        let last_arg   = prefix.args.pop();
        if let Some(last_arg) = last_arg{
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
}

impl HasRepr for ParsedInput {
    fn repr(&self) -> String {
        let expr          = self.expression.as_ref().map_or("".to_string(), HasRepr::repr);
        let offset:String = std::iter::repeat(' ').take(self.pattern_offset).collect();
        iformat!("{expr}{offset}{self.pattern}")
    }
}


// ===========================
// === Searcher Controller ===
// ===========================

#[derive(Clone,Debug,Eq,PartialEq)]
enum CompletionId {
    Function, Argument{id:usize}
}

impl CompletionId {
    fn completed_fragment(&self,input:&ParsedInput) -> Option<String> {
        match (self,&input.expression) {
            (_                          , None)       => None,
            (CompletionId::Function     , Some(expr)) => Some(expr.func.repr()),
            (CompletionId::Argument {id}, Some(expr)) => expr.args.get(*id).map(HasRepr::repr),
        }
    }
}

#[derive(Clone,Debug)]
struct PickedCompletion {
    id              : CompletionId,
    completion      : Completion,
}

impl PickedCompletion {
    fn still_unmodified(&self, input:&ParsedInput) -> bool {
        self.id.completed_fragment(input).as_ref() == Some(&self.completion.name)
    }
}

/// A controller state. Currently it caches the currently kept suggestions list and the current
/// searcher input.
#[derive(Clone,Debug,Default)]
struct Data {
    current_input      : ParsedInput,
    current_list       : Suggestions,
    picked_completions : Vec<PickedCompletion>,
}

/// Searcher Controller.
///
/// This is an object providing all required functionalities for Searcher View: mainly it is the
/// suggestion list to display depending on the searcher input, and actions of picking one or
/// accepting the Searcher input (pressing "Enter").
#[derive(Clone,CloneRef,Debug)]
pub struct Searcher {
    logger          : Logger,
    data            : Rc<RefCell<Data>>,
    notifier        : notification::Publisher<Notification>,
    module          : Rc<model::module::QualifiedName>,
    position        : Immutable<TextLocation>,
    database        : Rc<model::SuggestionDatabase>,
    language_server : Rc<language_server::Connection>,
    parser          : Parser,
}

impl Searcher {
    /// Create new Searcher Controller.
    pub fn new
    ( parent   : impl AnyLogger
    , project  : &model::Project
    , module   : model::module::Path
    , position : TextLocation
    ) -> Self {
        let this = Self {
            position        : Immutable(position),
            logger          : Logger::sub(parent,"Searcher Controller"),
            data            : default(),
            notifier        : default(),
            module          : Rc::new(project.qualified_module_name(&module)),
            database        : project.suggestion_db.clone_ref(),
            language_server : project.language_server_rpc.clone_ref(),
            parser          : project.parser.clone_ref(),
        };
        this.reload_list();
        this
    }

    /// Subscribe to controller's notifications.
    pub fn subscribe(&self) -> Subscriber<Notification> {
        self.notifier.subscribe()
    }

    /// Get the current suggestion list.
    pub fn suggestions(&self) -> Suggestions {
        self.data.borrow().current_list.clone_ref()
    }

    /// Set the Searcher Input.
    ///
    /// This function should be called each time user modifies Searcher input in view. It may result
    /// in a new suggestion list (the aprriopriate notification will be emitted).
    pub fn set_input(&self, new_input:String) -> FallibleResult<()> {
        let parsed_input = ParsedInput::new(new_input,&self.parser)?;
        let old_id = self.next_completion();
        self.data.borrow_mut().current_input = parsed_input;
        let new_id = self.next_completion();

        self.invalidate_picked_completions();

        if old_id != new_id {
            self.reload_list()
        }
        Ok(())
    }

    pub fn pick_completion(&self, completion:Completion) -> String {
        let added_string      = &completion.name;
        let added_ast         = ast::Ast::var(added_string);
        let id                = self.next_completion();
        let picked_completion = PickedCompletion{id,completion};
        let pattern_offset    = self.data.borrow().current_input.pattern_offset;
        let new_expression    = match self.data.borrow_mut().current_input.expression.take() {
            None => {
                let ast = ast::prefix::Chain::new_non_strict(&added_ast);
                ast::Shifted::new(pattern_offset,ast)
            },
            Some(mut expression) => {
                let new_argument = ast::prefix::Argument {
                    sast      : ast::Shifted::new(pattern_offset,added_ast),
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
        self.data.borrow_mut().current_input = new_parsed_input;
        self.data.borrow_mut().picked_completions.push(picked_completion);
        self.reload_list();
        new_input
    }

    fn invalidate_picked_completions(&self) {
        let mut data = self.data.borrow_mut();
        let data     = data.deref_mut();
        let input    = &data.current_input;
        data.picked_completions.drain_filter(|compl| compl.still_unmodified(input));
    }

    /// Reload Suggestion List.
    ///
    /// The current list will be set as "Loading" and Language Server will be requested for a new
    /// list - once it be retrieved, the new list will be set and notification will be emitted.
    fn reload_list(&self) {
        let new_list = if self.next_completion() == CompletionId::Function {
            self.get_list_from_engine(None,None);
            Suggestions::Loading
        } else {
            // TODO[ao] Requesting for argument.
            Suggestions::Loaded {list:default()}
        };
        self.data.borrow_mut().current_list = new_list;
    }

    fn get_list_from_engine(&self, return_type:Option<String>, tags:Option<Vec<language_server::SuggestionEntryType>>) {
        let ls          = &self.language_server;
        let module      = self.module.as_ref();
        let self_type   = None;
        let position    = self.position.deref().into();
        let request     = ls.completion(module,&position,&self_type,&return_type,&tags);
        let data        = self.data.clone_ref();
        let database    = self.database.clone_ref();
        let logger      = self.logger.clone_ref();
        let notifier    = self.notifier.clone_ref();
        executor::global::spawn(async move {
            info!(logger,"Requesting new suggestion list.");
            let response = request.await;
            info!(logger,"Received suggestions from Language Server.");
            let new_list = match response {
                Ok(list) => {
                    let entry_ids   = list.results.into_iter();
                    let entries     = entry_ids.filter_map(|id| {
                        let entry = database.get(id);
                        if entry.is_none() {
                            error!(logger,"Missing entry {id} in Suggestion Database.");
                        }
                        entry
                    });
                    let suggestions = entries.map(Suggestion::Completion);
                    Suggestions::Loaded {list:Rc::new(suggestions.collect())}
                },
                Err(error) => Suggestions::Error(Rc::new(error.into()))
            };
            data.borrow_mut().current_list = new_list;
            notifier.publish(Notification::NewSuggestionList).await;
        });
    }

    fn next_completion(&self) -> CompletionId {
        match &self.data.borrow().current_input.expression {
            None             => CompletionId::Function,
            Some(expression) => CompletionId::Argument {id:expression.args.len()},
        }
    }
}


// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::model::module::Path;

    use json_rpc::expect_call;
    use utils::test::traits::*;

    struct Fixture {
        searcher : Searcher,
        entry1   : Completion,
        entry9   : Completion,
    }

    impl Fixture {
        fn new(client_setup:impl FnOnce(&mut language_server::MockClient)) -> Self {
            let mut client = language_server::MockClient::default();
            client_setup(&mut client);
            let searcher = Searcher {
                logger          : default(),
                data            : default(),
                notifier        : default(),
                module          : Rc::new(QualifiedName::from_path(&Path::from_mock_module_name("Test"),"Test")),
                position        : Immutable(TextLocation::at_document_begin()),
                database        : default(),
                language_server : language_server::Connection::new_mock_rc(client),
                parser          : Parser::new_or_panic(),
            };
            let entry1 = model::suggestion_database::Entry {
                name          : "TestFunction1".to_string(),
                kind          : model::suggestion_database::EntryKind::Function,
                module        : "Test.Test".to_string(),
                arguments     : vec![],
                return_type   : "Number".to_string(),
                documentation : default(),
                self_type     : None
            };
            let entry9 = model::suggestion_database::Entry {
                name : "TestFunction2".to_string(),
                ..entry1.clone()
            };
            searcher.database.put_entry(1,entry1);
            let entry1 = searcher.database.get(1).unwrap();
            searcher.database.put_entry(9,entry9);
            let entry9 = searcher.database.get(9).unwrap();
            Fixture{searcher,entry1,entry9}
        }
    }


    #[test]
    fn loading_list() {
        let mut test = TestWithLocalPoolExecutor::set_up();
        let Fixture{searcher,entry1,entry9} = Fixture::new(|client| {
            let completion_response = language_server::response::Completion {
                results: vec![1,5,9],
                current_version: default(),
            };
            expect_call!(client.completion(
                module      = "Test.Test".to_string(),
                position    = TextLocation::at_document_begin().into(),
                self_type   = None,
                return_type = None,
                tag         = None
            ) => Ok(completion_response));
        });

        let mut subscriber = searcher.subscribe();
        searcher.reload_list();
        assert!(searcher.suggestions().is_loading());
        test.run_until_stalled();
        let expected_list = vec![Suggestion::Completion(entry1),Suggestion::Completion(entry9)];
        assert_eq!(searcher.suggestions().list(), Some(&expected_list));
        let notification = subscriber.next().boxed_local().expect_ready();
        assert_eq!(notification, Some(Notification::NewSuggestionList));
    }

    #[test]
    fn parsed_input() {
        let parser = Parser::new_or_panic();

        fn args_repr(prefix:&ast::prefix::Chain) -> Vec<String> {
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
        assert_eq!(args_repr(&expression) , Vec::<String>::new());
        assert_eq!(parsed.pattern_offset,   2);
        assert_eq!(parsed.pattern.as_str(), "");

        let input  = "foo bar";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        let expression = parsed.expression.unwrap();
        assert_eq!(expression.off         , 0);
        assert_eq!(expression.func.repr() , "foo");
        assert_eq!(args_repr(&expression) , Vec::<String>::new());
        assert_eq!(parsed.pattern_offset,   1);
        assert_eq!(parsed.pattern.as_str(), "bar");

        let input  = "foo  bar  baz";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        let expression = parsed.expression.unwrap();
        assert_eq!(expression.off         , 0);
        assert_eq!(expression.func.repr() , "foo");
        assert_eq!(args_repr(&expression) , vec!["  bar".to_string()]);
        assert_eq!(parsed.pattern_offset,   2);
        assert_eq!(parsed.pattern.as_str(), "baz");

        let input  = "  foo bar baz ";
        let parsed = ParsedInput::new(input.to_string(),&parser).unwrap();
        let expression = parsed.expression.unwrap();
        assert_eq!(expression.off         , 2);
        assert_eq!(expression.func.repr() , "foo");
        assert_eq!(args_repr(&expression) , vec![" bar".to_string()," baz".to_string()]);
        assert_eq!(parsed.pattern_offset,   1);
        assert_eq!(parsed.pattern.as_str(), "");
    }
}
