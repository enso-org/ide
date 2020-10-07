//! Definition of commands, labeled FPR endpoints useful when implementing actions which can be
//! altered at runtime, like a keyboard shortcut management.

use crate::prelude::*;
use crate::frp;

use super::shortcut;
use super::shortcut::Shortcut;
use super::Application;



// ================
// === Provider ===
// ================

pub trait XXX = Deref where <Self as Deref>::Target : CommandApi;

/// A visual component of an application.
pub trait View : FrpNetworkProvider + XXX {
    /// Identifier of the command provider class.
    fn label() -> &'static str;
    /// Constructor.
    fn new(app:&Application) -> Self;

    /// Set of default shortcuts.
    fn default_shortcuts() -> Vec<Shortcut> {
        default()
    }

    // /// Helper for defining shortcut targeting this object.
    // fn self_shortcut_when
    // (action:impl Into<shortcut::Rule>, command:impl Into<shortcut::Command>, condition:shortcut::Condition) -> Shortcut {
    //     Shortcut::new_when(action,Self::label(),command,condition)
    // }

    /// Add a new shortcut targeting the self object.
    fn self_shortcut
    (action_type:shortcut::ActionType, pattern:impl Into<String>, command:impl Into<shortcut::Command>) -> Shortcut {
        Shortcut::new(shortcut::Rule::new(action_type,pattern),Self::label(),command)
    }

    /// Add a new shortcut targeting the self object.
    fn self_shortcut_when
    (action_type:shortcut::ActionType, pattern:impl Into<String>, command:impl Into<shortcut::Command>, condition:impl Into<shortcut::Condition>) -> Shortcut {
        Shortcut::new_when(shortcut::Rule::new(action_type,pattern),Self::label(),command,condition)
    }
}

/// FRP Network provider. Used to check whether FRP bindings are still alive.
pub trait FrpNetworkProvider {
    /// The underlying frp network accessor.
    fn network(&self) -> &frp::Network;
}



// ======================
// === API Definition ===
// ======================

#[allow(missing_docs)]
pub trait CommandApi : Sized {
    fn command_api(&self) -> Rc<RefCell<HashMap<String,frp::Source<()>>>> { default() }
    fn status_api(&self) -> Rc<RefCell<HashMap<String,frp::Sampler<bool>>>> { default() }
}



// ========================
// === ProviderInstance ===
// ========================

/// Instance of command `Provider`. It contains bindings to all FRP endpoints defined by the
/// `Provider`. See the docs of `Provider` to learn more.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct ProviderInstance {
    pub network     : frp::WeakNetwork,
    pub command_map : Rc<RefCell<HashMap<String,frp::Source<()>>>>,
    pub status_map  : Rc<RefCell<HashMap<String,frp::Sampler<bool>>>>,
}

impl ProviderInstance {
    /// Check whether the underlying object is still alive.
    pub fn check_alive(&self) -> bool {
        self.network.upgrade().is_some()
    }
}



// ================
// === Registry ===
// ================

/// A command registry. Allows registering command `Providers` and corresponding
/// `ProviderInstance`s. See docs of `Provider` to learn more.
#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct Registry {
    pub logger    : Logger,
    pub instances : Rc<RefCell<HashMap<String,Vec<ProviderInstance>>>>,
}

impl Registry {
    /// Constructor.
    pub fn create(logger:impl AnyLogger) -> Self {
        let logger    = Logger::sub(logger,"views");
        let instances = default();
        Self {logger,instances}
    }

    /// Registers the command `Provider`.
    pub fn register<V:View>(&self) {
        let label  = V::label();
        let exists = self.instances.borrow().get(label).is_some();
        if exists {
            warning!(&self.logger, "The view '{label}' was already registered.")
        } else {
            self.instances.borrow_mut().insert(label.into(),default());
        }
    }

    // fn command_api(&self) -> Rc<RefCell<HashMap<String,frp::Source<()>>>> { default() }

    /// Registers the command `ProviderInstance`.
    pub fn register_instance<T:View>(&self, target:&T) {
        let label   = T::label();
        let network = T::network(target).downgrade();
        // let command_doc_map : HashMap<String,String> = T::command_api_docs().into_iter().map(|t| {
        //     (t.label,t.caption)
        // }).collect();
        // let command_map = T::command_api(target).into_iter().map(|t| {
        //     let caption = command_doc_map.get(&t.label).unwrap().clone(); // fixme unwrap
        //     let frp     = t.frp;
        //     let endpoint = FrpEndpoint {caption,frp};
        //     (t.label,endpoint)
        // }).collect();
        //
        // let status_doc_map : HashMap<String,String> = T::status_api_docs().into_iter().map(|t| {
        //     (t.label,t.caption)
        // }).collect();
        // let status_map = T::status_api(target).into_iter().map(|t| {
        //     let caption = status_doc_map.get(&t.label).unwrap().clone(); // fixme unwrap
        //     let frp     = t.frp;
        //     let endpoint = FrpEndpoint {caption,frp};
        //     (t.label,endpoint)
        // }).collect();

        // let instance = ProviderInstance {network,command_map,status_map};
        let command_map = target.deref().command_api();
        let status_map = target.deref().status_api();
        let instance = ProviderInstance {network,command_map,status_map};
        let was_registered = self.instances.borrow().get(label).is_some();
        if !was_registered {
            self.register::<T>();
            warning!(&self.logger,
                "The command provider '{label}' was created but never registered. You should \
                always register available command providers as soon as possible to spread the \
                information about their API.");
        };
        self.instances.borrow_mut().get_mut(label).unwrap().push(instance);
    }
}
