//! This module implements GPU-based texture support. Proper texture handling is a complex topic.
//! Follow the link to learn more about many assumptions this module was built upon:
//! https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D

pub mod types;
pub mod storage;
pub mod class;

use crate::prelude::*;

use crate::system::gpu::data::buffer::item::JsBufferViewArr;
use crate::system::gpu::types::*;
use crate::system::web;
use nalgebra::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::HtmlImageElement;
use web_sys::WebGlTexture;


pub use class::*;
pub use types::*;
pub use storage::*;



// ===================
// === WithContent ===
// ===================

pub trait WithContent {
    type Content;
    fn with_content<F:FnOnce(&Self::Content)->T,T>(&self, f:F) -> T;
}

impl<T:Deref> WithContent for T
    where <T as Deref>::Target: WithContent {
    type Content = <<T as Deref>::Target as WithContent>::Content;
    default fn with_content<F:FnOnce(&Self::Content)->R,R>(&self, f:F) -> R {
        self.deref().with_content(f)
    }
}



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






















// ===================
// === RemoteImage ===
// ===================

/// Texture downloaded from URL. This source implies asynchronous loading.
#[derive(Debug)]
pub struct RemoteImageData {
    /// An url from where the texture is downloaded.
    pub url : String,
}

impl RemoteImageData {
    fn new<S:Str>(url:S) -> Self {
        Self {url:url.into()}
    }
}

impl<S:Str> From<S> for RemoteImageData {
    fn from(s:S) -> Self {
        Self::new(s)
    }
}

impl<I,T> StorageRelation<I,T> for RemoteImage {
    type Storage = RemoteImageData;
}

impl<I:InternalFormat,T:Item>
Texture<RemoteImage,I,T> {
    /// Initializes default texture value. It is useful when the texture data needs to be downloaded
    /// asynchronously. This method creates a mock 1px x 1px texture and uses it as a mock texture
    /// until the download is complete.
    pub fn init_mock(&self) {
        let target          = Context::TEXTURE_2D;
        let level           = 0;
        let internal_format = Self::gl_internal_format();
        let format          = Self::gl_format().into();
        let elem_type       = Self::gl_elem_type();
        let width           = 1;
        let height          = 1;
        let border          = 0;
        let color           = vec![0,0,255,255];
        self.context().bind_texture(Context::TEXTURE_2D,Some(&self.gl_texture()));
        self.context().tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
        (target,level,internal_format,width,height,border,format,elem_type,Some(&color)).unwrap();
    }
}

