#![cfg_attr(test, allow(dead_code))]
#![feature(unboxed_closures)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(specialization)]
#![feature(associated_type_defaults)]
#![feature(set_stdio)]
//#![warn(missing_docs)]

// Lints. To be refactored after this gets resolved:
// https://github.com/rust-lang/cargo/issues/5034
#![allow(clippy::option_map_unit_fn)]

/// Uncomment the following code to enable macro debugging.
//#![feature(trace_macros)]
//#![recursion_limit="256"]
//trace_macros!(true);


// =================================
// === Module Structure Reexport ===
// =================================

pub mod animation;
pub mod control;
pub mod data;
pub mod debug;
pub mod display;
pub mod traits;

pub use basegl_prelude as prelude;
pub mod system {
    pub use basegl_system_web as web;
}


type InstanceId = usize;

// ==================
// === Example 01 ===
// ==================

mod example_01 {
    use super::*;
    use crate::set_stdout;
    use crate::display::world::*;
    use crate::prelude::*;
    use nalgebra::{Vector2, Vector3, Matrix4};
    use wasm_bindgen::prelude::*;
    use basegl_system_web::{Logger, get_performance};
    use web_sys::Performance;
    use crate::display::object::DisplayObjectData;
    use crate::display::object::DisplayObjectOps;


    #[derive(Clone,Debug)]
    pub struct SymbolRef {
        world        : World,
        symbol_id    : SymbolId,
    }

    impl SymbolRef {
        pub fn new(world: World, symbol_id:SymbolId) -> Self {
            Self {world,symbol_id}
        }
    }

    #[derive(Clone,Debug)]
    pub struct SpriteRef {
        symbol_ref  : SymbolRef,
        instance_id : InstanceId,
    }

    impl SpriteRef {
        pub fn new(symbol_ref:SymbolRef, instance_id:InstanceId) -> Self {
            Self {symbol_ref,instance_id}
        }
    }


    pub struct Sprite {
        rc: Rc<RefCell<SpriteData>>
    }

    impl Sprite {
        pub fn new(sprite_ref:SpriteRef, transform:Var<Matrix4<f32>>, bbox:Var<Vector2<f32>>) -> Self {
            let data = SpriteData::new(sprite_ref,transform,bbox);
            let rc   = Rc::new(RefCell::new(data));
            Self {rc}
        }

        pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
            self.rc.borrow().display_object.mod_position(f);
        }

