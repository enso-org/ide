//! Keyboard and mouse shortcut management.

use crate::prelude::*;

use super::command;

// use crate::frp::io::keyboard::Keyboard;
// use crate::frp::io::keyboard;
use crate::frp::io::keyboard2;
use crate::frp::io::mouse::Mouse;
use crate::frp::io::mouse;
use crate::frp;
use crate::system::web;
use enso_shortcuts as shortcuts;
use enso_shortcuts::Registry as TRAIT_Registry; // fixme

pub use shortcuts::ActionType;
pub use ActionType::*;



// ==============
// === Rule ===
// ==============

/// Keyboard action defined as `ActionType` and `ActionPattern`, like "press both key 'n' and primary
/// mouse button". Please note that the release action happens as soon as the key mask is no longer
/// valid. For example, pressing key "n", and then pressing key "a" (while holding "n") will trigger
/// the release event for the key "n".
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct Rule {
    pub tp      : shortcuts::ActionType,
    pub pattern : String,
}

impl Rule {
    /// Constructor.
    pub fn new(tp:impl Into<shortcuts::ActionType>, pattern:impl Into<String>) -> Self {
        let tp      = tp.into();
        let pattern = pattern.into();
        Self {tp,pattern}
    }
}



// ===============
// === Command ===
// ===============

/// A command name.
#[derive(Clone,Debug,Eq,From,Hash,Into,PartialEq,Shrinkwrap)]
pub struct Command {
    name : String,
}

impl From<&str> for Command {
    fn from(s:&str) -> Self {
        Self {name:s.into()}
    }
}



// =================
// === Condition ===
// =================

// TODO[WD]: Uncomment and handle more complex cases. Left commented to show the direction of future
//           development.
/// Condition expression.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum Condition {
    Ok,
    Simple (String),
    // Or  (Box<Condition>, Box<Condition>),
    // And (Box<Condition>, Box<Condition>),
}



// ============
// === Action ===
// ============

/// A shortcut rule. Consist of target identifier (like "TextEditor"), a `Command` that will be
/// evaluated on the target, and a `Condition` which needs to be true in order for the command
/// to be executed.
#[derive(Clone,Debug)]
pub struct Action {
    target  : String,
    command : Command,
    when    : Condition,
}

impl Action {
    /// Constructor. Version without condition checker.
    pub fn new<T,C>(target:T, command:C) -> Self
        where T:Into<String>, C:Into<Command> {
        Self::new_when(target,command,Condition::Ok)
    }

    /// Constructor.
    pub fn new_when<T,C>(target:T, command:C, when:Condition) -> Self
        where T:Into<String>, C:Into<Command> {
        let target  = target.into();
        let command = command.into();
        Self {target,when,command}
    }
}



// ================
// === Shortcut ===
// ================

/// A keyboard shortcut, an `Rule` associated with a `Action`.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct Shortcut {
    #[shrinkwrap(main_field)]
    rule   : Action,
    action : Rule,
}

impl Shortcut {
    /// Constructor. Version without condition checker.
    pub fn new<A,T,C>(action:A, target:T, command:C) -> Self
    where A:Into<Rule>, T:Into<String>, C:Into<Command> {
        let rule   = Action::new(target,command);
        let action = action.into();
        Self {rule,action}
    }

    /// Constructor.
    pub fn new_when<A,T,C>(action:A, target:T, command:C, condition:Condition) -> Self
        where A:Into<Rule>, T:Into<String>, C:Into<Command> {
        let rule     = Action::new_when(target,command,condition);
        let action = action.into();
        Self {rule,action}
    }
}



// ================
// === Registry ===
// ================

/// Shortcut registry. See `Shortcut` to learn more.
///
/// You can add new shortcuts by using the `add` method and get a `Handle` back. When `Handle` is
/// dropped, the shortcut will be lazily removed. This is useful when defining shortcuts by GUI
/// components. When a component is unloaded, all its default shortcuts should be removed as well.
///
/// ## Implementation Notes
/// There should be a layer for user shortcuts which will remember handles permanently until a
/// shortcut is unregistered.
#[derive(Clone,CloneRef,Debug)]
pub struct Registry {
    model   : RegistryModel,
    network : frp::Network,
}

