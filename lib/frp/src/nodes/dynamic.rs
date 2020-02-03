//! Root module for FRP Dynamic node types. The Dynamic type is a generalization of Event and
//! Behavior and is very easy to work with. You should use this type in most (all?) cases in your
//! FRP flows.

use crate::prelude::*;

use crate::data::*;
use crate::node::*;
use crate::nodes::prim::*;
use crate::nodes::lambda::*;



// ===============
// === Dynamic ===
// ===============

/// The `Dynamic` type is an `Event` with an associated `Behavior`. You can assume that the
/// behavior just always holds the last event value.
#[derive(Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Dynamic<Out:Value> {
    /// The `Event` component.
    pub event : Event<Out>,
    /// The `Behavior` component.
    pub behavior : Behavior<Out>,
}

impl<Out:Value> Dynamic<Out> {
    /// Constructor.
    pub fn new<E,B>(event:E, behavior:B) -> Self
        where E:Into<Event<Out>>, B:Into<Behavior<Out>> {
        let event    = event.into();
        let behavior = behavior.into();
        Self {event,behavior}
    }

    /// Drops the current value and outputs a constant.
    pub fn constant<Label,T>(&self, label:Label, value:T) -> Dynamic<T>
        where Label:Into<CowString>, T:Value {
        self.map(label,move |_| value.clone())
    }

    /// Merges two event streams. The output event will be emitted whenever one of the streams emit
    /// an event.
    pub fn merge<Label>(&self, label:Label, that:&Dynamic<Out>) -> Self
        where Label:Into<CowString> {
        (&Merge::new_named(label,&self.event,&that.event)).into()
    }

    /// Consumes the input event and switches the output from `false` to `true` and vice versa.
    pub fn toggle<Label>(&self, label:Label) -> Dynamic<bool>
        where Label:Into<CowString> {
        (&Toggle::new_named(label,&self.event)).into()
    }

    /// Passes the event trough only if its argument evaluates to `true`.
    pub fn gate<Label>(&self, label:Label, that:&Dynamic<bool>) -> Self
        where Label:Into<CowString> {
        (&Gate::new_named(label,that,self)).into()
    }

    /// Samples this behavior whenever a new event appears in the argument's event stream. The
    /// input event is dropped and a new event of the behavior's value is generated.
    pub fn sample<Label,T>(&self, label:Label, that:&Dynamic<T>) -> Self
        where Label : Into<CowString>,
              T     : Value {
        (&Sample::new_named(label,&self.behavior,that)).into()
    }

    /// Maps the current value with the provided lambda. This is one of the most powerful utilities,
    /// however, you should try not to use it too often. The reason is that it also makes
    /// optimizations impossible, as lambdas are like "black-boxes" for the FRP engine.
    pub fn map<Label,F,R>(&self, label:Label, f:F) -> Dynamic<R>
        where Label : Into<CowString>,
              R     : Value,
              F     : 'static + Fn(&Out) -> R {
        (&Lambda::new_named(label,&self.event,f)).into()
    }

    /// Maps the current value with the provided lambda. This is one of the most powerful utilities,
    /// however, you should try not to use it too often. The reason is that it also makes
    /// optimizations impossible, as lambdas are like "black-boxes" for the FRP engine.
    pub fn map2<Label,T,F,R>(&self, label:Label, that:&Dynamic<T>, f:F) -> Dynamic<R>
        where Label : Into<CowString>,
              T     : Value,
              R     : Value,
              F     : 'static + Fn(&Out,&T) -> R {
        (&Lambda2::new_named(label,&self.event,that,f)).into()
    }
}

impl<Out:Value> Dynamic<Out> {
    /// Creates a new FRP source.
    pub fn source<Label>(label:Label) -> Self
        where Label : Into<CowString> {
        let event = Source::<EventData<Out>>::new_named(label);
        (&event).into()
    }
}

// === Instances ===

impl<Out:Value> CloneRef for Dynamic<Out> {
    fn clone_ref(&self) -> Self {
        let event    = self.event.clone_ref();
        let behavior = self.behavior.clone_ref();
        Self {event,behavior}
    }
}

impl<Out:Value, T:Into<Event<Out>>> From<T> for Dynamic<Out> {
    fn from(t:T) -> Self {
        let event    = t.into();
        let behavior = Hold :: new_named(event.label(),&event);
        behavior.set_display_id(event.display_id());
        let event    = (&event).into();
        let behavior = (&behavior).into();
        Dynamic {event,behavior}
    }
}

impl<Out:Value> From<&Dynamic<Out>> for Event<Out> {
    fn from(t:&Dynamic<Out>) -> Self {
        t.event.clone_ref()
    }
}

impl<Out:Value> From<&Dynamic<Out>> for Behavior<Out> {
    fn from(t:&Dynamic<Out>) -> Self {
        t.behavior.clone_ref()
    }
}