        pub fn set_position(&self, value:Vector3<f32>) {
            self.rc.borrow().display_object.set_position(value)
        }
    }

    impl From<&Sprite> for DisplayObjectData {
        fn from(t:&Sprite) -> Self {
            t.rc.borrow().display_object.clone_ref()
        }
    }

    pub struct SpriteData {
        sprite_ref     : SpriteRef,
        display_object : DisplayObjectData,
        transform      : Var<Matrix4<f32>>,
        bbox           : Var<Vector2<f32>>,
    }

    impl SpriteData {
        pub fn new
        (sprite_ref:SpriteRef, transform:Var<Matrix4<f32>>, bbox:Var<Vector2<f32>>) -> Self {
            let logger         = Logger::new(format!("Sprite{}",sprite_ref.instance_id));
            let display_object = DisplayObjectData::new(logger);
            let transform_cp   = transform.clone();
            display_object.set_on_updated(move |t| {
                transform_cp.set(t.matrix().clone());
            });
            Self {sprite_ref,display_object,transform,bbox}
        }
    }

    pub struct SpriteSystem {
        display_object : DisplayObjectData,
        symbol_ref     : SymbolRef,
        transform      : Buffer<Matrix4<f32>>,
        uv             : Buffer<Vector2<f32>>,
        bbox           : Buffer<Vector2<f32>>,
    }

    impl SpriteSystem {
        pub fn new(world:&World) -> Self {
            let logger         = Logger::new("SpriteSystem");
            let display_object = DisplayObjectData::new(logger);
            let world_data     = &mut world.borrow_mut();
            let workspace      = &mut world_data.workspace;
            let symbol_id      = workspace.new_symbol();
            let symbol         = &mut workspace[symbol_id];
            let mesh           = &mut symbol.surface;
            let uv             = mesh.scopes.point.add_buffer("uv");
            let transform      = mesh.scopes.instance.add_buffer("transform");
            let bbox           = mesh.scopes.instance.add_buffer("bbox");

            let p1_index = mesh.scopes.point.add_instance();
            let p2_index = mesh.scopes.point.add_instance();
            let p3_index = mesh.scopes.point.add_instance();
            let p4_index = mesh.scopes.point.add_instance();

            uv.get(p1_index).set(Vector2::new(0.0, 0.0));
            uv.get(p2_index).set(Vector2::new(0.0, 1.0));
            uv.get(p3_index).set(Vector2::new(1.0, 0.0));
            uv.get(p4_index).set(Vector2::new(1.0, 1.0));

            let world      = world.clone_ref();
            let symbol_ref = SymbolRef::new(world,symbol_id);
            Self {display_object,symbol_ref,transform,uv,bbox}
        }

        pub fn new_instance(&self) -> Sprite {
            let world_data   = &mut self.symbol_ref.world.borrow_mut();
            let symbol       = &mut world_data.workspace[self.symbol_ref.symbol_id];
            let instance_id  = symbol.surface.instance.add_instance();
            let transform    = self.transform.get(instance_id);
            let bbox         = self.bbox.get(instance_id);
            let sprite_ref   = SpriteRef::new(self.symbol_ref.clone(),instance_id);
            bbox.set(Vector2::new(2.0,2.0));
            let sprite = Sprite::new(sprite_ref,transform,bbox);
            self.add_child(&sprite);
            sprite
        }
    }

    impl From<&SpriteSystem> for DisplayObjectData {
        fn from(t:&SpriteSystem) -> Self {
            t.display_object.clone_ref()
        }
    }



    #[wasm_bindgen]
    #[allow(dead_code)]
    pub fn run_example_basic_objects_management() {
        set_panic_hook();
        console_error_panic_hook::set_once();
        set_stdout();
        init(&WorldData::new("canvas"));
    }

    #[derive(Debug)]
    pub struct Rect {
        position : Var<Vector2<f32>>,
        color    : Var<Vector3<f32>>,
    }

    fn init(world: &World) {

        let sprite_system = SpriteSystem::new(world);

        let sprite1 = sprite_system.new_instance();
        sprite1.mod_position(|t| t.y += 0.5);
        sprite1.rc.borrow().display_object.update();


        let mut sprites: Vec<Sprite> = default();
        let count = 100;
        for i in 0 .. count {
            let sprite = sprite_system.new_instance();
            sprites.push(sprite);
        }
//
        let performance = get_performance().unwrap();
//
//
        let mut i:i32 = 0;
        world.on_frame(move |_| on_frame(&mut i,&sprite1,&mut sprites,&performance,&sprite_system)).forget();


    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::many_single_char_names)]
    pub fn on_frame(ii:&mut i32, sprite1:&Sprite, sprites:&mut Vec<Sprite>, performance:&Performance,sprite_system:&SpriteSystem) {
//        camera.mod_position(|p| {
//            p.x -= 0.1;
//            p.z += 1.0
//        });


        if *ii < 50i32 {
            sprite1.mod_position(|p| p.y += 0.5);
//            sprite1.update();
            sprite_system.update();

        }

        *ii += 1;

        if *ii < 1000i32 {
//            let count = 100;
//            if sprites.len() < 100_000 {
//                for _ in 0..count {
//                    let widget = make_widget(inst_scope);
//                    sprites.push(widget);
//                }
//            }

            let t = (performance.now() / 1000.0) as f32;
            let length = sprites.len() as f32;
            for (i, sprite) in sprites.iter_mut().enumerate() {
                let i = i as f32;
                let d = (i / length - 0.5) * 2.0;

                let mut y = d;
                let r = (1.0 - y * y).sqrt();
                let mut x = (y * 100.0 + t).cos() * r;
                let mut z = (y * 100.0 + t).sin() * r;

                x += (y * 1.25 + t * 2.50).cos() * 0.5;
                y += (z * 1.25 + t * 2.00).cos() * 0.5;
                z += (x * 1.25 + t * 3.25).cos() * 0.5;
                sprite.set_position(Vector3::new(x * 50.0 + 200.0, y * 50.0 + 100.0, z * 50.0));
//            sprite.transform.set_position(Vector3::new(0.0, 0.0, 0.0));
//                sprite.update();

//            let faster_t = t * 100.0;
//            let r = (i +   0.0 + faster_t) as u8 % 255;
//            let g = (i +  85.0 + faster_t) as u8 % 255;
//            let b = (i + 170.0 + faster_t) as u8 % 255;
//            set_gradient_bg(&object.dom, &r.into(), &g.into(), &b.into());
            }

            sprite_system.update();

        }




    }


    pub struct Widget {
        pub transform : DisplayObjectData,
        pub mm        : Var<Matrix4<f32>>,
    }

    impl Widget {
        pub fn new(logger:Logger, mm:Var<Matrix4<f32>>) -> Self {
            let transform = DisplayObjectData::new(logger);
            let mm_cp = mm.clone();
            transform.set_on_updated(move |t| {
                mm_cp.set(t.matrix().clone());
            });
            Self {transform,mm}
        }
    }
}

