//! This module implements GPU-based texture support. Proper texture handling is a complex topic.
//! Follow the link to learn more about many assumptions this module was built upon:
//! https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D

use crate::prelude::*;

use crate::display::symbol::material::VarDecl;
use crate::system::gpu::data::buffer::item::JsBufferViewArr;
use crate::system::gpu::data::uniform::IntoUniformValueImpl;
use crate::system::gpu::types::*;
use crate::system::gpu::types::glsl::PrimType;
use crate::system::web;

use nalgebra::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::HtmlImageElement;
use web_sys::Url;
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



// ================
// === GL Types ===
// ================

crate::define_singletons_gl! {
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



// ==============
// === Format ===
// ==============

/// Trait for every format of a texture.
pub trait Format = Default + Into<AnyFormat>;



// =================
// === AnyFormat ===
// =================

/// Texture formats. A `GlEnum` specifying the format of the texel data. Follow the link to learn
/// more: https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
pub mod format {
    use super::*;
    crate::define_singleton_enum_gl_from! {
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
    crate::define_singleton_enum_gl_from! {
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

/// See docs of `generate_internal_format_instances`.
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

/// Runs the provided macro with all texture format relations. In order to learn more about the
/// possible relations, refer to the source code and to the guide:
/// https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
#[macro_export]
macro_rules! with_texture_format_relations { ($f:ident $args:tt) => { $crate::$f! { $args
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

with_texture_format_relations!(generate_internal_format_instances []);



// =======================
// === Texture sources ===
// =======================

// === Empty Texture ===

/// An empty texture, created as 1x1 texture in Rgba format.
pub struct EmptyTexture;



// === Texture from url ===

/// Texture loaded from given url. It is loaded to gpu asynchronously, putting a mocked texture
/// before
pub struct TextureFromUrl {
    /// An url from where the texture is loaded.
    pub url       : String,
}


impl<S:Str> From<S> for TextureFromUrl {
    fn from(s:S) -> Self {
        Self{url:s.into()}
    }
}


// === Texture from memory ===

/// A texture created from slice.
pub struct TextureFromMemory<'a,T> {
    /// A slice from which the texture data are taken.
    pub view   : &'a[T],
    /// Texture's width
    pub width  : i32,
    /// Texture's height
    pub height : i32,
}



// ===============
// === Texture ===
// ===============

/// A Texture.
pub struct Texture<Source,InternalFormat,ElemType> {
    /// The source of texture data
    pub source : Source,
    /// A format and element type of this texture
    pub phantom : PhantomData2<InternalFormat,ElemType>,
}

/// Bounds for every texture item type.
pub trait TextureItemType = PhantomInto<GlEnum> + 'static;

impl<S,I,T> Texture<S,I,T> {
    /// Create a new texture from given source.
    pub fn new(source:S) -> Self {
        let phantom = PhantomData;
        Texture{source,phantom}
    }
}

impl<S, I:InternalFormat, T:TextureItemType> Texture<S,I,T> {
    /// Internal format instance of this texture. Please note, that this value could be computed
    /// without taking self reference, however it was defined in such way for convenient usage.
    pub fn internal_format(&self) -> AnyInternalFormat {
        <I>::default().into()
    }

    /// Format instance of this texture. Please note, that this value could be computed
    /// without taking self reference, however it was defined in such way for convenient usage.
    pub fn format(&self) -> AnyFormat {
        <I::Format>::default().into()
    }

    /// Internal format of this texture as `GlEnum`. Please note, that this value could be computed
    /// without taking self reference, however it was defined in such way for convenient usage.
    pub fn gl_internal_format(&self) -> i32 {
        let GlEnum(u) = self.internal_format().into_gl_enum();
        u as i32
    }

    /// Format of this texture as `GlEnum`. Please note, that this value could be computed
    /// without taking self reference, however it was defined in such way for convenient usage.
    pub fn gl_format(&self) -> GlEnum {
        self.format().into_gl_enum()
    }

    /// Element type of this texture as `GlEnum`. Please note, that this value could be computed
    /// without taking self reference, however it was defined in such way for convenient usage.
    pub fn gl_elem_type(&self) -> u32 {
        <T>::gl_enum().into()
    }
}

impl<Source> Texture<Source,Rgba,u8> {
    /// Constructor for rgba u8 texture.
    pub fn rgba(source:Source) -> Self {
        Self::new(source)
    }
}

impl<Source> Texture<Source,Rgb,u8> {
    /// Constructor for rgb u8 texture.
    pub fn rgb(source:Source) -> Self {
        Self::new(source)
    }
}

impl<Source,I,T> From<Source> for Texture<Source,I,T> {
    fn from(source:Source) -> Self {
        let phantom = PhantomData;
        Texture{source,phantom}
    }
}

impl<S,I,T> From<Texture<S,I,T>> for VarDecl {
    fn from(_:Texture<S,I,T>) -> Self {
        VarDecl::new(PrimType::Sampler2d, None)
    }
}

impl<I,T> GpuDefault for Texture<EmptyTexture,I,T> {
    fn gpu_default() -> Self {
        Self::new(EmptyTexture)
    }
}

// TODO[ao]: The both IntoUniformValueImpl implementations should be generated only for valid I,T
// combinations (see `with_texture_format_relations` macro above). Then possibly we can remove
// I,T parameters from BoundTexture.

impl<I,T> IntoUniformValueImpl for Texture<EmptyTexture,I,T>
where I : InternalFormat,
      T : TextureItemType {
    type Result = BoundTexture<I,T>;
    fn into_uniform_value(self, context:&Context) -> Self::Result {
        BoundTexture::new_empty(context)
    }
}

impl<I,T> IntoUniformValueImpl for Texture<TextureFromUrl,I,T>
where I : InternalFormat,
      T : TextureItemType {
    type Result = BoundTexture<I,T>;
    fn into_uniform_value(self, context:&Context) -> Self::Result {
        BoundTexture::new_from_url(&self,context)
    }
}

impl<'a,I,T> IntoUniformValueImpl for Texture<TextureFromMemory<'a,T>,I,T>
where I : InternalFormat,
      T : TextureItemType + JsBufferViewArr {
    type Result = BoundTexture<I,T>;
    fn into_uniform_value(self, context:&Context) -> Self::Result {
        BoundTexture::new_from_memory(&self,context)
    }
}



// ====================
// === BoundTexture ===
// ====================

/// Texture bound to GL context.
#[derive(Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct BoundTexture<I,T> {
    rc: Rc<RefCell<BoundTextureData<I,T>>>
}

/// Texture bound to GL context.
#[derive(Debug)]
pub struct BoundTextureData<I,T> {
    gl_texture : WebGlTexture,
    context    : Context,
    format     : PhantomData<I>,
    elem_type  : PhantomData<T>,
}

impl<I,T> BoundTextureData<I,T> {
    /// Constructor.
    pub fn new(context:&Context) -> Self {
        Self {
            gl_texture : context.create_texture().unwrap(),
            context    : context.clone(),
            format     : PhantomData,
            elem_type  : PhantomData,
        }
    }

    /// Initializes default texture value. It is useful when the texture data needs to be downloaded
    /// asynchronously. This method creates a mock 1px x 1px texture and uses it as a mock texture
    /// until the download is complete.
    pub fn init_mock(&self) {
        let target          = Context::TEXTURE_2D;
        let level           = 0;
        let internal_format = Context::RGBA as i32;
        let format          = Context::RGBA;
        let elem_type       = Context::UNSIGNED_BYTE;
        let width           = 1;
        let height          = 1;
        let border          = 0;
        let color           = vec![0,0,255,255];
        self.context.bind_texture(Context::TEXTURE_2D,Some(&self.gl_texture));
        self.context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
        (target,level,internal_format,width,height,border,format,elem_type,Some(&color)).unwrap();
    }
}

impl<I:InternalFormat,T:TextureItemType> BoundTexture<I,T> {
    /// Constructor for empty texture.
    pub fn new_empty(context:&Context) -> Self {
        let data = BoundTextureData::new(context);
        let rc   = Rc::new(RefCell::new(data));
        let out  = Self {rc};
        out.init_mock();
        out
    }

    /// Constructor from url.
    pub fn new_from_url(texture:&Texture<TextureFromUrl,I,T>, context:&Context) -> Self {
        let data = BoundTextureData::new(context);
        let rc   = Rc::new(RefCell::new(data));
        let out  = Self {rc};
        out.init_mock();
        out.reload_from_url(texture);
        out
    }

    /// Initializes default texture value. It is useful when the texture data needs to be downloaded
    /// asynchronously. This method creates a mock 1px x 1px texture and uses it as a mock texture
    /// until the download is complete.
    pub fn init_mock(&self) {
        self.rc.borrow().init_mock()
    }

    /// Loads or re-loads the texture data from the provided source. This action will be performed
    /// asynchronously.
    pub fn reload_from_url(&self, texture:&Texture<TextureFromUrl,I,T>) {
        let internal_format = texture.gl_internal_format();
        let format          = texture.gl_format().into();
        let elem_type       = texture.gl_elem_type();
        let target          = Context::TEXTURE_2D;
        let level           = 0;
        let image           = HtmlImageElement::new().unwrap();
        let no_callback     = <Option<Closure<dyn FnMut()>>>::None;
        let callback_ref    = Rc::new(RefCell::new(no_callback));
        let image_ref       = Rc::new(RefCell::new(image));
        let this            = self.clone();
        let callback_ref2   = callback_ref.clone();
        let image_ref_opt   = image_ref.clone();
        let callback: Closure<dyn FnMut()> = Closure::once(move || {
            let _keep_alive = callback_ref2;
            let data        = this.rc.borrow();
            let image       = image_ref_opt.borrow();
            data.context.bind_texture(target,Some(&data.gl_texture));
            data.context.tex_image_2d_with_u32_and_u32_and_html_image_element
            (target,level,internal_format,format,elem_type,&image).unwrap();
        });
        let js_callback = callback.as_ref().unchecked_ref();
        let image       = image_ref.borrow();
        request_cors_if_not_same_origin(&image,&texture.source.url);
        image.set_src(&texture.source.url);
        image.add_event_listener_with_callback("load",js_callback).unwrap();
        *callback_ref.borrow_mut() = Some(callback);
    }
}

impl<I:InternalFormat,T:TextureItemType + JsBufferViewArr> BoundTexture<I,T> {
    /// Constructs from memory view.
    pub fn new_from_memory(texture:&Texture<TextureFromMemory<'_,T>,I,T>, context:&Context) -> Self {
        let data = BoundTextureData::new(context);
        let rc   = Rc::new(RefCell::new(data));
        let out  = Self {rc};
        out.reload_from_memory(texture);
        out
    }

    /// Loads or re-loads the texture data from the provided source.
    pub fn reload_from_memory(&self, texture:&Texture<TextureFromMemory<'_,T>,I,T>) {
        let data            = &self.rc.borrow();
        let context         = &data.context;
        let internal_format = texture.gl_internal_format();
        let format          = texture.gl_format().into();
        let elem_type       = texture.gl_elem_type();
        let target          = Context::TEXTURE_2D;
        let level           = 0;
        let border          = 0;
        let width           = texture.source.width;
        let height          = texture.source.height;
        unsafe {
            // We use unsafe array view which is used immediately, so no allocations should happen
            // until we drop the view.
            let view        = texture.source.view.js_buffer_view();
            let result = context
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view
                (target,level,internal_format,width,height,border,format,elem_type,Some(&view));
            result.unwrap();
        }
    }
}


// === ContextTextureOps ===

/// Trait with webgl context operations on texture `Texture`. Implemented for `BoundTexture`, made
/// for making distinction in `Uniform` implementations.
pub trait ContextTextureOps<Texture> {
    /// A guard removing created binding at end of scope.
    type Guard;
    /// Bind texture for specific unit
    fn bind_texture_unit(&self, texture:&Texture, unit:u32) -> Self::Guard;
}

impl<I,T> ContextTextureOps<BoundTexture<I,T>> for Context {
    type Guard = TextureBindingGuard;

    fn bind_texture_unit(&self, texture:&BoundTexture<I,T>, unit:u32) -> Self::Guard {
        let context    = self.clone();
        let target     = Context::TEXTURE_2D;
        let gl_texture = &texture.rc.borrow().gl_texture;
        context.active_texture(unit);
        context.bind_texture(target,Some(gl_texture));
        context.active_texture(Context::TEXTURE0);
        TextureBindingGuard {context,target,unit}
    }
}

/// Guard which unbinds texture in specific texture unit on drop.
pub struct TextureBindingGuard {
    context : Context,
    target  : u32,
    unit    : u32,
}

impl Drop for TextureBindingGuard {
    fn drop(&mut self) {
        self.context.active_texture(self.unit);
        self.context.bind_texture(self.target,None);
        self.context.active_texture(Context::TEXTURE0);
    }
}


// === Utils ===

/// CORS = Cross Origin Resource Sharing. It's a way for the webpage to ask the image server for
/// permission to use the image. To do this we set the crossOrigin attribute to something and then
/// when the browser tries to get the image from the server, if it's not the same domain, the browser
/// will ask for CORS permission. The string we set `cross_origin` to is sent to the server.
/// The server can look at that string and decide whether or not to give you permission. Most
/// servers that support CORS don't look at the string, they just give permission to everyone.
///
/// **Note**
/// Why don't want to just always see the permission because asking for permission takes 2 HTTP
/// requests, so it's slower than not asking. If we know we're on the same domain or we know we
/// won't use the image for anything except img tags and or canvas2d then we don't want to set
/// crossDomain because it will make things slower.
fn request_cors_if_not_same_origin(img:&HtmlImageElement, url_str:&str) {
    let url    = Url::new(url_str).unwrap();
    let origin = web::window().location().origin().unwrap();
    if url.origin() != origin {
        img.set_cross_origin(Some(""));
    }
}



// ======================
// === Meta Iterators ===
// ======================

/// See docs of `with_all_texture_types`.
#[macro_export]
macro_rules! with_all_texture_types_cartesians {
    ($f:ident [$($out:tt)*]) => {
        $f! { $($out)* }
    };
    ($f:ident $out:tt [$a:tt []] $($in:tt)*) => {
        $crate::with_all_texture_types_cartesians! {$f $out $($in)*}
    };
    ($f:ident [$($out:tt)*] [$a:tt [$b:tt $($bs:tt)*]] $($in:tt)*) => {
        $crate::with_all_texture_types_cartesians! {$f [$($out)* [$a $b]] [$a [$($bs)*]]  $($in)* }
    };
}

/// See docs of `with_all_texture_types`.
#[macro_export]
macro_rules! with_all_texture_types_impl {
    ( [$f:ident]
     $( $internal_format:ident $format:ident $color_renderable:tt $filterable:tt
        [$($possible_types:ident : $bytes_per_element:ident),*]
    )*) => {
        $crate::with_all_texture_types_cartesians!
            { $f [] $([$internal_format [$($possible_types)*]])* }
    }
}

/// Runs the argument macro providing it with list of all possible texture types:
/// `arg! { [Alpha u8] [Alpha f16] [Alpha f32] [Luminance u8] ... }`
#[macro_export]
macro_rules! with_all_texture_types {
    ($f:ident) => {
        $crate::with_texture_format_relations! { with_all_texture_types_impl [$f] }
    }
}
