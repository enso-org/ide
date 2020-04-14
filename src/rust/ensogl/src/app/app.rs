

use crate::prelude::*;

use crate::control::io::keyboard::listener::KeyboardFrpBindings;
use crate::display::world::World;
use crate::frp as frp;
use crate::frp::io::keyboard::Keyboard;
use crate::frp::io::keyboard;



// ===========
// === App ===
// ===========

/// A top level structure for an application. It combines a view, keyboard shortcut manager, and is
/// intended to also manage layout of visible panes.
#[derive(Debug,Clone,CloneRef)]
pub struct App {
    pub logger    : Logger,
    pub view      : World,
    pub modules   : view::Registry,
    pub shortcuts : shortcut::Registry
}

impl App {
    /// Constructor.
    pub fn new(dom:&web_sys::HtmlElement) -> Self {
        let logger    = Logger::new("App");
        let view      = World::new(dom);
        let modules   = default();
        let shortcuts = shortcut::Registry::new(&logger,&modules);
        Self {logger,view,modules,shortcuts}
    }

    pub fn new_module_instance<M:View>(&self) -> M {
        self.modules.new_module_instance(self)
    }
}



// ==============
// === View ===
// ==============





pub struct EndpointDefinition<T> {
    pub label : String,
    pub frp   : T
}

impl<T> EndpointDefinition<T> {
    pub fn new<L,X>(label:L, frp:X) -> Self
    where L:Into<String>, X:Into<T> {
        let label = label.into();
        let frp   = frp.into();
        Self {label,frp}
    }
}



use view::CommandProvider;
pub use view::View;


pub struct EndpointDocs {
    pub label   : String,
    pub caption : String,
}

impl EndpointDocs {
    pub fn new<L,C>(label:L, caption:C) -> Self
    where L:Into<String>, C:Into<String> {
        let label   = label.into();
        let caption = caption.into();
        Self {label,caption}
    }
}


#[derive(Debug)]
pub struct Endpoint<S> {
    pub caption : String,
    pub frp     : S,
}


pub mod view {
    use super::*;

    pub trait View : NetworkProvider + CommandProvider + StatusProvider {
        const LABEL : &'static str;
        fn new(app:&App) -> Self;
    }

    #[derive(Debug)]
    pub struct Instance {
        pub network     : frp::WeakNetwork,
        pub command_map : HashMap<String,Command>,
        pub status_map  : HashMap<String,Status>,
    }

    impl Instance {
        pub fn check_alive(&self) -> bool {
            self.network.upgrade().is_some()
        }
    }


    #[derive(Debug,Clone,CloneRef,Default)]
    pub struct Registry {
        pub map : Rc<RefCell<HashMap<String,Vec<Instance>>>>,
    }

    impl Registry {
        pub fn new_module_instance<M:View>(&self, app:&App) -> M {
            let module      = M::new(app);
            let label       = M::LABEL.into();
            let network     = M::network(&module).downgrade();
            let command_doc_map : HashMap<String,String> = M::command_api_docs().into_iter().map(|t| {
                (t.label,t.caption)
            }).collect();
            let command_map = M::command_api(&module).into_iter().map(|t| {
                let caption = command_doc_map.get(&t.label).unwrap().clone(); // fixme unwrap
                let frp     = t.frp;
                let endpoint = Endpoint {caption,frp};
                (t.label,endpoint)
            }).collect();

            let status_doc_map : HashMap<String,String> = M::status_api_docs().into_iter().map(|t| {
                (t.label,t.caption)
            }).collect();
            let status_map = M::status_api(&module).into_iter().map(|t| {
                let caption = status_doc_map.get(&t.label).unwrap().clone(); // fixme unwrap
                let frp     = t.frp;
                let endpoint = Endpoint {caption,frp};
                (t.label,endpoint)
            }).collect();

            let module_instance = Instance {network,command_map,status_map};
            self.map.borrow_mut().entry(label).or_default().push(module_instance);
            module
        }
    }



    pub trait NetworkProvider {
        fn network(&self) -> &frp::Network;
    }

    pub trait CommandProvider : Sized {
        fn command_api_docs() -> Vec<EndpointDocs>;
        fn command_api(&self) -> Vec<CommandDefinition>;
    }