// ==================
// === Example 03 ===
// ==================

mod example_03 {
    use super::*;
    use wasm_bindgen::prelude::*;

    use crate::display::world::{WorldData, Workspace, Add};
    use crate::display::shape::text::font::FontId;
    use crate::Color;
    use crate::data::dirty::traits::*;

    use itertools::iproduct;
    use nalgebra::{Point2,Vector2};

    const FONT_NAMES : &[&str] = &
        [ "DejaVuSans"
        , "DejaVuSansMono"
        , "DejaVuSansMono-Bold"
        , "DejaVuSerif"
        ];

    const SIZES : &[f64] = &[0.024, 0.032, 0.048];

    #[wasm_bindgen]
    #[allow(dead_code)]
    pub fn run_example_text() {
        set_panic_hook();
        basegl_core_msdf_sys::run_once_initialized(|| {
            let mut world_ref = WorldData::new("canvas");
            let world :&mut WorldData = &mut world_ref.borrow_mut();
            let workspace     = &mut world.workspace;
            let fonts         = &mut world.fonts;
            let font_ids_iter = FONT_NAMES.iter().map(|name| fonts.load_embedded_font(name).unwrap());
            let font_ids      = font_ids_iter.collect::<Box<[FontId]>>();

            let all_cases     = iproduct!(0..font_ids.len(), 0..SIZES.len());

            for (font, size) in all_cases {

                let x = -0.95 + 0.6 * (size as f64);
                let y = 0.90 - 0.45 * (font as f64);
                let text_compnent = crate::display::shape::text::TextComponentBuilder {
                    workspace,
                    fonts,
                    text : "To be, or not to be, that is the question:\n\
                        Whether 'tis nobler in the mind to suffer\n\
                        The slings and arrows of outrageous fortune,\n\
                        Or to take arms against a sea of troubles\n\
                        And by opposing end them."
                        .to_string(),
                    font_id: font_ids[font],
                    position: Point2::new(x, y),
                    size: Vector2::new(0.5, 0.2),
                    text_size: SIZES[size],
                    color    : Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0},
                }.build();
                workspace.text_components.push(text_compnent);
            }
            world.workspace_dirty.set();

//            world.on_frame(move |w| {
//                let space = &mut w.workspace;
//                for text_component in &mut space.text_components {
//                    text_component.scroll(Vector2::new(0.0,0.00001));
//                }
//                w.workspace_dirty.set();
//            }).forget();
        });
    }
}


// =================
// === Utilities ===
// =================

#[derive(Debug)]
pub struct Color<T> {
    pub r : T,
    pub g : T,
    pub b : T,
    pub a : T,
}

#[derive(Debug)]
pub struct Area<T> {
    pub left   : T,
    pub right  : T,
    pub top    : T,
    pub bottom : T,
}

impl<T:std::ops::Sub+Clone> Area<T> {
    pub fn width(&self) -> T::Output {
        self.right.clone() - self.left.clone()
    }

    pub fn height(&self) -> T::Output {
        self.top.clone() - self.bottom.clone()
    }
}

// ===============
// === Printer ===
// ===============

type PrintFn = fn(&str) -> std::io::Result<()>;

struct Printer {
    printfn: PrintFn,
    buffer: String,
    is_buffered: bool,
}

impl Printer {
    fn new(printfn: PrintFn, is_buffered: bool) -> Printer {
        Printer {
            buffer: String::new(),
            printfn,
            is_buffered,
        }
    }
}

impl std::io::Write for Printer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.push_str(&String::from_utf8_lossy(buf));

        if !self.is_buffered {
            (self.printfn)(&self.buffer)?;
            self.buffer.clear();

            return Ok(buf.len());
        }

        if let Some(i) = self.buffer.rfind('\n') {
            let buffered = {
                let (first, last) = self.buffer.split_at(i);
                (self.printfn)(first)?;

                String::from(&last[1..])
            };

            self.buffer.clear();
            self.buffer.push_str(&buffered);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        (self.printfn)(&self.buffer)?;
        self.buffer.clear();

        Ok(())
    }
}

fn _print(msg: &str) -> std::io::Result<()> {
    web_sys::console::info_1(&msg.to_string().into());
    Ok(())
}


pub fn set_stdout() {
    let printer = Printer::new(_print, true);
    std::io::set_print(Some(Box::new(printer)));
}

pub fn set_stdout_unbuffered() {
    let printer = Printer::new(_print, false);
    std::io::set_print(Some(Box::new(printer)));
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
}
