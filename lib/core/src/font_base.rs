use crate::prelude::*;
use basegl_core_msdf_sys as msdf_sys;
use basegl_core_fonts_base as font_base;

pub struct Font {
    msdf_sys_handle : msdf_sys::Font,
    pub name        : String,
    chars           : HashMap<u32, CharInfo>
}

pub struct CharInfo {
    msdf : msdf_sys::MutlichannelSignedDistanceField,
}
