

use crate::prelude::*;

use enso_frp as frp;
use ensogl::display::world::World;
use ensogl::control::io::keyboard::listener::KeyboardFrpBindings;
use enso_frp::io::keyboard;
use enso_frp::io::keyboard::Keyboard;



#[derive(Debug,Clone,CloneRef)]
pub struct App {
    pub logger    : Logger,
    pub world     : World,
    pub modules   : module::Registry,
    pub shortcuts : shortcut::Registry
}

impl App {
    pub fn new(dom:&web_sys::HtmlElement) -> Self {
        let logger    = Logger::new("App");
        let world     = World::new(dom);
        let modules   = default();
        let shortcuts = shortcut::Registry::new(&logger,&modules);
        Self {logger,world,modules,shortcuts}
    }

    pub fn new_module_instance<M:Module>(&self) -> M {
        self.modules.new_module_instance(self)
    }


}


#[derive(Debug)]
pub struct ModuleInstance {
    pub network     : frp::WeakNetwork,
    pub command_map : HashMap<String,module::Command>,
    pub status_map  : HashMap<String,module::Status>,
}

impl ModuleInstance {
    pub fn check_alive(&self) -> bool {
        self.network.upgrade().is_some()
    }
}




use module::CommandProvider;
pub use module::Module;

pub struct EndpointDefinition<S,T> {
    pub label   : String,
    pub caption : String,
    pub getter  : Box<dyn for<'t> Fn(&'t T) -> &'t S>
}

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

impl<S:CloneRef,T> EndpointDefinition<S,T> {
    pub fn new<L,C,F>(label:L, caption:C, f:F) -> Self
        where L:Str, C:Str, F:'static + for<'t> Fn(&'t T) -> &'t S {
        let label   = label.into();
        let caption = caption.into();
        let getter  = Box::new(f);
        Self {label,caption,getter}
    }

    pub fn instance(&self, t:&T) -> Endpoint<S> {
        let caption = self.caption.clone();
        let frp     = (self.getter)(t).clone_ref();
        Endpoint {caption,frp}
    }
}

#[derive(Debug)]
pub struct Endpoint<S> {
    pub caption : String,
    pub frp     : S,
}


pub mod module {
    use super::*;


    #[derive(Debug,Clone,CloneRef,Default)]
    pub struct Registry {
        pub map : Rc<RefCell<HashMap<String,Vec<ModuleInstance>>>>,
    }

    impl Registry {
        pub fn new_module_instance<M:Module>(&self, app:&App) -> M {
            let module      = M::new(app);
            let label       = M::LABEL.into();
            let network     = M::network(&module).downgrade();
            let command_doc_map : HashMap<String,String> = M::commands_docs().into_iter().map(|t| {
                (t.label,t.caption)
            }).collect();
            let command_map = M::commands(&module).into_iter().map(|t| {
                let caption = command_doc_map.get(&t.0).unwrap().clone(); // fixme unwrap
                let frp     = t.1;
                let endpoint = Endpoint {caption,frp};
                (t.0,endpoint)
            }).collect();


            let status_map = M::status_api().into_iter().map(|def| {
                let instance = def.instance(&module);
                let label    = def.label;
                (label,instance)
            }).collect();
            let module_instance = ModuleInstance {network,command_map,status_map};
            self.map.borrow_mut().entry(label).or_default().push(module_instance);
            module
        }
    }

    pub trait Module : NetworkProvider + CommandProvider + StatusProvider {
        const LABEL : &'static str;
        fn new(app:&App) -> Self;
    }

    pub trait NetworkProvider {
        fn network(&self) -> &frp::Network;
    }

    pub trait CommandProvider : Sized {
        fn commands_docs() -> Vec<EndpointDocs>;
        fn commands(&self) -> Vec<(String,frp::Source)>;
//        fn command_api() -> Vec<CommandDefinition<Self>>;
    }

    pub type Command              = Endpoint<frp::Source>;
    pub type CommandDefinition<T> = EndpointDefinition<frp::Source,T>;


    pub trait StatusProvider : Sized {
        fn status_api() -> Vec<StatusDefinition<Self>>;
    }

    pub type Status              = Endpoint<frp::Sampler<bool>>;
    pub type StatusDefinition<T> = EndpointDefinition<frp::Sampler<bool>,T>;


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
        modules           : module::Registry,
        rule_map          : Rc<RefCell<HashMap<keyboard::KeyMask,Vec<Rule>>>>
    }

    impl Registry {
        pub fn new(logger:&Logger, modules:&module::Registry) -> Self {
            let logger            = logger.sub("ShortcutManager");
            let keyboard          = Keyboard::default();
            let keyboard_bindings = Rc::new(KeyboardFrpBindings::new(&logger,&keyboard));
            let modules           = modules.clone_ref();
            let rule_map          = default();
            let network           = default();
            Self {logger,keyboard,keyboard_bindings,network,modules,rule_map} . init()
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
            let modules  = self.modules.clone_ref();
            frp::extend_network! { network
                def _action = self.keyboard.key_mask.map(move |key_mask| {
                    let opt_rules = rule_map.borrow().get(key_mask).cloned();
                    opt_rules.for_each(|rules| {
                        let mut targets = Vec::new();
                        {
                            let borrowed_modules_map = modules.map.borrow();
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


        fn check(condition:&Condition, status_map:&HashMap<String,module::Status>) -> bool {
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