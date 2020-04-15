pub mod command;
pub mod view;


use crate::prelude::*;

use crate::control::io::keyboard::listener::KeyboardFrpBindings;
use crate::display;
use crate::display::world::World;
use crate::frp;
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
    pub display   : World,
    pub commands  : command::Registry, // <- shortcuts +
    pub shortcuts : shortcut::Registry, // -> needs command_center
    pub views     : view::Registry,
}

impl App {
    /// Constructor.
    pub fn new(dom:&web_sys::HtmlElement) -> Self {
        let logger    = Logger::new("App");
        let display   = World::new(dom);
        let commands  = command::Registry::create(&logger);
        let shortcuts = shortcut::Registry::new(&logger,&commands);
        let views     = view::Registry::create(&logger,&display,&commands,&shortcuts);
        Self {logger,display,commands,shortcuts,views}
    }

//    pub fn new_view<V:View>(&self) -> V {
//        let view = V::new(&self.display);
//        self.commands.register_instance(&view);
//        view
//    }

//    pub fn default_shortcuts() -> {
//
//    }


}

impl display::Object for App {
    fn display_object(&self) -> &display::object::Instance {
        self.display.display_object()
    }
}

//use view::CommandProvider;
pub use view::View;





// ================
// === Shortcut ===
// ================


pub mod shortcut {
    use super::*;
    use enso_prelude::CloneRef;





    #[derive(Clone,CloneRef,Debug)]
    pub struct Registry {
        logger            : Logger,
        keyboard          : Keyboard,
        keyboard_bindings : Rc<KeyboardFrpBindings>,
        network           : frp::Network,
        command_registry  : command::Registry,
        rule_map          : Rc<RefCell<HashMap<keyboard::KeyMask,Vec<Instance>>>>
    }

    impl Registry {
        pub fn new(logger:&Logger, command_registry:&command::Registry) -> Self {
            let logger            = logger.sub("ShortcutManager");
            let keyboard          = Keyboard::default();
            let keyboard_bindings = Rc::new(KeyboardFrpBindings::new(&logger,&keyboard));
            let command_registry  = command_registry.clone_ref();
            let rule_map          = default();
            let network           = default();
            Self {logger,keyboard,keyboard_bindings,network,command_registry,rule_map} . init()
        }

        fn init(self) -> Self {
            let network          = &self.network;
            let logger           = self.logger.clone_ref();
            let rule_map         = self.rule_map.clone_ref();
            let command_registry = self.command_registry.clone_ref();
            frp::extend_network! { network
                def _action = self.keyboard.key_mask.map(move |key_mask| {
                    rule_map.borrow_mut().get_mut(key_mask).map(|rules| {
                        let mut targets = Vec::new();
                        {
                            let borrowed_command_map = command_registry.map.borrow();
                            rules.retain(|weak_rule| {
                                weak_rule.upgrade().map(|rule| {
                                    let target = &rule.target;
                                    borrowed_command_map.get(target).for_each(|commands| {
                                        for command in commands {
                                            if (Self::check(&rule.when,&command.status_map)) {
                                                let command_name = &rule.command.name;
                                                match command.command_map.get(command_name){
                                                    Some(t) => targets.push(t.frp.clone_ref()),
                                                    None => warning!(&logger,"Command {command_name} was not found on {target}."),
                                                }
                                            }
                                        }
                                    })
                                }).is_some()
                            })
                        }
                        for target in targets {
                            target.emit(())
                        }
                    })
                });
            }
            self
        }


        fn check(condition:&Condition, status_map:&HashMap<String,command::Status>) -> bool {
            match condition {
                Condition::Ok => true,
                Condition::Simple(label) => status_map.get(label).map(|t| t.frp.value()).unwrap_or(false)
            }
        }
    }

    impl Add<Shortcut> for &Registry {
        type Output = Handle;
        fn add(self, shortcut:Shortcut) -> Handle {
            let handle   = Handle::new(shortcut.rule);
            let instance = handle.downgrade();
            self.rule_map.borrow_mut().entry(shortcut.key_mask).or_default().push(instance);
            handle
        }
    }


    #[derive(Clone,Debug,Shrinkwrap)]
    pub struct Shortcut {
        #[shrinkwrap(main_field)]
        rule     : Rule,
        key_mask : keyboard::KeyMask,
    }

    impl Shortcut {
        pub fn new_<M,T,C>(key_mask:M, target:T, command:C) -> Self
            where M:Into<keyboard::KeyMask>, T:Into<String>, C:Into<Command> {
            let rule     = Rule::new_(target,command);
            let key_mask = key_mask.into();
            Self {rule,key_mask}
        }

        pub fn new<M,T,C>(key_mask:M, target:T, command:C, condition:Condition) -> Self
        where M:Into<keyboard::KeyMask>, T:Into<String>, C:Into<Command> {
            let rule     = Rule::new(target,command,condition);
            let key_mask = key_mask.into();
            Self {rule,key_mask}
        }
    }

    #[derive(Clone,CloneRef,Debug,Shrinkwrap)]
    pub struct Handle {
        rule : Rc<Rule>
    }

    impl Handle {
        pub fn new(rule:Rule) -> Self {
            let rule = Rc::new(rule);
            Self {rule}
        }

        pub fn downgrade(&self) -> Instance {
            let rule = Rc::downgrade(&self.rule);
            Instance {rule}
        }
    }

    #[derive(Clone,CloneRef,Debug)]
    pub struct Instance {
        rule : Weak<Rule>
    }

    impl Instance {
        pub fn upgrade(&self) -> Option<Handle> {
            self.rule.upgrade().map(|rule| Handle {rule})
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
