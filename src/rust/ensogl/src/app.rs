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
    pub views     : view::Registry,
    pub shortcuts : shortcut::Registry
}

impl App {
    /// Constructor.
    pub fn new(dom:&web_sys::HtmlElement) -> Self {
        let logger    = Logger::new("App");
        let display   = World::new(dom);
        let views     = view::Registry::create(&logger,&display);
        let shortcuts = shortcut::Registry::new(&logger,&views);
        Self {logger,display,views,shortcuts}
    }

//    pub fn default_shortcuts() -> {
//
//    }


}

impl display::Object for App {
    fn display_object(&self) -> &display::object::Instance {
        self.display.display_object()
    }
}



// ============
// === View ===
// ============

use view::CommandProvider;
pub use view::View;








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
