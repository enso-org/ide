use basegl::display::object::DisplayObjectOps;
use basegl::display::shape::text::glyph::font::FontRegistry;
use basegl::display::shape::text::text_field::TextField;
use basegl::display::shape::text::text_field::TextFieldProperties;
use basegl::display::world::*;
use basegl::system::web;

use nalgebra::Vector2;
use nalgebra::Vector4;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::MouseEvent;

const TEXT:&str =
    "To be, or not to be, that is the question:
Whether 'tis nobler in the mind to suffer
The slings and arrows of outrageous fortune,
Or to take arms against a sea of troubles
And by opposing end them. To die—to sleep,
No more; and by a sleep to say we end
The heart-ache and the thousand natural shocks
That flesh is heir to: 'tis a consummation
Devoutly to be wish'd.";

pub struct TextEditor {
    text_field : TextField
}

impl TextEditor {
    pub fn new(world:&World) -> Self {
        let scene     = world.scene();
        let camera    = scene.camera();
        let screen    = camera.screen();
        let mut fonts = FontRegistry::new();
        let font_id   = fonts.load_embedded_font("DejaVuSansMono").unwrap();

        let properties = TextFieldProperties {
            font_id,
            text_size  : 16.0,
            base_color : Vector4::new(0.0, 0.0, 0.0, 1.0),
            size       : Vector2::new(screen.width, screen.height)
        };

        let mut text_field = TextField::new(&world,TEXT,properties,&mut fonts);
        text_field.set_position(Vector3::new(0.0, screen.height, 0.0));
        text_field.jump_cursor(Vector2::new(50.0, -40.0),false,&mut fonts);
        world.add_child(&text_field);
        text_field.update();

//        let c: Closure<dyn FnMut(JsValue)> = Closure::wrap(Box::new(move |val:JsValue| {
//            let val = val.unchecked_into::<MouseEvent>();
//            let x = val.x() as f32 - 10.0;
//            let y = (screen.height - val.y() as f32) - 600.0;
//            text_field.jump_cursor(Vector2::new(x,y),true,&mut fonts);
//        }));
//        web::document().unwrap().add_event_listener_with_callback
//        ("click",c.as_ref().unchecked_ref()).unwrap();
//        c.forget();
        Self {text_field}
    }
}