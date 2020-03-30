
use crate::prelude::*;

use crate::display;
use crate::display::shape;
use crate::display::shape::*;
use crate::display::scene::MouseTarget;
use crate::display::scene::ShapeRegistry;
use crate::display::object::traits::*;
use crate::display::shape::primitive::system::Shape;
use crate::display::scene::Scene;


#[macro_export]
macro_rules! component {
    (
        $name:ident

        Definition {
            $($field:ident : $field_type:ty),* $(,)?
        }


    ) => {
        #[derive(Clone,CloneRef,Debug,Shrinkwrap)]
        pub struct $name ($crate::gui::component::ComponentWrapper<Definition>);

        impl<'t> From<&'t $name> for &'t display::object::Node {
            fn from(t:&'t $name) -> Self {
                t.0.display_object()
            }
        }

        impl $name {
            fn create(label:&str, definition:Definition) -> Self {
                let data = $crate::gui::component::ComponentWrapper::create(label,definition);
                Self(data)
            }
        }

        #[derive(Debug,Clone,CloneRef)]
        pub struct Definition {
            $(pub $field : $field_type),*
        }




    };
}





pub trait Component : MouseTarget + CloneRef + 'static {
    type ComponentSystem : ShapeSystem + CloneRef;

    fn on_view_cons(&self, scene:&Scene, shape_registry:&ShapeRegistry) {}
    fn on_view_drop(&self, scene:&Scene, shape_registry:&ShapeRegistry) {}
}

pub type ComponentSystem<T> = <T as Component>::ComponentSystem;





//#[derive(Debug,Derivative,Shrinkwrap)]
//#[derivative(Clone(bound="Definition:Clone"))]
//pub struct ComponentWrapperTemplate<Definition,Shape> {
//    #[shrinkwrap(main_field)]
//    pub definition     : Definition,
//    pub logger         : Logger,
//    pub display_object : display::object::Node,
//    pub shape          : Rc<RefCell<Option<Shape>>>,
//}
//
//
//pub type ComponentWrapper<Definition> = ComponentWrapperTemplate<Definition,shape::ShapeDefinition<ComponentSystem<Definition>>>;
//
//impl<Definition,Shape> CloneRef for ComponentWrapperTemplate<Definition,Shape>
//    where Definition : CloneRef {
//    fn clone_ref(&self) -> Self {
//        let definition     = self.definition.clone_ref();
//        let logger         = self.logger.clone_ref();
//        let display_object = self.display_object.clone_ref();
//        let shape          = self.shape.clone_ref();
//        Self {definition,logger,display_object,shape}
//    }
//}
//
//impl<'t,Definition,Shape>
//From<&'t ComponentWrapperTemplate<Definition,Shape>> for &'t display::object::Node {
//    fn from(t:&'t ComponentWrapperTemplate<Definition,Shape>) -> Self {
//        &t.display_object
//    }
//}
//
//impl<Definition:Component> ComponentWrapperTemplate<Definition,shape::ShapeDefinition<ComponentSystem<Definition>>> {
//    pub fn create(label:&str,definition:Definition) -> Self {
//        let logger         = Logger::new(label);
//        let display_object = display::object::Node::new(&logger);
//        let shape          = default();
//        Self {shape,definition,logger,display_object} . create_init()
//    }
//
//    fn create_init(self) -> Self {
//        let shape               = &self.shape;
//        let definition          = &self.definition;
//        let display_object_weak = self.display_object.downgrade();
//
//        self.display_object.set_on_show_with(enclose!((shape,definition) move |scene| {
//            let shape_registry : &ShapeRegistry = &scene.shapes;
//            definition.on_view_cons(scene,shape_registry);
//        }));
//
//        self.display_object.set_on_hide_with(enclose!((shape) move |scene| {
//            shape.borrow().as_ref().for_each(|shape| {
//                // TODO scene.shapes.remove_mouse_target(...)
//            });
//            *shape.borrow_mut() = None;
//        }));
//
//        self
//    }
//}

