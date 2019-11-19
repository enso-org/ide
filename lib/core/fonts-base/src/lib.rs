use basegl_prelude::*;

pub struct FontsBase {
    pub fonts_by_name : HashMap<&'static str, &'static [u8]>
}

impl FontsBase {
    pub fn new() -> FontsBase {
        let mut fonts_by_name : HashMap<&'static str, &'static [u8]>
            = HashMap::new();
        include!(concat!(env!("OUT_DIR"), "/fill_map.rs"));
        FontsBase {
            fonts_by_name
        }
    }
}