    pub type Command           = Endpoint<frp::Source>;
    pub type Status            = Endpoint<frp::Sampler<bool>>;
    pub type CommandDefinition = EndpointDefinition<frp::Source>;
    pub type StatusDefinition  = EndpointDefinition<frp::Sampler<bool>>;


    pub trait StatusProvider : Sized {
        fn status_api_docs() -> Vec<EndpointDocs>;
        fn status_api(&self) -> Vec<StatusDefinition>;
    }


}




pub mod shortcut {
    use super::*;
    use enso_prelude::CloneRef;


    #[derive(Clone,CloneRef,Debug)]
    pub struct Registry {
        logger            : Logger,
        keyboard          : Keyboard,
        keyboard_bindings : Rc<KeyboardFrpBindings>,
        network           : frp::Network,
        views             : view::Registry,
        rule_map          : Rc<RefCell<HashMap<keyboard::KeyMask,Vec<Rule>>>>
    }

    impl Registry {
        pub fn new(logger:&Logger, views:&view::Registry) -> Self {
            let logger            = logger.sub("ShortcutManager");
            let keyboard          = Keyboard::default();
            let keyboard_bindings = Rc::new(KeyboardFrpBindings::new(&logger,&keyboard));
            let views             = views.clone_ref();
            let rule_map          = default();
            let network           = default();
            Self {logger,keyboard,keyboard_bindings,network,views,rule_map} . init()
        }

        pub fn add(&self, keys:&[keyboard::Key], rule:Rule) {
            self.add_by_key_mask(keys.into(),rule)
        }

        pub fn add_by_key_mask(&self, key_mask:keyboard::KeyMask, rule:Rule) {
            self.rule_map.borrow_mut().entry(key_mask).or_default().push(rule);
        }

        fn init(self) -> Self {
            let network  = &self.network;
            let logger   = self.logger.clone_ref();
            let rule_map = self.rule_map.clone_ref();
            let views    = self.views.clone_ref();
            frp::extend_network! { network
                def _action = self.keyboard.key_mask.map(move |key_mask| {
                    let opt_rules = rule_map.borrow().get(key_mask).cloned();
                    opt_rules.for_each(|rules| {
                        let mut targets = Vec::new();
                        {
                            let borrowed_modules_map = views.map.borrow();
                            for rule in &rules {
                                let target = &rule.target;
                                borrowed_modules_map.get(target).for_each(|mods| {
                                    for module in mods {
                                        if (Self::check(&rule.when,&module.status_map)) {
                                            let command_name = &rule.command.name;
                                            match module.command_map.get(command_name){
                                                Some(t) => targets.push(t.frp.clone_ref()),
                                                None => warning!(&logger,"Command {command_name} was not found on {target}."),
                                            }
                                        }
                                    }
                                })
                            }
                        }
                        for target in targets {
                            target.emit(())
                        }
                    });
                });
            }
            self
        }


        fn check(condition:&Condition, status_map:&HashMap<String,view::Status>) -> bool {
            match condition {
                Condition::Ok => true,
                Condition::Simple(label) => status_map.get(label).map(|t| t.frp.value()).unwrap_or(false)
            }
        }
    }



    #[derive(Clone,Debug)]
    pub struct Rule {
        target  : String,
        when    : Condition,
        command : Command
    }

    impl Rule {
        pub fn new_<T,C>(target:T, command:C) -> Self
        where T:Into<String>, C:Into<Command> {
            Self::new(target,command,Condition::Ok)
        }

        pub fn new<T,C>(target:T, command:C, when:Condition) -> Self
        where T:Into<String>, C:Into<Command> {
            let target  = target.into();
            let command = command.into();
            Self {target,when,command}
        }
    }

    #[derive(Clone,Debug)]
    pub enum Condition {
        Ok,
        Simple (String),
//        Or     (Box<Condition>, Box<Condition>),
//        And    (Box<Condition>, Box<Condition>),
    }



    #[derive(Clone,Debug,Eq,From,Hash,Into,PartialEq,Shrinkwrap)]
    pub struct Command {
        name : String,
    }

    impl From<&str> for Command {
        fn from(s:&str) -> Self {
            Self {name:s.into()}
        }
    }



}
