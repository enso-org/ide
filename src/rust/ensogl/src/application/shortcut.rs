//! Keyboard shortcut management.

use crate::prelude::*;

use super::command;

use crate::control::io::keyboard::listener::KeyboardFrpBindings;
use crate::frp::io::keyboard::KeyMask;
use crate::frp::io::keyboard::Keyboard;
use crate::frp;
#[cfg(target_arch = "wasm32")]
use crate::system::web;

#[cfg(not(target_arch = "wasm32"))]
use std::time;



// ================
// === Registry ===
// ================

type RuleMap   = HashMap<KeyMask,Vec<WeakHandle>>;
type ActionMap = HashMap<ActionType,RuleMap>;

/// Keyboard shortcut registry. You can add new shortcuts by using the `add` method and get a
/// `Handle` back. When `Handle` is dropped, the shortcut will be lazily removed. This is useful
/// when defining shortcuts by GUI components. When a component is unloaded, all its default
/// shortcuts should be removed as well.
///
/// Note: we should probably handle user shortcuts in a slightly different way. User shortcuts
/// should persist and should probably not return handles. Alternatively, there should be an user
/// shortcut manager which will own and manage the handles.
#[derive(Clone,CloneRef,Debug)]
pub struct Registry {
    model   : RegistryModel,
    network : frp::Network,
}

/// Internal representation of `Registry`.
#[derive(Clone,CloneRef,Debug)]
pub struct RegistryModel {
    logger                : Logger,
    keyboard              : Keyboard,
    keyboard_bindings     : Rc<KeyboardFrpBindings>,
    command_registry      : command::Registry,
    action_map            : Rc<RefCell<ActionMap>>,
    double_press_detector : Rc<RefCell<DoublePressDetector>>
}

impl Deref for Registry {
    type Target = RegistryModel;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl RegistryModel {
    /// Constructor.
    pub fn new(logger:&Logger, command_registry:&command::Registry) -> Self {
        let logger                = logger.sub("ShortcutRegistry");
        let keyboard              = Keyboard::default();
        let keyboard_bindings     = Rc::new(KeyboardFrpBindings::new(&logger,&keyboard));
        let command_registry      = command_registry.clone_ref();
        let action_map            = default();
        let double_press_detector = DoublePressDetector::new(750.0);
        let double_press_detector = Rc::new(RefCell::new(double_press_detector));

        Self {logger,keyboard,keyboard_bindings,command_registry,action_map,double_press_detector}
    }
}

impl Registry {
    /// Constructor.
    pub fn new(logger:&Logger, command_registry:&command::Registry) -> Self {
        let model = RegistryModel::new(logger,command_registry);

        frp::new_network! { network
            eval model.keyboard.key_mask          ((m) model.process_action(ActionType::Press,m));
            eval model.keyboard.previous_key_mask ((m) model.process_action(ActionType::Release,m));
        }
        Self {model,network}
    }
}

impl RegistryModel {

    /// Check the action against internal state and potentially change it. Right now this is used
    /// to detect DoublePress actions.
    fn pre_process_action(&self, action_type:ActionType, key_mask:&KeyMask) -> ActionType {
        let mut detector     =  self.double_press_detector.borrow_mut();
        let is_double_press  = detector.process_action(action_type,key_mask);
        if is_double_press {
            ActionType::DoublePress
        } else {
            action_type
        }
    }

    fn process_action(&self, action_type:ActionType, key_mask:&KeyMask) {
       // TODO: Decide whether double press is in addition or instead of the normal press.
        let action_type    = self.pre_process_action(action_type, key_mask);
        let action_map_mut = &mut self.action_map.borrow_mut();
        if let Some(rule_map) = action_map_mut.get_mut(&action_type) {
            if let Some(rules) = rule_map.get_mut(key_mask) {
                self.process_rules(rules)
            }
        }
    }

    fn process_rules(&self, rules:&mut Vec<WeakHandle>) {
        let mut targets = Vec::new();
        {
            let borrowed_command_map = self.command_registry.instances.borrow();
            rules.retain(|weak_rule| {
                weak_rule.upgrade().map(|rule| {
                    let target = &rule.target;
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
                }).is_some()
            })
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
    type Output = Handle;
    fn add(self, shortcut:Shortcut) -> Handle {
        let handle     = Handle::new(shortcut.rule);
        let instance   = handle.downgrade();
        let action_map = &mut self.action_map.borrow_mut();
        let rule_map   = action_map.entry(shortcut.action.tp).or_default();
        let rules      = rule_map.entry(shortcut.action.key_mask).or_default();
        rules.push(instance);
        handle
    }
}



// ==============
// === Handle ===
// ==============

/// A handle to registered shortcut. When dropped, the shortcut will be lazily removed from the
/// `Registry`.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Handle {
    rule : Rc<Rule>
}

impl Handle {
    /// Constructor.
    pub fn new(rule:Rule) -> Self {
        let rule = Rc::new(rule);
        Self {rule}
    }

