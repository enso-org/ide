#![feature(type_ascription)]
#![feature(unboxed_closures)]
#![cfg_attr(test, allow(dead_code))]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(proc_macro_hygiene)]
#![feature(specialization)]
#![feature(weak_into_raw)]
#![feature(associated_type_defaults)]
#![feature(set_stdio)]
#![feature(overlapping_marker_traits)]
//#![warn(missing_docs)]


// Lints. To be refactored after this gets resolved: https://github.com/rust-lang/cargo/issues/5034
#![allow(clippy::option_map_unit_fn)]
#![feature(trace_macros)]
#![recursion_limit="256"]
trace_macros!(true);

// TODO: remove unstable features unless one will get stabilized soon

// =================================
// === Module Structure Reexport ===
// =================================

pub mod data;
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

use display::world::*;
use wasm_bindgen::prelude::*;

use system::web::Logger;
use system::web::fmt;

use bit_field::BitField;
use crate::display::symbol::scope::Scope;
use crate::display::symbol::attribute;
use crate::display::symbol::geometry;
use crate::display::symbol::mesh;
use nalgebra;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use nalgebra::Matrix;
use nalgebra::base::dimension::U1;
use nalgebra::base::dimension::U2;
use prelude::*;

use optics;


macro_rules! map {
    ($f:ident, $args:tt) => {
        map_impl!{ [], $f, $args }
     };
}

macro_rules! map_impl {
    ($out:tt       , $f:ident, []) => { $out };
    ([$($out:tt)*] , $f:ident, [$t1:tt]) => { 
        map_impl!([$($out)* $f!($t1)], $f, []);
    };        
    ([$($out:tt)*], $f:ident, [$t1:tt, $($ts:tt)*]) => { 
        map_impl!([$($out)* $f!($t1),], $f, [$($ts)*]);
    }
}

macro_rules! length {
    ([]) => { 0 };
    ([$t1:tt]) => { 1 };
    ([$t1:tt,$t2:tt]) => { 2 };
    ([$t1:tt,$t2:tt,$t3:tt]) => { 3 };
    ([$t1:tt,$t2:tt,$t3:tt,$t4:tt]) => { 4 };
    ([$t1:tt,$t2:tt,$t3:tt,$t4:tt,$t5:tt]) => { 5 };
    ([$t1:tt,$t2:tt,$t3:tt,$t4:tt,$t5:tt,$t6:tt]) => { 6 };
    ([$t1:tt,$t2:tt,$t3:tt,$t4:tt,$t5:tt,$t6:tt,$t7:tt]) => { 7 };
    ([$t1:tt,$t2:tt,$t3:tt,$t4:tt,$t5:tt,$t6:tt,$t7:tt,$t8:tt]) => { 8 };
    ([$t1:tt,$t2:tt,$t3:tt,$t4:tt,$t5:tt,$t6:tt,$t7:tt,$t8:tt,$t9:tt]) => { 9 };
}

macro_rules! decrement {
    (1) => { 0 };
    (2) => { 1 };
    (3) => { 2 };
    (4) => { 3 };
    (5) => { 4 };
    (6) => { 5 };
    (7) => { 6 };
    (8) => { 7 };
    (9) => { 8 };
}



pub fn test<'t>(vp:&'t[f32]) -> &'t [Vector3<f32>] {
    unsafe {
        std::slice::from_raw_parts(vp.as_ptr().cast(), vp.len() / 3)
    } 
}


use std::ops::Index;
use rustc_hash::FxHashSet;
use std::collections::HashSet;
use crate::display::mesh_registry::MeshRegistry;
use console_error_panic_hook;

type PositionID = BufferIndex<Vector2<f32>>;
type Position   = Buffer<Vector2<f32>>;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    set_stdout();

    init(&mut World::new().borrow_mut());
}

fn init(world: &mut World) {
    let logger = Logger::new("root");

    let wspace_id : WorkspaceID         = world.add(Workspace::build("canvas"));
    // let wspace_id : WorkspaceID         = world.add_workspace("canvas");
    let workspace : &mut Workspace      = &mut world[wspace_id];
    let mesh_id   : MeshID              = workspace.new_mesh();
    let mesh      : &mut Mesh           = &mut workspace[mesh_id];
    let geo       : &mut Geometry       = &mut mesh.geometry;
    let scopes    : &mut Scopes         = &mut geo.scopes;
    let pt_scope  : &mut AttributeScope = &mut scopes.point;
    let pos_id    : PositionID          = pt_scope.add_attribute("position", Buffer::builder());

    let logger = Logger::new("mesh_registry");

//
//
//    let mut mesh_registry = MeshRegistry::new(logger, ());
//    let mesh1_ix = mesh_registry.new_mesh();
//
//    let logger = Logger::new("mesh1");
//    let mut mesh1 = mesh::Mesh::new(logger, ());
//
//    // let logger = Logger::new("geo1");
//    // let mut geo1 = Geometry::new(logger, ());
//    let geo1 = &mut mesh1.geometry;

    println!("---- 1");
    let inst_ix = pt_scope.add_instance();
    println!("---- ix: {} ", inst_ix);

    let pos       : &Position            = &pt_scope[pos_id];


    let pos_view: View<Vector2<f32>> = pos.view(inst_ix);

    world.on_frame(move |w| on_frame(w, wspace_id, mesh_id, pos_id, &pos_view)).forget();

}

pub fn on_frame(world: &mut World, wspace_id: WorkspaceID, mesh_id: MeshID, pos_id: PositionID, pos_view: &View<Vector2<f32>>) {
    let workspace : &mut Workspace      = &mut world[wspace_id];
    let mesh      : &mut Mesh           = &mut workspace[mesh_id];
    let geo       : &mut Geometry       = &mut mesh.geometry;
    let scopes    : &mut Scopes         = &mut geo.scopes;
    let pt_scope  : &mut AttributeScope = &mut scopes.point;
    let pos       : &mut Position       = &mut pt_scope[pos_id];

    pos_view.modify(|p| p.x += 1.0)

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