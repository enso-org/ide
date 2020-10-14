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

pub trait DerefToCommandApi = Deref where <Self as Deref>::Target : CommandApi;

/// A visual component of an application.
pub trait View : FrpNetworkProvider + DerefToCommandApi {
    /// Identifier of the command provider class.
    fn label() -> &'static str;

    /// Constructor.
    fn new(app:&Application) -> Self;

    /// Application reference.
    fn app(&self) -> &Application;

    /// Set of default shortcuts.
    fn default_shortcuts() -> Vec<Shortcut> {
        default()
    }

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

    /// Disable the command in this component instance.
    fn disable_command(&self, name:impl AsRef<str>) where Self:Sized {
        self.app().commands.disable_command(self,name)
    }

    /// Enable the command in this component instance.
    fn enable_command(&self, name:impl AsRef<str>) where Self:Sized {
        self.app().commands.enable_command(self,name)
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

#[derive(Debug)]
pub struct Command {
    pub frp     : frp::Source,
    pub enabled : bool,
}

// impl Deref for Command {
//     type Target = frp::Source;
//     fn deref(&self) -> &Self::Target {
//         &self.frp
//     }
// }

impl Command {
    /// Constructor.
    pub fn new(frp:frp::Source<()>) -> Self {
        let enabled = true;
        Self {frp,enabled}
    }
}

#[allow(missing_docs)]
pub trait CommandApi : Sized {
    fn command_api(&self) -> Rc<RefCell<HashMap<String,Command>>> { default() }
    fn status_api(&self) -> Rc<RefCell<HashMap<String,frp::Sampler<bool>>>> { default() }
}



// ========================
// === ProviderInstance ===
// ========================

/// Instance of command `Provider`. It contains bindings to all FRP endpoints defined by the
/// `Provider`. See the docs of `Provider` to learn more.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct ProviderInstance {
    pub network     : frp::WeakNetwork,
    pub command_map : Rc<RefCell<HashMap<String,Command>>>,
    pub status_map  : Rc<RefCell<HashMap<String,frp::Sampler<bool>>>>,
}

impl ProviderInstance {
    /// Check whether the underlying object is still alive.
    pub fn check_alive(&self) -> bool {
        self.network.upgrade().is_some()
    }

    /// The ID of this instance.
    pub fn id(&self) -> frp::NetworkId {
        self.network.id()
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
    pub logger   : Logger,
    pub name_map : Rc<RefCell<HashMap<String,Vec<ProviderInstance>>>>,
    pub id_map   : Rc<RefCell<HashMap<frp::NetworkId,ProviderInstance>>>,
}

impl Registry {
    /// Constructor.
    pub fn create(logger:impl AnyLogger) -> Self {
        let logger   = Logger::sub(logger,"views");
        let name_map = default();
        let id_map   = default();
        Self {logger,name_map,id_map}
    }

    /// Registers the command `Provider`.
    pub fn register<V:View>(&self) {
        let label  = V::label();
        let exists = self.name_map.borrow().get(label).is_some();
        if exists {
            warning!(&self.logger, "The view '{label}' was already registered.")
        } else {
            self.name_map.borrow_mut().insert(label.into(),default());
        }
    }

    /// Registers the command `ProviderInstance`.
    pub fn register_instance<T:View>(&self, target:&T) {
        let label       = T::label();
        let network     = T::network(target).downgrade();
        let command_map = target.deref().command_api();
        let status_map  = target.deref().status_api();
        let instance    = ProviderInstance {network,command_map,status_map};
        let was_registered = self.name_map.borrow().get(label).is_some();
        if !was_registered {
            self.register::<T>();
            warning!(&self.logger,
                "The command provider '{label}' was created but never registered. You should \
                always register available command providers as soon as possible to spread the \
                information about their API.");
        };
        let id = instance.id();
        self.name_map.borrow_mut().get_mut(label).unwrap().push(instance.clone_ref());
        self.id_map.borrow_mut().insert(id,instance);
    }

    fn with_command_mut<T:View>
    ( &self
    , target : &T
    , name   : impl AsRef<str>
    , f      : impl Fn(&mut Command)
    ) {
        let name = name.as_ref();
        let id   = T::network(target).id();
        match self.id_map.borrow_mut().get(&id) {
            None => warning!(&self.logger,"The provided component ID is invalid {id}."),
            Some(instance) => match instance.command_map.borrow_mut().get_mut(name) {
                None => warning!(&self.logger,"The command name {name} is invalid."),
                Some(command) => f(command)
            }
        }
    }

    fn disable_command<T:View>(&self, target:&T, name:impl AsRef<str>) {
        self.with_command_mut(target,name,|command| command.enabled = false)
    }

    fn enable_command<T:View>(&self, target:&T, name:impl AsRef<str>) {
        self.with_command_mut(target,name,|command| command.enabled = true)
    }
}
