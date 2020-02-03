use crate::prelude::*;

use crate::data::*;
use crate::node::*;
use crate::nodes::prim::*;
use crate::nodes::lambda::*;



// ===============
// === Dynamic ===
// ===============

pub struct Dynamic<Out:Value> {
    pub event    : Event    <Out>,
    pub behavior : Behavior <Out>,
}

impl<Out:Value> Dynamic<Out> {
    pub fn new<E,B>(event:E, behavior:B) -> Self
        where E:Into<Event<Out>>, B:Into<Behavior<Out>> {
        let event    = event.into();
        let behavior = behavior.into();
        Self {event,behavior}
    }

    pub fn merge<Label>(&self, label:Label, that:&Dynamic<Out>) -> Self
        where Label:Into<CowString> {
        (&Merge::new_named(label,&self.event,&that.event)).into()
    }

    pub fn toggle<Label>(&self, label:Label) -> Dynamic<bool>
        where Label:Into<CowString> {
        (&Toggle::new_named(label,&self.event)).into()
    }

    pub fn gate<Label>(&self, label:Label, that:&Dynamic<bool>) -> Self
        where Label:Into<CowString> {
        (&Gate::new_named(label,that,self)).into()
    }

    pub fn sample<Label,T>(&self, label:Label, that:&Dynamic<T>) -> Self
        where Label : Into<CowString>,
              T     : Value {
        (&Sample::new_named(label,&self.behavior,that)).into()
    }

    pub fn map<Label,F,R>(&self, label:Label, f:F) -> Dynamic<R>
        where Label : Into<CowString>,
              R     : Value,
              F     : 'static + Fn(&Out) -> R {
        (&Lambda::new_named(label,&self.event,f)).into()
    }

    pub fn map2<Label,T,F,R>(&self, label:Label, that:&Dynamic<T>, f:F) -> Dynamic<R>
        where Label : Into<CowString>,
              T     : Value,
              R     : Value,
              F     : 'static + Fn(&Out,&T) -> R {
        (&Lambda2::new_named(label,&self.event,that,f)).into()
    }

    pub fn constant<Label,T>(&self, label:Label, value:T) -> Dynamic<T>
        where Label:Into<CowString>, T:Value {
        self.map(label,move |_| value.clone())
    }
}

impl<Out:Value> Dynamic<Out> {
    pub fn source<Label>(label:Label) -> Self
        where Label : Into<CowString> {
        let event = Source::<EventData<Out>>::new_named(label);
        (&event).into()
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
