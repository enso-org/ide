//! This module implements GPU-based texture support. Proper texture handling is a complex topic.
//! Follow the link to learn more about many assumptions this module was built upon:
//! https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D

use crate::prelude::*;

use crate::display::render::webgl::Context;
use nalgebra::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::HtmlImageElement;
use web_sys::WebGlTexture;


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
            $($field:ident),* $(,)?
        }
    ) => {
        $(#[$($meta)*])*
        #[allow(missing_docs)]
        pub enum $name { $($field),* }

        impl IsGlEnum for $name {
            fn to_gl_enum(&self) -> u32 {
                match self {
                    $(Self::$field => $field.to_gl_enum()),*
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

macro_rules! gen_unsupported_types {
    ( $($name:ident),* $(,)? ) => {$(
        #[derive(Copy,Clone,Debug)]
        pub struct $name {}
    )*}
}

/// Types which are used in WebGL but are not (yet) bound to Rust types.
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
pub mod unsupported_types {
    use super::*;
    gen_unsupported_types!
        { f16, f32_u24_u8_REV, u16_4_4_4_4, u16_5_5_5_1, u16_5_6_5, u32_f10_f11_f11_REV, u32_24_8
        , u32_2_10_10_10_REV, u32_5_9_9_9_REV
        }
}
pub use unsupported_types::*;



// ==============
// === GlType ===
// ==============

/// Class of GL primitive types, including bytes, shorts, ints, etc.
pub trait PrimType: Copy + 'static {
    fn gl_type() -> u32;
}

macro_rules! gen_prim_type_instances {
    ( $($name:ident = $expr:expr),* $(,)? ) => {$(
        impl PrimType for $name {
            fn gl_type() -> u32 {
                $expr
            }
        }
    )*}
}

gen_prim_type_instances! {
    u8                  = Context::UNSIGNED_BYTE,
    u16                 = Context::UNSIGNED_SHORT,
    u32                 = Context::UNSIGNED_INT,
    i8                  = Context::BYTE,
    i16                 = Context::SHORT,
    i32                 = Context::INT,
    f16                 = Context::HALF_FLOAT,
    f32                 = Context::FLOAT,
    f32_u24_u8_REV      = Context::FLOAT_32_UNSIGNED_INT_24_8_REV,
    u16_4_4_4_4         = Context::UNSIGNED_SHORT_4_4_4_4,
    u16_5_5_5_1         = Context::UNSIGNED_SHORT_5_5_5_1,
    u16_5_6_5           = Context::UNSIGNED_SHORT_5_6_5,
    u32_f10_f11_f11_REV = Context::UNSIGNED_INT_10F_11F_11F_REV,
    u32_24_8            = Context::UNSIGNED_INT_24_8,
    u32_2_10_10_10_REV  = Context::UNSIGNED_INT_2_10_10_10_REV,
    u32_5_9_9_9_REV     = Context::UNSIGNED_INT_5_9_9_9_REV,
}



// ================
// === GL Types ===
// ================

macro_rules! gl_variants {
    ( $($name:ident = $expr:expr),* $(,)? ) => {$(
        #[allow(missing_docs)]
        pub struct $name;

        impl Default for $name {
            fn default() -> Self {
                Self
            }
        }

        impl IsGlEnum for $name {
            fn to_gl_enum(&self) -> u32 {
                $expr
            }
        }
    )*}
}

gl_variants! {
    Alpha             = Context::ALPHA,
    Depth24Stencil8   = Context::DEPTH24_STENCIL8,
    Depth32fStencil8  = Context::DEPTH32F_STENCIL8,
    DepthComponent    = Context::DEPTH_COMPONENT,
    DepthComponent16  = Context::DEPTH_COMPONENT16,
    DepthComponent24  = Context::DEPTH_COMPONENT24,
    DepthComponent32f = Context::DEPTH_COMPONENT32F,
    DepthStencil      = Context::DEPTH_STENCIL,
    Luminance         = Context::LUMINANCE,
    LuminanceAlpha    = Context::LUMINANCE_ALPHA,
    R11fG11fB10f      = Context::R11F_G11F_B10F,
    R16f              = Context::R16F,
    R16i              = Context::R16I,
    R16ui             = Context::R16UI,
    R32f              = Context::R32F,
    R32i              = Context::R32I,
    R32ui             = Context::R32UI,
    R8                = Context::R8,
    R8i               = Context::R8I,
    R8SNorm           = Context::R8_SNORM,
    R8ui              = Context::R8UI,
    Red               = Context::RED,
    RedInteger        = Context::RED_INTEGER,
    Rg                = Context::RG,
    Rg16f             = Context::RG16F,
    Rg16i             = Context::RG16I,
    Rg16ui            = Context::RG16UI,
    Rg32f             = Context::RG32F,
    Rg32i             = Context::RG32I,
    Rg32ui            = Context::RG32UI,
    Rg8               = Context::RG8,
    Rg8i              = Context::RG8I,
    Rg8SNorm          = Context::RG8_SNORM,
    Rg8ui             = Context::RG8UI,
    Rgb               = Context::RGB,
    Rgb10A2           = Context::RGB10_A2,
    Rgb10A2ui         = Context::RGB10_A2UI,
    Rgb16f            = Context::RGB16F,
    Rgb16i            = Context::RGB16I,
    Rgb16ui           = Context::RGB16UI,
    Rgb32f            = Context::RGB32F,
    Rgb32i            = Context::RGB32I,
    Rgb32ui           = Context::RGB32UI,
    Rgb565            = Context::RGB565,
    Rgb5A1            = Context::RGB5_A1,
    Rgb8              = Context::RGB8,
    Rgb8i             = Context::RGB8I,
    Rgb8SNorm         = Context::RGB8_SNORM,
    Rgb8ui            = Context::RGB8UI,
    Rgb9E5            = Context::RGB9_E5,
    Rgba              = Context::RGBA,
    Rgba16f           = Context::RGBA16F,
    Rgba16i           = Context::RGBA16I,
    Rgba16ui          = Context::RGBA16UI,
    Rgba32f           = Context::RGBA32F,
    Rgba32i           = Context::RGBA32I,
    Rgba32ui          = Context::RGBA32UI,
    Rgba4             = Context::RGBA4,
    Rgba8             = Context::RGBA8,
    Rgba8i            = Context::RGBA8I,
    Rgba8SNorm        = Context::RGBA8_SNORM,
    Rgba8ui           = Context::RGBA8UI,
    RgbaInteger       = Context::RGBA_INTEGER,
    RgbInteger        = Context::RGB_INTEGER,
    RgInteger         = Context::RG_INTEGER,
    SRgb8             = Context::SRGB8,
    SRgb8Alpha8       = Context::SRGB8_ALPHA8,
}



// =================
// === AnyFormat ===
// =================

/// Texture formats. A `GlEnum` specifying the format of the texel data. Follow the link to learn
/// more: https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
pub mod format {
    use super::*;
    gl_enum! {
        AnyFormat
            { Alpha, DepthComponent, DepthStencil, Luminance, LuminanceAlpha, Red, RedInteger, Rg
            , Rgb, Rgba, RgbaInteger, RgbInteger, RgInteger,
            }
    }
}
pub use format::*;




// =========================
// === AnyInternalFormat ===
// =========================

/// A GLenum specifying the color components in the texture. Follow the link to learn more:
/// https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
pub mod internal_format {
    use super::*;
    gl_enum! {
        AnyInternalFormat
            { Alpha, Luminance, LuminanceAlpha, Rgb, Rgba, R8, R8SNorm, R16f, R32f, R8ui, R8i
            , R16ui, R16i, R32ui, R32i, Rg8, Rg8SNorm, Rg16f, Rg32f, Rg8ui, Rg8i, Rg16ui, Rg16i
            , Rg32ui, Rg32i, Rgb8, SRgb8, Rgb565, Rgb8SNorm, R11fG11fB10f, Rgb9E5, Rgb16f, Rgb32f
            , Rgb8ui, Rgb8i, Rgb16ui, Rgb16i, Rgb32ui, Rgb32i, Rgba8, SRgb8Alpha8, Rgba8SNorm
            , Rgb5A1, Rgba4, Rgb10A2, Rgba16f, Rgba32f, Rgba8ui, Rgba8i, Rgb10A2ui, Rgba16ui
            , Rgba16i, Rgba32i, Rgba32ui, DepthComponent16, DepthComponent24, DepthComponent32f
            , Depth24Stencil8, Depth32fStencil8
            }
    }
}
pub use internal_format::*;



// ==============
// === Format ===
// ==============

pub trait Format = Default + Into<AnyFormat>;



// ======================
// === InternalFormat ===
// ======================

/// Provides information about the size of a texture element for a given `InternalFormat`.
pub trait TextureElement<Type> {
    /// The size in bytes of a single element of the texture.
    type ByteSize: DimName;
}

/// Provides information about the suitable format and checks if the texture is color renderable
/// and filterable for a given `InternalFormat`.
pub trait InternalFormat : Default + Into<AnyInternalFormat> +'static {
    /// The `Format` associated with this `InternalFormat`. Please note that `InternalFormat`
    /// dictates which `Format` to use, but this relation is asymmetrical.
    type Format: Format;

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


/// Generates `TextureElement` and `InternalFormat` instances. Please note that the relation
/// between internal format, format, and possible client texel types is very strict and you are
/// not allowed to choose them arbitrary. Follow the link to learn more about possible relations and
/// how the values were composed below:
/// https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
#[macro_export]
macro_rules! generate_internal_format_instances {
    ([] $( $internal_format:ident $format:ident $color_renderable:tt $filterable:tt $elem_descs:tt
    )*) => {
        $(
            $crate::generate_internal_format_instances_item!
            { $internal_format $format $color_renderable $filterable $elem_descs }
        )*
    }
}

#[macro_export]
macro_rules! generate_internal_format_instances_item {
    ( $internal_format:ident $format:ident $color_renderable:tt $filterable:tt
      [$($possible_types:ident : $bytes_per_element:ident),*]
    ) => {
        $(impl TextureElement<$possible_types> for $internal_format {
            type ByteSize = $bytes_per_element;
        })*

        impl InternalFormat for $internal_format {
            type Format          = $format;
            type ColorRenderable = $color_renderable;
            type Filterable      = $filterable;
        }
    }
}

#[macro_export]
macro_rules! with_format_relations { ($f:ident $args:tt) => { $crate::$f! { $args
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
    R11fG11fB10f      Rgb            False True  [f32:U12,f16:U6,u32_f10_f11_f11_REV:U4]
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
    Depth32fStencil8  DepthStencil   True  False [f32_u24_u8_REV:U4]
}}}

with_format_relations!(generate_internal_format_instances []);



// =====================
// === TextureSource ===
// =====================

/// Source of the texture. Please note that the texture will be loaded asynchronously on demand.
#[derive(Clone,Debug)]
pub enum TextureSource {
    /// URL the texture should be loaded from. This source implies asynchronous loading.
    Url(String)
}

impl<S:Str> From<S> for TextureSource {
    fn from(s:S) -> Self {
        Self::Url(s.into())
    }
}



// ===============
// === Texture ===
// ===============

/// Texture representation.
#[derive(Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
pub struct Texture<InternalFormat,ElemType> {
    source  : TextureSource,
    phantom : PhantomData2<InternalFormat,ElemType>,
}



impl<I:InternalFormat,T:PrimType> Texture<I,T> {
    /// Constructor.
    pub fn new<S:Into<TextureSource>>(source:S) -> Self {
        let source  = source.into();
        let phantom = PhantomData;
        Self {source,phantom}
    }

    pub fn internal_format(&self) -> AnyInternalFormat {
        <I>::default().into()
    }

    pub fn format(&self) -> AnyFormat {
        <I::Format>::default().into()
    }

    pub fn gl_internal_format(&self) -> i32 {
        self.internal_format().to_gl_enum() as i32
    }

    pub fn gl_format(&self) -> u32 {
        self.format().to_gl_enum()
    }

    pub fn gl_elem_type(&self) -> u32 {
        <T>::gl_type()
    }
}

impl Texture<Rgba,u8> {
    pub fn Rgba<S:Into<TextureSource>>(source:S) -> Self {
        let source  = source.into();
        let phantom = PhantomData;
        Self {source,phantom}
    }
}



#[derive(Debug)]
pub struct BoundData<T> {
    texture    : T,
    gl_texture : WebGlTexture,
    context    : Context,
}

impl<T> BoundData<T> {
    /// Constructor.
    pub fn new(texture:T,context:&Context) -> Self {
        let gl_texture = context.create_texture().unwrap();
        let context    = context.clone();
        Self {texture,gl_texture,context}
    }
}


#[derive(Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Bound<T> {
    rc: Rc<RefCell<BoundData<T>>>
}

impl<F:InternalFormat,T:PrimType> Bound<Texture<F,T>> {
    /// Constructor.
    pub fn new(texture:Texture<F,T>, context:&Context) -> Self {
        let data = BoundData::new(texture,context);
        let rc   = Rc::new(RefCell::new(data));
        let out  = Self {rc};
        out.init_mock();
        out.reload();
        out
    }

    pub fn init_mock(&self) {
        let data            = self.rc.borrow();
        let texture         = &data.texture;
        let target          = Context::TEXTURE_2D;
        let level           = 0;
        let internal_format = texture.gl_internal_format();
        let format          = texture.gl_format();
        let elem_type       = texture.gl_elem_type();
        let width           = 1;
        let height          = 1;
        let border          = 0;
        let color           = vec![0,0,255,255];
        data.context.bind_texture(Context::TEXTURE_2D,Some(&data.gl_texture));
        data.context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
            (target,level,internal_format,width,height,border,format,elem_type,Some(&color));
    }

    pub fn reload(&self) {
        let data = self.rc.borrow();
        match &data.texture.source {
            TextureSource::Url(url) => {
                let image             = HtmlImageElement::new().unwrap();
                let no_callback       = <Option<Closure<dyn FnMut()>>>::None;
                let callback_ref      = Rc::new(RefCell::new(no_callback));
                let image_ref         = Rc::new(RefCell::new(image));
                let mut this          = self.clone();
                let mut callback_ref2 = callback_ref.clone();
                let mut image_ref_opt = image_ref.clone();
                let callback: Closure<dyn FnMut()> = Closure::once(move || {
                    let keep_alive      = callback_ref2;
                    let data            = this.rc.borrow();
                    let texture         = &data.texture;
                    let image           = image_ref_opt.borrow();
                    let target          = Context::TEXTURE_2D;
                    let level           = 0;
                    let internal_format = texture.gl_internal_format();
                    let format          = texture.gl_format();
                    let elem_type       = texture.gl_elem_type();
                    data.context.bind_texture(target,Some(&data.gl_texture));
                    data.context.tex_image_2d_with_u32_and_u32_and_html_image_element
                        (target,level,internal_format,format,elem_type,&image);
                });
                let image = image_ref.borrow();
                image.set_src(url);
                image.add_event_listener_with_callback("load",callback.as_ref().unchecked_ref());
                *callback_ref.borrow_mut() = Some(callback);
            }
        }

    }
}





// ==================
// === AnyTexture ===
// ==================

#[macro_export]
macro_rules! cartesians {
    ($f:ident [$($out:tt)*]) => {
        $f! { $($out)* }
    };
    ($f:ident $out:tt [$a:tt []] $($in:tt)*) => {
        $crate::cartesians! {$f $out $($in)*}
    };
    ($f:ident [$($out:tt)*] [$a:tt [$b:tt $($bs:tt)*]] $($in:tt)*) => {
        $crate::cartesians! {$f [$($out)* [$a $b]] [$a [$($bs)*]]  $($in)* }
    };
}

#[macro_export]
macro_rules! with_all_texture_types_impl {
    ( [$f:ident]
     $( $internal_format:ident $format:ident $color_renderable:tt $filterable:tt
        [$($possible_types:ident : $bytes_per_element:ident),*]
    )*) => {
        $crate::cartesians! { $f [] $([$internal_format [$($possible_types)*]])* }
    }
}

#[macro_export]
macro_rules! with_all_texture_types {
    ($f:ident) => {
        $crate::with_format_relations! { with_all_texture_types_impl [$f] }
    }
}


macro_rules! generate_any_texture {
    ( $([$internal_format:tt $type:tt])* ) => { paste::item! {
        /// Wrapper for any valid texture type.
        #[allow(non_camel_case_types)]
        #[allow(missing_docs)]
        #[derive(Clone,Debug)]
        pub enum AnyTexture {
            $([< $internal_format _ $type >](Texture<$internal_format,$type>)),*
        }
        $(impl From<Texture<$internal_format,$type>> for AnyTexture {
            fn from(t:Texture<$internal_format,$type>) -> Self {
                Self::[< $internal_format _ $type >](t)
            }
        })*
    }}
}

with_all_texture_types!(generate_any_texture);

//use crate::system::gpu::data::class::ContextUniformOps;
//use web_sys::WebGlUniformLocation;

//impl ContextUniformOps<AnyTexture> for Context {
//    fn set_uniform(&self, location:&WebGlUniformLocation, value:&AnyTexture){
//        todo!()
//    }
//}
