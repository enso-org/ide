//! This module contains functionality that allows ports and edges to be colored according
//! to their type information.

use crate::prelude::*;

use crate::graph_editor::SharedHashMap;
use crate::graph_editor::Type;

use ensogl::data::color;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;


/// Color that should be used if no type information is available.
pub const DEFAULT_TYPE_COLOR : color::Lcha = color::Lcha::new(0.5, 0.0, 0.0, 1.0);



// ================================
// === Type to Color Conversion ===
// ================================

fn type_to_hash(type_information:Type) -> u64 {
    let mut hasher = DefaultHasher::new();
    type_information.0.hash(&mut hasher);
    hasher.finish()
}

/// Return the color that corresponds to the given type. Can be used to color edges and ports.
pub fn color_for_type(type_information:Type) -> color::Lch {
    let hue =  (type_to_hash(type_information) % 360) as f32 / 360.0;
    color::Lch::new(0.5, 0.8, hue)
}



// ================
// === Type Map ===
// ================

/// `TypeMap` allows to keep track of the type and color of a `ast::Id`. It allows to store the
/// `ast::Id` -> `Type` mapping and infer the colour for the given `ast::Id` from that.
#[derive(Clone,CloneRef,Debug,Default,Shrinkwrap)]
pub struct TypeColorMap {
    data: SharedHashMap<ast::Id,Type>,
}

impl TypeColorMap {
    /// Return the colour for the `ast_id`. If no type information is available, returns `None`.
    pub fn type_color(&self, ast_id:ast::Id) -> Option<color::Lcha> {
        self.data.get_cloned(&ast_id).map(|type_information| {
            color_for_type(type_information).into()
        })
    }
}
