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
//#![feature(trace_macros)]
//#![recursion_limit="256"]
//trace_macros!(true);


// =================================
// === Module Structure Reexport ===
// =================================

pub mod control;
pub mod data;
pub mod math;
pub mod dirty;
pub mod display;
pub use basegl_prelude as prelude;
pub mod backend {
    pub use basegl_backend_webgl as webgl;
}
pub mod system {
    pub use basegl_system_web as web;
}
pub mod tp;


// ============
// === Main ===
// ============

use console_error_panic_hook;
use display::world::*;
use nalgebra;
use nalgebra::Vector3;
use nalgebra::Vector2;
use nalgebra::Matrix4;
use wasm_bindgen::prelude::*;

use display::symbol::material::shader;

type Position = SharedBuffer<Vector3<f32>>;
type ModelMatrix = SharedBuffer<Matrix4<f32>>;

use basegl_prelude::IsRc;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    set_stdout();
    init(&mut World::new().borrow_mut());
}

//#[derive(Debug)]
//pub struct Rect {
//    position : Var<Vector3<f32>>,
//}

use display::symbol::display_object::*;
use basegl_system_web::Logger;

fn init(world: &mut World) {
    let wspace_id : WorkspaceID    = world.add(Workspace::build("canvas"));
    let workspace : &mut Workspace = &mut world[wspace_id];
    let mesh_id   : MeshID         = workspace.new_mesh();
    let mesh      : &mut Mesh      = &mut workspace[mesh_id];
    let geo       : &mut Geometry  = &mut mesh.geometry;
    let scopes    : &mut Scopes    = &mut geo.scopes;
    let pt_scope  : &mut VarScope  = &mut scopes.point;
    let pos       : Position       = pt_scope.add_buffer("position");
    let model_matrix : ModelMatrix = pt_scope.add_buffer("model_matrix");
    let uv           : SharedBuffer<Vector2<f32>> = pt_scope.add_buffer("uv");
    let bbox         : SharedBuffer<Vector2<f32>> = pt_scope.add_buffer("bbox");

    let p1_ix = pt_scope.add_instance();
    let p2_ix = pt_scope.add_instance();
    let p3_ix = pt_scope.add_instance();
    let p4_ix = pt_scope.add_instance();

    let p1 = pos.get(p1_ix);
    let p2 = pos.get(p2_ix);
    let p3 = pos.get(p3_ix);
    let p4 = pos.get(p4_ix);


    p1.set(Vector3::new(-0.0, -0.0, 0.0));
    p2.set(Vector3::new( 0.0, -0.0, 0.0));
    p3.set(Vector3::new( 0.0,  0.0, 0.0));
    p4.set(Vector3::new( 0.0,  0.0, 0.0));


    let uv1 = uv.get(p1_ix);
    let uv2 = uv.get(p2_ix);
    let uv3 = uv.get(p3_ix);
    let uv4 = uv.get(p4_ix);

    uv1.set(Vector2::new(0.0, 0.0));
    uv2.set(Vector2::new(0.0, 1.0));
    uv3.set(Vector2::new(1.0, 0.0));
    uv4.set(Vector2::new(1.0, 1.0));

    let bbox1 = bbox.get(p1_ix);
    let bbox2 = bbox.get(p2_ix);
    let bbox3 = bbox.get(p3_ix);
    let bbox4 = bbox.get(p4_ix);

    bbox1.set(Vector2::new(0.5, 0.5));
    bbox2.set(Vector2::new(0.5, 0.5));
    bbox3.set(Vector2::new(0.5, 0.5));
    bbox4.set(Vector2::new(0.5, 0.5));


    let mm1 = model_matrix.get(p1_ix);
    let mm2 = model_matrix.get(p2_ix);
    let mm3 = model_matrix.get(p3_ix);
    let mm4 = model_matrix.get(p4_ix);

    mm1.modify(|t| {t.append_translation_mut(&Vector3::new(-1.0, -1.0, 0.0));});
    mm2.modify(|t| {t.append_translation_mut(&Vector3::new(-1.0,  1.0, 0.0));});
    mm3.modify(|t| {t.append_translation_mut(&Vector3::new( 1.0, -1.0, 0.0));});
    mm4.modify(|t| {t.append_translation_mut(&Vector3::new( 1.0,  1.0, 0.0));});
//    mm5.modify(|t| {t.append_translation_mut(&Vector3::new(-1.0,  1.0, 0.0));});
//    mm6.modify(|t| {t.append_translation_mut(&Vector3::new(-1.0, -1.0, 0.0));});
//
//    mm1.set(Matrix4::new( 0.0,  0.0, 0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0));
//    mm2.set(Matrix4::new( 0.0,  0.0, 0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0));
//    mm3.set(Matrix4::new( 0.0,  0.0, 0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0));
//
//    mm4.set(Matrix4::new( 0.0,  0.0, 0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0));
//    mm5.set(Matrix4::new( 0.0,  0.0, 0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0));
//    mm6.set(Matrix4::new( 0.0,  0.0, 0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0));

    println!("---------- {:?}", *mm1.get());

//    println!("{:?}",pos);
//    println!("{:?}",pos.borrow().as_prim());

//    world.on_frame(move |_| on_frame(&p6)).forget();


    shader::main();

    println!("------------ 1");
    let obj1 = HierarchicalTransform::new(Logger::new("obj1"));
    let obj2 = HierarchicalTransform::new(Logger::new("obj2"));
    let obj3 = HierarchicalTransform::new(Logger::new("obj3"));
    obj1.add_child(&obj2);
    obj1.update();
    println!("------------ 2");
    obj1.mod_position(|t| t.x += 5.0);
    obj2.mod_position(|t| t.y += 6.0);
    obj1.update();
    println!("------------ 3");
    obj1.remove_child(&obj2);
    obj2.update();
    println!("{:?}",obj2.global_position());

    println!("------------ 4");

    let obj1 = HierarchicalObject::new(Logger::new("obj1"));
    let obj2 = HierarchicalObject::new(Logger::new("obj2"));
    obj1.add_child(&obj2);

    println!("------------ 5");
    println!("{:?}",obj2.index());



//    obj2.mod_position(|t| t.y += 5.0);
//
//    obj1.mod_rotation(|t| t.z += 90.0);
////    obj1.mod_rotation(|t| t.y += 3.0);
//    obj1.update();
//    println!(">>> {:?}", obj2.global_position());
//    println!(">>> {:?}", obj1.matrix());
//    println!("------------ 4");
//    obj1.update();
//    println!(">>> {:?}", obj2.global_position());
//    println!(">>> {:?}", obj2.matrix());
//
//
//    println!("============");

}

pub fn on_frame(p: &Var<Vector3<f32>>) {
     p.modify(|t| t.x += 0.01)
}

////////////////////////////////////////////////
////////////////////////////////////////////////

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