impl<I:InternalFormat,T:Item>
TextureReload for Texture<RemoteImage,I,T> {
    /// Loads or re-loads the texture data from the provided url.
    /// This action will be performed asynchronously.
    fn reload(&self) {
        let url           = &self.storage().url;
        let image         = HtmlImageElement::new().unwrap();
        let no_callback   = <Option<Closure<dyn FnMut()>>>::None;
        let callback_ref  = Rc::new(RefCell::new(no_callback));
        let image_ref     = Rc::new(RefCell::new(image));
        let callback_ref2 = callback_ref.clone();
        let image_ref_opt = image_ref.clone();
        let context       = self.context().clone();
        let gl_texture    = self.gl_texture().clone();
        let callback: Closure<dyn FnMut()> = Closure::once(move || {
            let _keep_alive     = callback_ref2;
            let image           = image_ref_opt.borrow();
            let target          = Context::TEXTURE_2D;
            let level           = 0;
            let internal_format = Self::gl_internal_format();
            let format          = Self::gl_format().into();
            let elem_type       = Self::gl_elem_type();
            context.bind_texture(target,Some(&gl_texture));
            context.tex_image_2d_with_u32_and_u32_and_html_image_element
            (target,level,internal_format,format,elem_type,&image).unwrap();

            Self::set_texture_parameters(&context);
        });
        let js_callback = callback.as_ref().unchecked_ref();
        let image       = image_ref.borrow();
        request_cors_if_not_same_origin(&image,url);
        image.set_src(url);
        image.add_event_listener_with_callback("load",js_callback).unwrap();
        *callback_ref.borrow_mut() = Some(callback);
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
    let url    = web_sys::Url::new(url_str).unwrap();
    let origin = web::window().location().origin().unwrap();
    if url.origin() != origin {
        img.set_cross_origin(Some(""));
    }
}







// ===============
// === GpuOnly ===
// ===============

/// Sized, uninitialized texture.
#[derive(Debug)]
pub struct GpuOnlyData {
    /// Texture width.
    pub width  : i32,
    /// Texture height.
    pub height : i32,
}

impl GpuOnlyData {
    fn new(width:i32, height:i32) -> Self {
        Self {width,height}
    }
}

impl<I,T> StorageRelation<I,T> for GpuOnly {
    type Storage = GpuOnlyData;
}

impl From<(i32,i32)> for GpuOnlyData {
    fn from(t:(i32,i32)) -> Self {
        Self::new(t.0,t.1)
    }
}

impl<I:InternalFormat,T:Item>
TextureReload for Texture<GpuOnly,I,T> {
    fn reload(&self) {
        let width           = self.storage().width;
        let height          = self.storage().height;
        let target          = Context::TEXTURE_2D;
        let level           = 0;
        let border          = 0;
        let internal_format = Self::gl_internal_format();
        let format          = Self::gl_format().into();
        let elem_type       = Self::gl_elem_type();

        self.context().bind_texture(target,Some(&self.gl_texture()));
        self.context().tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
        (target,level,internal_format,width,height,border,format,elem_type,None).unwrap();

        Self::set_texture_parameters(self.context());
    }
}






// =============
// === Owned ===
// =============

/// Texture plain data.
#[derive(Debug)]
pub struct OwnedData<T> {
    /// An array containing texture data.
    pub data: Vec<T>,
    /// Texture width.
    pub width: i32,
    /// Texture height.
    pub height: i32,
}

impl<T> OwnedData<T> {
    fn new(data:Vec<T>, width:i32, height:i32) -> Self {
        Self {data,width,height}
    }
}

impl<I,T:Debug> StorageRelation<I,T> for Owned {
    type Storage = OwnedData<T>;
}


impl<I:InternalFormat,T:Item+JsBufferViewArr>
TextureReload for Texture<Owned,I,T> {
    fn reload(&self) {
        let width           = self.storage().width;
        let height          = self.storage().height;
        let target          = Context::TEXTURE_2D;
        let level           = 0;
        let border          = 0;
        let internal_format = Self::gl_internal_format();
        let format          = Self::gl_format().into();
        let elem_type       = Self::gl_elem_type();

        self.context().bind_texture(target,Some(&self.gl_texture()));
        unsafe {
            // We use unsafe array view which is used immediately, so no allocations should happen
            // until we drop the view.
            let view   = self.storage().data.js_buffer_view();
            let result = self.context()
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view
                (target,level,internal_format,width,height,border,format,elem_type,Some(&view));
            result.unwrap();
        }

        Self::set_texture_parameters(&self.context());
    }
}



















































// ======================
// === Meta Iterators ===
// ======================

/// See docs of `with_all_texture_types`.
#[macro_export]
macro_rules! with_all_texture_types_cartesians {
    ($f:ident [$($out:tt)*]) => {
        shapely::cartesian! { [[$f]] [Owned GpuOnly RemoteImage] [$($out)*] }
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

///// Runs the argument macro providing it with list of all possible texture types:
///// `arg! { [Alpha u8] [Alpha f16] [Alpha f32] [Luminance u8] ... }`
//#[macro_export]
//macro_rules! with_all_texture_types {
//    ($f:ident) => {
//        $crate::with_texture_format_relations! { with_all_texture_types_impl [$f] }
//    }
//}
