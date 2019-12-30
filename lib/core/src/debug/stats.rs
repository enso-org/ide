//! This module defines a structure gathering statistics of the running engine. The statistics are
//! an amazing tool for debugging what is really happening under the hood and understanding the
//! performance characteristics.

use crate::prelude::*;



// =============
// === Stats ===
// =============

/// Structure containing all the gathered stats.
#[derive(Debug,Clone)]
pub struct Stats {
    rc: Rc<RefCell<StatsData>>
}

impl Default for Stats {
    fn default() -> Self {
        let rc = Rc::new(RefCell::new(default()));
        Self {rc}
    }
}

impl Stats {
    /// Cheap, reference-based clone.
    pub fn clone_ref(&self) -> Self {
        self.clone()
    }

    /// Resets the per-frame statistics.
    pub fn reset(&self) {
        self.rc.borrow_mut().reset();
    }
}

macro_rules! gen_stats {
    ($($field:ident : $field_type:ty),* $(,)?) => { paste::item! {

        #[derive(Debug,Default,Clone,Copy)]
        struct StatsData {
            $($field : $field_type),*
        }

        impl Stats { $(

            /// Field getter.
            pub fn $field(&self) -> $field_type {
                self.rc.borrow().$field
            }

            /// Field setter.
            pub fn [<set _ $field>](&self, value:$field_type) {
                self.rc.borrow_mut().$field = value;
            }

            /// Field modifier.
            pub fn [<mod _ $field>]<F:FnOnce($field_type)->$field_type>(&self, f:F) {
                let value = self.$field();
                let value = f(value);
                self.[<set _ $field>](value);
            }

            /// Increments field's value.
            pub fn [<inc _ $field>](&self) {
                self.[<mod _ $field>](|t| t+1);
            }

            /// Decrements field's value.
            pub fn [<dec _ $field>](&self) {
                self.[<mod _ $field>](|t| t-1);
            }

        )* }
    }};
}

gen_stats!{
    gpu_memory_usage       : u32,
    draw_call_count        : usize,
    buffer_count           : usize,
    data_upload_count      : usize,
    data_upload_size       : u32,
    sprite_system_count    : usize,
    sprite_count           : usize,
    symbol_count           : usize,
    mesh_count             : usize,
    material_count         : usize,
    material_compile_count : usize,
}

impl StatsData {
    fn reset(&mut self) {
        self.draw_call_count        = 0;
        self.material_compile_count = 0;
        self.data_upload_count      = 0;
        self.data_upload_size       = 0;
    }
}