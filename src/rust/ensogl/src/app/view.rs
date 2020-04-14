
use crate::prelude::*;
use crate::frp;
use crate::frp::io::keyboard;
use crate::display::world::World;



pub struct FrpEndpointDefinition<T> {
    pub label : String,
    pub frp   : T
}

impl<T> FrpEndpointDefinition<T> {
    pub fn new<L,X>(label:L, frp:X) -> Self
        where L:Into<String>, X:Into<T> {
        let label = label.into();
        let frp   = frp.into();
        Self {label,frp}
    }
}

pub struct FrpEndpointDocs {
    pub label   : String,
    pub caption : String,
}

impl FrpEndpointDocs {
    pub fn new<L,C>(label:L, caption:C) -> Self
        where L:Into<String>, C:Into<String> {
        let label   = label.into();
        let caption = caption.into();
        Self {label,caption}
    }
}


#[derive(Debug)]
pub struct FrpEndpoint<S> {
    pub caption : String,
    pub frp     : S,
}


use super::App; // FIXME
use super::shortcut; // FIXME
use super::shortcut::Shortcut; // FIXME

pub trait View : FrpNetworkProvider + CommandProvider + StatusProvider {
    fn view_name() -> &'static str;
    fn new(world:&World) -> Self;
    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        default()
    }

    fn self_shortcut<M,C>(key_mask:M, command:C, condition:shortcut::Condition) -> Shortcut
    where M:Into<keyboard::KeyMask>, C:Into<shortcut::Command> {
        Shortcut::new(key_mask,Self::view_name(),command,condition)
    }

    fn self_shortcut_<M,C>(key_mask:M, command:C) -> Shortcut
    where M:Into<keyboard::KeyMask>, C:Into<shortcut::Command> {
        Shortcut::new_(key_mask,Self::view_name(),command)
    }

//    fn default_shortcuts() -> Vec<app::shortcut::Shortcut> {
//        use app::shortcut::Shortcut;
//        use keyboard::Key;
//        vec! [ Shortcut::new_(&[Key::Character("n".into())],Self::view_name(),"add_node_at_cursor")
//               , Shortcut::new_(&[Key::Backspace],Self::view_name(),"remove_selected_nodes")
//        ]
//    }
}

#[derive(Debug)]
pub struct Instance {
    pub nnn     : frp::WeakNetwork,
    pub command_map : HashMap<String,Command>,
    pub status_map  : HashMap<String,Status>,
}

impl Instance {
    pub fn check_alive(&self) -> bool {
        self.nnn.upgrade().is_some()
    }
}


#[derive(Debug,Clone,CloneRef)]
pub struct Registry {
    pub display : World,
    pub map     : Rc<RefCell<HashMap<String,Vec<Instance>>>>,
}

impl Registry {
    pub fn create(display:&World) -> Self {
        let display = display.clone_ref();
        let map     = default();
        Self {display,map}
    }

    pub fn register<V:View>(&self) {

    }

    pub fn new<V:View>(&self) -> V {
        let view    = V::new(&self.display);
        let label   = V::view_name().into();
        let nnn = V::network(&view).downgrade();
        let command_doc_map : HashMap<String,String> = V::command_api_docs().into_iter().map(|t| {
            (t.label,t.caption)
        }).collect();
        let command_map = V::command_api(&view).into_iter().map(|t| {
            let caption = command_doc_map.get(&t.label).unwrap().clone(); // fixme unwrap
            let frp     = t.frp;
            let endpoint = FrpEndpoint {caption,frp};
            (t.label,endpoint)
        }).collect();

        let status_doc_map : HashMap<String,String> = V::status_api_docs().into_iter().map(|t| {
            (t.label,t.caption)
        }).collect();
        let status_map = V::status_api(&view).into_iter().map(|t| {
            let caption = status_doc_map.get(&t.label).unwrap().clone(); // fixme unwrap
            let frp     = t.frp;
            let endpoint = FrpEndpoint {caption,frp};
            (t.label,endpoint)
        }).collect();

        let module_instance = Instance {nnn,command_map,status_map};
        self.map.borrow_mut().entry(label).or_default().push(module_instance);
        view
    }
}



pub trait FrpNetworkProvider {
    fn network(&self) -> &frp::Network;
}

pub trait CommandProvider : Sized {
    fn command_api_docs() -> Vec<FrpEndpointDocs>;
    fn command_api(&self) -> Vec<CommandDefinition>;
}

pub type Command           = FrpEndpoint<frp::Source>;
pub type Status            = FrpEndpoint<frp::Sampler<bool>>;
pub type CommandDefinition = FrpEndpointDefinition<frp::Source>;
pub type StatusDefinition  = FrpEndpointDefinition<frp::Sampler<bool>>;


pub trait StatusProvider : Sized {
    fn status_api_docs() -> Vec<FrpEndpointDocs>;
    fn status_api(&self) -> Vec<StatusDefinition>;
}
