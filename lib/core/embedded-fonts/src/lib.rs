use basegl_prelude::*;

/// A base of built-in fonts in application
///
/// The structure keeps only a binary data in ttf format. The data should be
/// then interpreted by user (e.g. by using msdf-sys crate)
///
/// For list of embedded fonts, see FONTS_TO_EXTRACT constant in `build.rs`
pub struct EmbeddedFonts {
    pub font_data_by_name: HashMap<&'static str, &'static [u8]>
}

impl EmbeddedFonts {
    /// Creates an embedded fonts base filled with data
    ///
    /// For list of embedded fonts, see `FONTS_TO_EXTRACT` constant in
    /// `build.rs`
    pub fn create_and_fill() -> EmbeddedFonts {
        let mut fonts_by_name : HashMap<&'static str, &'static [u8]>
            = HashMap::new();
        include!(concat!(env!("OUT_DIR"), "/fill_map.rs"));
        EmbeddedFonts {
            font_data_by_name: fonts_by_name
        }
    }
}