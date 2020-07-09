//! This module contains all structures related to Searcher Controller.

use crate::prelude::*;

use crate::notification;

use data::text::TextLocation;
use enso_protocol::language_server;
use flo_stream::Subscriber;


#[derive(Clone,CloneRef,Debug,Eq,PartialEq)]
pub enum Suggestion {
    Completion(Rc<model::suggestion_database::Entry>)
}


#[derive(Clone,CloneRef,Debug)]
pub enum Suggestions {
    Loading,
    Loaded {
        list : Rc<Vec<Suggestion>>
    },
    Error(Rc<failure::Error>)
}

impl Suggestions {
    pub fn is_loading(&self) -> bool {
        match self {
            Self::Loading => true,
            _             => false,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Self::Error(_) => true,
            _              => false,
        }
    }

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

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Notification {
    NewList
}

#[derive(Clone,Debug,Default)]
struct Data {
    current_input : String,
    current_list  : Suggestions,
}

#[derive(Clone,CloneRef,Debug)]
pub struct Searcher {
    logger          : Logger,
    data            : Rc<RefCell<Data>>,
    notifier        : notification::Publisher<Notification>,
    module          : Rc<model::module::QualifiedName>,
    position        : Immutable<TextLocation>,
    database        : Rc<model::SuggestionDatabase>,
    language_server : Rc<language_server::Connection>,
}

impl Searcher {
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
        };
        this.reload_list();
        this
    }

    pub fn subscribe(&self) -> Subscriber<Notification> {
        self.notifier.subscribe()
    }

    pub fn suggestions(&self) -> Suggestions {
        self.data.borrow().current_list.clone_ref()
    }

    fn reload_list(&self) {
        let module      = self.module.deref().into();
        let self_type   = None;
        let return_type = None;
        let tags        = None;
        let position    = self.position.deref().into();
        let request     = self.language_server.completion(module,&position,&self_type,&return_type,&tags);
        let data        = self.data.clone_ref();
        let database    = self.database.clone_ref();
        let logger      = self.logger.clone_ref();
        let notifier    = self.notifier.clone_ref();

        self.data.borrow_mut().current_list = Suggestions::Loading;
        executor::global::spawn(async move {
            info!(logger,"Requesting new suggestion list");
            let ls_response = request.await;
            info!(logger,"Received suggestions from Language Server");
            let new_list = match ls_response {
                Ok(list) => {
                    let entry_ids   = list.results.into_iter();
                    let entries     = entry_ids.filter_map(|id| {
                        let entry = database.get(id);
                        if entry.is_none() {
                            error!(logger,"Missing entry {id} in Suggestion Database");
                        }
                        entry
                    });
                    let suggestions = entries.map(Suggestion::Completion);
                    Suggestions::Loaded {list:Rc::new(suggestions.collect())}
                },
                Err(error) => Suggestions::Error(Rc::new(error.into()))
            };
            data.borrow_mut().current_list = new_list;
            notifier.publish(Notification::NewList).await;
        });
    }
}


// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::model::module::QualifiedName;
    use crate::model::module::Path;

    use json_rpc::expect_call;
    use utils::test::traits::*;

    #[test]
    fn reloading_list() {
        let mut test = TestWithLocalPoolExecutor::set_up();
        let client   = language_server::MockClient::default();

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

        let searcher = Searcher {
            logger          : default(),
            data            : default(),
            notifier        : default(),
            module          : Rc::new(QualifiedName::from_path(&Path::from_mock_module_name("Test"),"Test")),
            position        : Immutable(TextLocation::at_document_begin()),
            database        : default(),
            language_server : language_server::Connection::new_mock_rc(client),
        };
        let entry1 = model::suggestion_database::Entry {
            name          : "TestFunction1".to_string(),
            kind          : model::suggestion_database::Kind::Function,
            module        : "Test.Test".to_string(),
            arguments     : vec![],
            return_type   : "Number".to_string(),
            documentation : default(),
            self_type     : None
        };
        let entry2 = model::suggestion_database::Entry {
            name : "TestFunction2".to_string(),
            ..entry1.clone()
        };
        searcher.database.put_entry(1,entry1);
        let entry1 = searcher.database.get(1).unwrap();
        searcher.database.put_entry(9,entry2);
        let entry2 = searcher.database.get(9).unwrap();

        let mut subscriber = searcher.subscribe();

        searcher.reload_list();
        assert!(searcher.suggestions().is_loading());
        test.run_until_stalled();
        let expected_list = vec![Suggestion::Completion(entry1),Suggestion::Completion(entry2)];
        assert_eq!(searcher.suggestions().list(), Some(&expected_list));
        assert_eq!(subscriber.next().boxed_local().expect_ready(), Some(Notification::NewList));
    }
}