    fn downgrade(&self) -> WeakHandle {
        let rule = Rc::downgrade(&self.rule);
        WeakHandle {rule}
    }
}

/// Weak version of the `Handle`.
#[derive(Clone,CloneRef,Debug)]
pub struct WeakHandle {
    rule : Weak<Rule>
}

impl WeakHandle {
    fn upgrade(&self) -> Option<Handle> {
        self.rule.upgrade().map(|rule| Handle {rule})
    }
}



// ==============
// === Action ===
// ==============

/// A type of a keyboard action.
#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
#[allow(missing_docs)]
pub enum ActionType {
    Press, Release, DoublePress
}

/// Keyboard action defined as `ActionType` and `KeyMask`, like "release key 'n'". Please note that
/// the release action happens as soon as the key mask is no longer valid. So for example, after
/// pressing key "n", and then pressing key "a" (while holding "n"), the release event of the key
/// "n" will be emitted.
#[derive(Clone,Copy,Debug)]
#[allow(missing_docs)]
pub struct Action {
    pub tp       : ActionType,
    pub key_mask : KeyMask,
}

impl Action {
    /// Constructor.
    pub fn new(tp:impl Into<ActionType>, key_mask:impl Into<KeyMask>) -> Self {
        let tp       = tp.into();
        let key_mask = key_mask.into();
        Self {tp,key_mask}
    }

    /// Smart constructor for the `Press` action.
    pub fn press(key_mask:impl Into<KeyMask>) -> Self {
        Self::new(ActionType::Press,key_mask)
    }

    /// Smart constructor for the `Release` action.
    pub fn release(key_mask:impl Into<KeyMask>) -> Self {
        Self::new(ActionType::Release,key_mask)
    }

    /// Smart constructor for the `DoublePress` action.
    pub fn double_press(key_mask:impl Into<KeyMask>) -> Self {
        Self::new(ActionType::DoublePress,key_mask)
    }
}



// ================
// === Shortcut ===
// ================

/// A keyboard shortcut, an `Action` associated with a `Rule`.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct Shortcut {
    #[shrinkwrap(main_field)]
    rule   : Rule,
    action : Action,
}

impl Shortcut {
    /// Constructor. Version without condition checker.
    pub fn new<A,T,C>(action:A, target:T, command:C) -> Self
    where A:Into<Action>, T:Into<String>, C:Into<Command> {
        let rule     = Rule::new(target,command);
        let action = action.into();
        Self {rule,action}
    }

    /// Constructor.
    pub fn new_when<A,T,C>(action:A, target:T, command:C, condition:Condition) -> Self
    where A:Into<Action>, T:Into<String>, C:Into<Command> {
        let rule     = Rule::new_when(target,command,condition);
        let action = action.into();
        Self {rule,action}
    }
}



// ============
// === Rule ===
// ============

/// A shortcut rule. Consist of target identifier (`command::ProviderTag`), a `Command` that will
/// be evaluated on the target, and a `Condition` which needs to be true in order for the command
/// to be executed.
#[derive(Clone,Debug)]
pub struct Rule {
    target  : String,
    command : Command,
    when    : Condition,
}

impl Rule {
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

// TODO: Uncomment and handle more complex cases. Left here to show the intention of future dev.
/// Condition expression.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum Condition {
    Ok,
    Simple (String),
    // Or     (Box<Condition>, Box<Condition>),
    // And    (Box<Condition>, Box<Condition>),
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
    fn self_shortcut_when<A,C>(action:A, command:C, condition:Condition) -> Shortcut
    where A:Into<Action>, C:Into<Command> {
        Shortcut::new_when(action,Self::label(),command,condition)
    }

    /// Helper for defining shortcut targeting this object. Version which does not accept
    /// condition checker.
    fn self_shortcut<A,C>(action:A, command:C) -> Shortcut
    where A:Into<Action>, C:Into<Command> {
        Shortcut::new(action,Self::label(),command)
    }
}

/// Default implementation for all objects. It allows implementing this trait only by objects which
/// want to use it.
impl<T:command::Provider> DefaultShortcutProvider for T {
    default fn default_shortcuts() -> Vec<Shortcut> {
        default()
    }
}



// ===========================
// === DoublePressDetector ===
// ===========================

/// Detects whether a key was pressed in quick succession.
// TODO Refactor for better platform independent time keeping.
// TODO Consider using application/world time.
#[derive(Clone,Debug)]
struct DoublePressDetector {
    prev_key     : Option<KeyMask>,
    prev_time    : Option<f64>,
    threshold_ms : f64,
}

impl DoublePressDetector {
    
    fn new(threshold_ms:f64) -> Self {
        DoublePressDetector {
            prev_key     : None,
            prev_time    : None,
            threshold_ms
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn now(&self) -> f64 {
        let performance = web::performance();
        performance.now()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn now(&self) -> f64 {
        time::Instant::now().elapsed().as_millis() as f64
    }

    /// Check whether this key hs been pressed within the threshold duration.
    pub fn process_action(&mut self, action_type:ActionType, key_mask:&KeyMask) -> bool {
        if action_type != ActionType::Press {
            return false
        }
        // TODO Investigate why we are getting zero keymask press events after every real event.
        if *key_mask == KeyMask::from_vec(vec![]) {
            return false;
        }

        let is_same_key = if let Some(prev_key_mask) = self.prev_key {
            prev_key_mask == *key_mask
        } else {
            false
        };
        let now = self.now();
        let within_threshold = if let Some(prev_time) = self.prev_time {
            (now - prev_time) < self.threshold_ms
        } else {
            false
        };
        self.prev_time = Some(now);
        self.prev_key  = Some(*key_mask);
        is_same_key && within_threshold
    }
}