/// Internal representation of `Registry`.
#[derive(Clone,CloneRef,Debug)]
pub struct RegistryModel {
    logger             : Logger,
    keyboard2          : keyboard2::Keyboard,
    mouse              : Mouse,
    command_registry   : command::Registry,
    shortcuts_registry : shortcuts::HashSetRegistry<Shortcut>,
}

impl Deref for Registry {
    type Target = RegistryModel;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl Registry {
    /// Constructor.
    pub fn new(logger:&Logger, mouse:&Mouse, keyboard2:&keyboard2::Keyboard, command_registry:&command::Registry) -> Self {
        let model = RegistryModel::new(logger,mouse,keyboard2,command_registry);
        let mouse = &model.mouse;

        frp::new_network! { network
            on_press   <- keyboard2.down.map (f!((t) model.shortcuts_registry.on_press(t.simple_name())));
            on_release <- keyboard2.up.map   (f!((t) model.shortcuts_registry.on_release(t.simple_name())));
            trace on_press;
            trace on_release;

            eval on_press ([model](m) {
                model.process_rules(&mut m.clone());
            });
            eval on_release ([model](m) {
                model.process_rules(&mut m.clone());
            });
        }
        Self {model,network}
    }
}

impl RegistryModel {
    /// Constructor.
    pub fn new
    (logger:impl AnyLogger, mouse:&Mouse, keyboard2:&keyboard2::Keyboard, command_registry:&command::Registry) -> Self {
        let logger             = Logger::sub(logger,"ShortcutRegistry");
        let keyboard2          = keyboard2.clone_ref();
        let mouse              = mouse.clone_ref();
        let command_registry   = command_registry.clone_ref();
        let shortcuts_registry = default();
        Self {logger,keyboard2,mouse,command_registry,shortcuts_registry}
    }

    fn process_rules(&self, rules:&mut Vec<Shortcut>) {
        let mut targets = Vec::new();
        {
            let borrowed_command_map = self.command_registry.instances.borrow();
            for rule in rules {
                let target = &rule.rule.target;
                borrowed_command_map.get(target).for_each(|commands| {
                    for command in commands {
                        if Self::condition_checker(&rule.when,&command.status_map) {
                            let command_name = &rule.command.name;
                            match command.command_map.get(command_name){
                                Some(t) => targets.push(t.frp.clone_ref()),
                                None    => warning!(&self.logger,
                                        "Command {command_name} was not found on {target}."),
                            }
                        }
                    }
                })
            }
        }
        for target in targets {
            target.emit(())
        }
    }

    fn condition_checker
    (condition:&Condition, status_map:&HashMap<String,command::Status>) -> bool {
        match condition {
            Condition::Ok           => true,
            Condition::Simple(name) => status_map.get(name).map(|t| t.frp.value()).unwrap_or(false)
        }
    }
}

impl Add<Shortcut> for &Registry {
    type Output = ();
    fn add(self, shortcut:Shortcut) {
        self.model.shortcuts_registry.add(shortcut.action.tp,&shortcut.action.pattern,shortcut.clone()); // fixme clone
    }
}



// ===============================
// === DefaultShortcutProvider ===
// ===============================

/// Trait allowing providing default set of shortcuts exposed by an object.
pub trait DefaultShortcutProvider : command::Provider {
    /// Set of default shortcuts.
    fn default_shortcuts() -> Vec<Shortcut> {
        default()
    }

    /// Helper for defining shortcut targeting this object.
    fn self_shortcut_when
    (action:impl Into<Rule>, command:impl Into<Command>, condition:Condition) -> Shortcut {
        Shortcut::new_when(action,Self::label(),command,condition)
    }

    fn self_shortcut
    (action_type:shortcuts::ActionType, pattern:impl Into<String>, command:impl Into<Command>) -> Shortcut {
        Shortcut::new(Rule::new(action_type,pattern),Self::label(),command)
    }
}
