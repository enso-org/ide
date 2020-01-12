//! This module implements GPU-based texture support. Proper texture handling is a complex topic.
//! Follow the link to learn more about many assumptions this module was built upon:
//! https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D

//use crate::prelude::*;

use crate::display::render::webgl::Context;
use nalgebra::*;


// =============
// === Value ===
// =============

/// Defines relation between types and values, like between `True` and `true`.
pub trait Value {
    /// The value-level counterpart of this type-value.
    type Type;

    /// The value of this type-value.
    fn value() -> Self::Type;
}



// =======================
// === Type-level Bool ===
// =======================

/// Type level `true` value.
pub struct True {}

/// Type level `false` value.
pub struct False {}

impl Value for True {
    type Type = bool;
    fn value() -> Self::Type {
        true
    }
}

impl Value for False {
    type Type = bool;
    fn value() -> Self::Type {
        false
    }
}



// ==============
// === GlEnum ===
// ==============

/// Converts Rust types to `GlEnum` values.
pub trait IsGlEnum {
    /// `GlEnum` value of this type.
    fn to_gl_enum(&self) -> u32;
}

macro_rules! gl_enum {
    (
        $(#[$($meta:tt)*])*
        $name:ident {
            $($field:ident = $expr:expr),* $(,)?
        }
    ) => {
        $(#[$($meta)*])*
        #[allow(missing_docs)]
        pub enum $name { $($field),* }

        $(#[allow(missing_docs)] pub struct $field {})*

        impl IsGlEnum for $name {
            fn to_gl_enum(&self) -> u32 {
                match self {
                    $(Self::$field => $expr),*
                }
            }
        }

        $(impl From<$field> for $name {
            fn from(_:$field) -> Self {
                Self::$field
            }
        })*
    }
}



// =========================
// === Unsupported Types ===
// =========================

/// Types which are used in WebGL but are not (yet) bound to Rust types.
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
pub mod unsupported_types {
    pub struct f16 {}
    pub struct f32_32_u32_24_8_REV {}
    pub struct u16_4_4_4_4 {}
    pub struct u16_5_5_5_1 {}
    pub struct u16_5_6_5 {}
    pub struct u32_10f_11f_11f_REV {}
    pub struct u32_24_8 {}
    pub struct u32_2_10_10_10_REV {}
    pub struct u32_5_9_9_9_REV {}
}
use unsupported_types::*;



// ==============
// === Format ===
// ==============

/// Texture formats. A `GlEnum` specifying the format of the texel data. Follow the link to learn
/// more: https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
pub mod format {
    use super::*;
    gl_enum! {
        Format {
            Alpha          = Context::ALPHA,
            DepthComponent = Context::DEPTH_COMPONENT,
            DepthStencil   = Context::DEPTH_STENCIL,
            Luminance      = Context::LUMINANCE,
            LuminanceAlpha = Context::LUMINANCE_ALPHA,
            Red            = Context::RED,
            RedInteger     = Context::RED_INTEGER,
            Rg             = Context::RG,
            Rgb            = Context::RGB,
            Rgba           = Context::RGBA,
            RgbaInteger    = Context::RGBA_INTEGER,
            RgbInteger     = Context::RGB_INTEGER,
            RgInteger      = Context::RG_INTEGER,
        }
    }
}
use format::*;



// ======================
// === InternalFormat ===
// ======================

/// A GLenum specifying the color components in the texture. Follow the link to learn more:
/// https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
pub mod internal_format {
    use super::*;
    gl_enum! {
        Format {
        Luminance         = Context::LUMINANCE,
        LuminanceAlpha    = Context::LUMINANCE_ALPHA,
        Rgb               = Context::RGB,
        Rgba              = Context::RGBA,
        R8                = Context::R8,
        R8SNorm           = Context::R8_SNORM,
        R16f              = Context::R16F,
        R32f              = Context::R32F,
        R8ui              = Context::R8UI,
        R8i               = Context::R8I,
        R16ui             = Context::R16UI,
        R16i              = Context::R16I,
        R32ui             = Context::R32UI,
        R32i              = Context::R32I,
        Rg8               = Context::RG8,
        Rg8SNorm          = Context::RG8_SNORM,
        Rg16f             = Context::RG16F,
        Rg32f             = Context::RG32F,
        Rg8ui             = Context::RG8UI,
        Rg8i              = Context::RG8I,
        Rg16ui            = Context::RG16UI,
        Rg16i             = Context::RG16I,
        Rg32ui            = Context::RG32UI,
        Rg32i             = Context::RG32I,
        Rgb8              = Context::RGB8,
        SRgb8             = Context::SRGB8,
        Rgb565            = Context::RGB565,
        Rgb8SNorm         = Context::RGB8_SNORM,
        R11fG11fB10f      = Context::R11F_G11F_B10F,
        Rgb9E5            = Context::RGB9_E5,
        Rgb16f            = Context::RGB16F,
        Rgb32f            = Context::RGB32F,
        Rgb8ui            = Context::RGB8UI,
        Rgb8i             = Context::RGB8I,
        Rgb16ui           = Context::RGB16UI,
        Rgb16i            = Context::RGB16I,
        Rgb32ui           = Context::RGB32UI,
        Rgb32i            = Context::RGB32I,
        Rgba8             = Context::RGBA8,
        SRgb8Alpha8       = Context::SRGB8_ALPHA8,
        Rgba8SNorm        = Context::RGBA8_SNORM,
        Rgb5A1            = Context::RGB5_A1,
        Rgba4             = Context::RGBA4,
        Rgb10A2           = Context::RGB10_A2,
        Rgba16f           = Context::RGBA16F,
        Rgba32f           = Context::RGBA32F,
        Rgba8ui           = Context::RGBA8UI,
        Rgba8i            = Context::RGBA8I,
        Rgb10A2ui         = Context::RGB10_A2UI,
        Rgba16ui          = Context::RGBA16UI,
        Rgba16i           = Context::RGBA16I,
        Rgba32i           = Context::RGBA32I,
        Rgba32ui          = Context::RGBA32UI,
        DepthComponent16  = Context::DEPTH_COMPONENT16,
        DepthComponent24  = Context::DEPTH_COMPONENT24,
        DepthComponent32f = Context::DEPTH_COMPONENT32F,
        Depth24Stencil8   = Context::DEPTH24_STENCIL8,
        Depth32fStencil8  = Context::DEPTH32F_STENCIL8,
        }
    }
}
use internal_format::*;



// ==========================
// === InternalFormatInfo ===
// ==========================

/// Provides information about the size of a texture element for a given `InternalFormat`.
pub trait TextureElement<Type> {
    /// The size in bytes of a single element of the texture.
    type ByteSize: DimName;
}

/// Provides information about the suitable format and checks if the texture is color renderable
/// and filterable for a given `InternalFormat`.
pub trait InternalFormatInfo {
    /// The `Format` associated with this `InternalFormat`. Please note that `InternalFormat`
    /// dictates which `Format` to use, but this relation is asymmetrical.
    type Format;

    /// Checks if the texture format can be rendered as color.
    type ColorRenderable: Value<Type=bool>;

    /// Checks it he texture can be filtered.
    type Filterable: Value<Type=bool>;

    /// Checks if the texture format can be rendered as color.
    fn color_renderable() -> bool {
        <Self::ColorRenderable as Value>::value()
    }

    /// Checks it he texture can be filtered.
    fn filterable() -> bool {
        <Self::Filterable as Value>::value()
    }
}


/// Generates `TextureElement` and `InternalFormatInfo` instances. Please note that the relation
/// between internal format, format, and possible client texel types is very strict and you are
/// not allowed to choose them arbitrary. Follow the link to learn more about possible relations and
/// how the values were composed below:
/// https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
macro_rules! generate_texture_internal_format_info {
    ($( $internal_format:ident $format:ident $color_renderable:tt $filterable:tt $elem_descs:tt
    )*) => {
        $(
            generate_texture_internal_format_info_item!
            { $internal_format $format $color_renderable $filterable $elem_descs }
        )*
    }
}

macro_rules! generate_texture_internal_format_info_item {
    ( $internal_format:ident $format:ident $color_renderable:tt $filterable:tt
      [$($possible_types:ident : $bytes_per_element:ident),*]
    ) => {
        $(impl TextureElement<$possible_types> for $internal_format {
            type ByteSize = $bytes_per_element;
        })*

        impl InternalFormatInfo for $internal_format {
            type Format          = $format;
            type ColorRenderable = $color_renderable;
            type Filterable      = $filterable;
        }
    }
}

generate_texture_internal_format_info! {
//  INTERNAL_FORMAT   FORMAT         COL   FILT  [POSSIBLE_TYPE:BYTES_PER_TEXTURE_ELEM]
    Alpha             Alpha          True  True  [u8:U1,f16:U2,f32:U4]
    Luminance         Luminance      True  True  [u8:U1,f16:U2,f32:U4]
    LuminanceAlpha    LuminanceAlpha True  True  [u8:U2,f16:U4,f32:U8]
    Rgb               Rgb            True  True  [u8:U3,f16:U6,f32:U12,u16_5_6_5:U2]
    Rgba              Rgba           True  True  [u8:U4,f16:U8,f32:U16,u16_4_4_4_4:U2,u16_5_5_5_1:U2]
    R8                Red            True  True  [u8:U1]
    R8SNorm           Red            False True  [i8:U1]
    R16f              Red            False True  [f32:U4,f16:U2]
    R32f              Red            False False [f32:U4]
    R8ui              RedInteger     True  False [u8:U1]
    R8i               RedInteger     True  False [i8:U1]
    R16ui             RedInteger     True  False [u16:U2]
    R16i              RedInteger     True  False [i16:U2]
    R32ui             RedInteger     True  False [u32:U4]
    R32i              RedInteger     True  False [i32:U4]
    Rg8               Rg             True  True  [u8:U2]
    Rg8SNorm          Rg             False True  [i8:U2]
    Rg16f             Rg             False True  [f32:U8,f16:U4]
    Rg32f             Rg             False False [f32:U8]
    Rg8ui             RgInteger      True  False [u8:U2]
    Rg8i              RgInteger      True  False [i8:U2]
    Rg16ui            RgInteger      True  False [u16:U4]
    Rg16i             RgInteger      True  False [i16:U4]
    Rg32ui            RgInteger      True  False [u32:U8]
    Rg32i             RgInteger      True  False [i32:U8]
    Rgb8              Rgb            True  True  [u8:U3]
    SRgb8             Rgb            False True  [u8:U3]
    Rgb565            Rgb            True  True  [u8:U3,u16_5_6_5:U2]
    Rgb8SNorm         Rgb            False True  [i8:U3]
    R11fG11fB10f      Rgb            False True  [f32:U12,f16:U6,u32_10f_11f_11f_REV:U4]
    Rgb9E5            Rgb            False True  [f32:U12,f16:U6,u32_5_9_9_9_REV:U4]
    Rgb16f            Rgb            False True  [f32:U12,f16:U6]
    Rgb32f            Rgb            False False [f32:U12]
    Rgb8ui            RgbInteger     False False [u8:U3]
    Rgb8i             RgbInteger     False False [i8:U3]
    Rgb16ui           RgbInteger     False False [u16:U6]
    Rgb16i            RgbInteger     False False [i16:U6]
    Rgb32ui           RgbInteger     False False [u32:U12]
    Rgb32i            RgbInteger     False False [i32:U12]
    Rgba8             Rgba           True  True  [u8:U4]
    SRgb8Alpha8       Rgba           True  True  [u8:U4]
    Rgba8SNorm        Rgba           False True  [i8:U4]
    Rgb5A1            Rgba           True  True  [u8:U4,u16_5_5_5_1:U2,u32_2_10_10_10_REV:U4]
    Rgba4             Rgba           True  True  [u8:U4,u16_4_4_4_4:U2]
    Rgb10A2           Rgba           True  True  [u32_2_10_10_10_REV:U4]
    Rgba16f           Rgba           False True  [f32:U16,f16:U8]
    Rgba32f           Rgba           False False [f32:U16]
    Rgba8ui           RgbaInteger    True  False [u8:U4]
    Rgba8i            RgbaInteger    True  False [i8:U4]
    Rgb10A2ui         RgbaInteger    True  False [u32_2_10_10_10_REV:U4]
    Rgba16ui          RgbaInteger    True  False [u16:U8]
    Rgba16i           RgbaInteger    True  False [i16:U8]
    Rgba32i           RgbaInteger    True  False [i32:U16]
    Rgba32ui          RgbaInteger    True  False [u32:U16]
    DepthComponent16  DepthComponent True  False [u16:U2,u32:U4]
    DepthComponent24  DepthComponent True  False [u32:U4]
    DepthComponent32f DepthComponent True  False [f32:U4]
    Depth24Stencil8   DepthStencil   True  False [u32_24_8:U4]
    Depth32fStencil8  DepthStencil   True  False [f32_32_u32_24_8_REV:U4]
}