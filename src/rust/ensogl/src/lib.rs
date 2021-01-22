//! EnsoGL is a blazing fast 2D vector rendering engine with a rich set of primitives and a GUI
//! component library. It is able to display millions of shapes 60 frames per second in a web
//! browser on a modern laptop hardware. This is the main entry point to the library, which
//! re-exports several components to a common namespace.
//!
//! # Rendering Architecture
//!
//! https://www.nomnoml.com :
//! ```ignore
//!     #zoom: 0.6
//!     #gutter:100
//!     #padding: 14
//!     #leading: 1.4
//!     #spacing: 60
//!     #edgeMargin:5
//!     #arrowSize: 0.8
//!     #fill: #FFFFFF; #fdf6e3
//!
//!     #background: #FFFFFF
//!     #.usr: visual=roundrect title=bold stroke=rgb(237,80,80)
//!     #.dyn: visual=roundrect title=bold dashed
//!     #.cpu: visual=roundrect title=bold
//!     #.gpu: stroke=rgb(68,133,187) visual=roundrect
//!
//!     [<gpu> Buffer]
//!     [<gpu> WebGL Context]
//!     [<cpu> AttributeScope]
//!     [<cpu> Attribute]
//!     [<cpu> Mesh]
//!     [<cpu> Material]
//!     [<cpu> Symbol]
//!     [<cpu> SymbolRegistry]
//!     [<cpu> World]
//!     [<cpu> Scene]
//!     [<cpu> View]
//!     [<cpu> SpriteSystem]
//!     [<cpu> Sprite]
//!     [<cpu> ShapeSystem]
//!     [<dyn> ShapeView]
//!     [<usr> *Shape]
//!     [<usr> *ShapeSystem]
//!     [<usr> *Component]
//!     [<cpu> Application]
//!
//!     [AttributeScope] o- [Buffer]
//!     [Buffer] o-- [Attribute]
//!     [Mesh]* o- 4[AttributeScope]
//!     [Symbol]* o- [Mesh]
//!     [Symbol]* o- [Material]
//!     [SymbolRegistry] o- [Symbol]
//!     [Scene] - [SymbolRegistry]
//!     [Scene] o- [View]
//!     [Scene] - [WebGL Context]
//!
//!     [SpriteSystem] o- [Symbol]
//!     [SpriteSystem] o-- [Sprite]
//!     [ShapeSystem] o- [SpriteSystem]
//!     [Sprite] o- [Symbol]
//!     [Sprite] o- [Attribute]
//!     [*Shape] o- [Sprite]
//!     [*ShapeSystem] o- [ShapeSystem]
//!     [*ShapeSystem] o-- [*Shape]
//!     [*Component] o- [ShapeView]
//!     [ShapeView] - [*Shape]
//!     [View] o- [Symbol]
//!     [View] o- [*ShapeSystem]
//!     [World] o- [Scene]
//!     [Application] - [World]
//!     [Application] o- [*Component]
//! ```
//! # Shapes Rendering
//!
//!
//!
//! ## The Current Architecture
//! The current implementation uses instanced rendering to display shapes. First, a simple
//! rectangular geometry is defined, and for each new instance, a new attribute is added to the list
//! of attached attribute arrays. During rendering, we use the `draw_arrays_instanced` WebGL call to
//! iterate over the arrays and draw each shape. The shape placement is done from within its vertex
//! shader.
//!
//! See the documentation of [`crate::system::gpu::data::Buffer`].
//! See the documentation of [`crate::system::gpu::data::Attribute`].
//! See the documentation of [`crate::system::gpu::data::AttributeScope`].
//!
//! ### Known Issues / Ideas of Improvement
//!
//! The current architecture is very efficient at shapes rendering, which comes with a few
//! limitations. Below, there are many other architectures described with their own gains and
//! problems and we should consider improving the current approach in the future. However, keep in
//! mind that the listed limitations allow us for very fast rendering pipeline, so it's questionable
//! whether we would like to ever change it.
//!
//! The most significant limitations of the current approach are:
//!
//! - No possibility to depth-sort the shapes instances.
//!   The used `draw_arrays_instanced` WebGL draw call iterates over all attrib arrays and draws a
//!   new instance for each entry. There is no possibility to specify the iteration order, while
//!   re-ordering the attrib arrays can be CPU heavy (with big instance count) and would require
//!   re-sending big amount of data between CPU and GPU (e.g. moving the top-most instance to the
//!   bottom would require moving its attribs in all attached attrib arrays from the last position
//!   to the front, and thus, re sending ALL attrib arrays to the GPU (for ALL INSTANCES)).
//!
//! - No efficient memory management.
//!   In case an instance with a high ID exists and many instances with lower IDs are already
//!   destroyed, the memory of the destroyed instances cannot be freed. This is because currently
//!   the sprite instances remember the ID (wrapper over usize) of the instance, which is used as
//!   the attrib array index. Thus, it is impossible to update the number in all sprite instances in
//!   memory, and sort the instances to move the destroyed ones to the end of the buffer to free it.
//!   This could be easily solved by using `Rc<Cell<ID>>` instead, however, it is important to
//!   benchmark how big performance impact this will cause. Also, other architectures may provide
//!   alternative solutions.
//!
//! - No possibility to render shape instances using different cameras (in separate draw calls).
//!   Currently, the shape instances are drawn with the `draw_arrays_instanced` WebGL draw call.
//!   This API allows drawing all instances at once, so it is not possible to draw only some subset
//!   of them, and thus, it is not possible to update the view-matrix uniform between the calls.
//!   The OpenGL 4.2 introduced a specialized draw call that would solve this issue entirely,
//!   however, it is not accessible from within WebGL
//!   ([glDrawArraysInstancedBaseInstance](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glDrawArraysInstancedBaseInstance.xhtml)).
//!
//!
//! ### Depth-sorting, memory cleaning, and indexes re-using.
//!
//! The current approach, however, doesn't allow us to depth-sort the shapes instances. Also, it
//! does not allow for efficient memory management in case an instance with a high ID exists and
//! many instances with lover IDs are already destroyed. This section describes possible alternative
//! architectures and compares them from this perspective.
//!
//! There are several possible implementation architectures for attribute management. The currently
//! used architecture may not be the best one, but the choice is not obvious and would require
//! complex benchmarking. However, lets compare the available architectures and lets list their
//! good and bad sides:
//!
//!
//! #### A. Drawing instanced geometry (the current architecture).
//!
//! - Rendering.
//!   Very fast. May not be as fast as some of other methods, but that may not be the case with
//!   modern hardware, see: https://stackoverflow.com/a/65376034/889902, and also
//!   https://stackoverflow.com/questions/62537968/using-opengl-instancing-for-rendering-2d-scene-with-object-depths-and-alpha-blen#answer-62538277
//!
//! - Changing attribute & GPU memory consumption.
//!   Very fast and with low memory consumption. Requires only 1 WebGL call (attribute per
//!   instance).
//!
//! - Visual sorting of instances (depth management).
//!   Complex. Requires sorting of all attribute buffers connected with a particular instance. For
//!   big buffers (many instances) it may require significant CPU -> GPU data upload. For example,
//!   taking the last element to the front, would require shifting all attributes in all buffers,
//!   which basically would mean uploading all data to the GPU from scratch for that particular
//!   geometry. Also, this would require keeping instance IDs in some kind of `Rc<Cell<usize>>`,
//!   as during sorting, the instance IDs will change, so all sprites would need to be updated.
//!
//!
//! #### B. Drawing non-instanced, indexed geometry.
//!
//! - Rendering.
//!   Very fast. May be faster than architecture (A). See it's description to learn more.
//!
//! - Changing attribute & GPU memory consumption.
//!   4 times slower and 4 times more memory hungry than architecture (A). Requires setting each
//!   attribute for each vertex (4 WebGL calls). During drawing, vertexes are re-used by using
//!   indexed geometry rendering.
//!
//! - Visual sorting of instances (depth management).
//!   The same issues as in architecture (A). Even more CPU -> GPU heavy, as the attribute count
//!   is bigger.
//!
//!
//! #### C. Drawing non-instanced, non-indexed geometry. Using indexing for sorting.
//!
//! - Rendering.
//!   Very fast. May be faster than architecture (A). See it's description to learn more.
//!
//! - Changing attribute & GPU memory consumption.
//!   6 times slower and 6 times more memory hungry than architecture (A). Requires setting each
//!   attribute for each vertex (6 WebGL calls). During drawing, vertexes are not re-used, and thus
//!   we need to set attributes for each vertex of each triangle.
//!
//! - Visual sorting of instances (depth management).
//!   Simple. We can re-use index buffer to sort the geometry by telling GPU in what order it
//!   should render each of the vertexes. Unlike previous architectures, this would not require to
//!   create any more internally mutable state regarding attribute index management (the indexes
//!   will not change during sorting).
//!
//!   However, sorting for the needs of memory compression (removing unused memory for sparse
//!   attrib arrays) would still require re-uploading sorted data to GPU, just as in architecture
//!   (A).
//!
//!
//! #### D. Keeping all attribute values in a texture and passing index buffer to the shader.
//!
//! This is a very different architecture to what is currently implemented and might require very
//! complex refactoring in order to be even tested and benchmarked properly. To learn more about the
//! idea, follow the link: https://stackoverflow.com/a/65376034/889902.
//!
//! - Rendering.
//!   Fast. May be slower than architecture (A). Needs real benchmarks.
//!
//! - Changing attribute & GPU memory consumption.
//!   Changing attribute would require 2 WebGL calls: the `bindTexture`, and `texParameterf` (or
//!   similar). Performance of this solution is questionable, but in real life, it may be as fast
//!   as architecture (A). The memory consumption should be fine as well, as WebGL textures behave
//!   like C++ Vectors, so even if we allocate the texture of max size, it will occupy only the
//!   needed space. This will also limit the number of instances on the stage, but the limit will
//!   be big enough (assuming max texture od 2048px x 2048px and 20 float attributes per shader,
//!   this will allow us to render over 200 000 shapes). Also, this architecture would allow us to
//!   pass more attributes to shaders than it is currently possible, which on the other hand,
//!   would probably negatively affect the fragment shader performance.
//!
//! - Visual sorting of instances (depth management).
//!   Simple. Next to the attribute texture, we can pass index buffer to the shader, which will
//!   dictate what initial offset in the texture should be used. This would allow for the fastest
//!   sorting mechanism of all of the above architectures.
//!
//!   However, sorting for the needs of memory compression (removing unused memory for sparse
//!   attrib arrays) would still require re-uploading sorted data to GPU, just as in architecture
//!   (A).
//!
//!
//! #### E. Using the depth-buffer for sorting.
//!
//! As with architecture (C), this is a very different architecture to what is currently
//! implemented and might require very complex refactoring in order to be even tested and
//! benchmarked properly. This architecture, however, is the most common architecture among all
//! WebGL / OpenGL applications, but it is not really well suitable for SDF-based shapes rendering,
//! as it requires anti-aliasing to be done by multisampling, which is not needed with SDF-based
//! rasterization. It lowers the quality and drastically increases the rendering time (in the case
//! of 4x4 multisampling, the rendering time is 16x bigger than the time of architecture (A)).
//!
//! There is one additional thread to consider here, namely, with some browsers, systems, and GPU
//! combinations, the super-sampling anti-aliasing is not accessible in WebGL. In such situations we
//! could use a post-processing anti-aliasing techniques, such as [FXAA][1] or [SMAA][2], however,
//! the resulting image quality will be even worse. We could also use custom multi-sampled render
//! buffers for implementing [multi-sampled depth buffers][3].
//! [1] https://github.com/mitsuhiko/webgl-meincraft/blob/master/assets/shaders/fxaa.glsl
//! [2] http://www.iryoku.com/papers/SMAA-Enhanced-Subpixel-Morphological-Antialiasing.pdf
//! [3] https://stackoverflow.com/questions/50613696/whats-the-purpose-of-multisample-renderbuffers
//!
//! - Rendering.
//!   May be 9x - 16x slower than architecture (A), depending on multi-sampling level. Also, the
//!   final image quality and edge sharpness will be lower. There is, however, an open question,
//!   whether there is an SDF-suitable depth-buffer sorting technique which would not cause such
//!   downsides (maybe involving SDF-based depth buffer). Currently, we don't know of any such
//!   technique.
//!
//! - Changing attribute & GPU memory consumption.
//!   Fast with low memory consumption. The same as with architecture (A), (B), or (C).
//!
//! - Visual sorting of instances (depth management).
//!   Simple and fast. Much faster than any other architecture listed before, as it does not
//!   require upfront CPU-side buffer sorting.
//!
//!
//! #### F. Using depth-peeling / dual depth-peeling algorithms.
//!
//! As with architecture (C), this is a very different architecture to what is currently
//! implemented and might require very complex refactoring in order to be even tested and
//! benchmarked properly. The idea is to render the scene multiple times, as long as some objects
//! do overlap, by "peeling" the top-most (and bottom-most) layers every time. See the
//! [Interactive Order-Independent Transparency][1], the
//! [Order Independent Transparency with Dual Depth Peeling][2], and the
//! [sample WebGL implementation][3] to learn more.
//!
//! [1] https://my.eng.utah.edu/~cs5610/handouts/order_independent_transparency.pdf
//! [2] http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.193.3485&rep=rep1&type=pdf
//! [3] https://medium.com/@shrekshao_71662/dual-depth-peeling-implementation-in-webgl-11baa061ba4b
//!
//! - Rendering.
//!   May be several times slower than architecture (A) due to the need to render the scene by
//!   peeling components. However, in contrast to the architecture (D), the final image quality
//!   should be as good as with architecture (A), (B), or (C).
//!
//! - Changing attribute & GPU memory consumption.
//!   Fast with low memory consumption. The same as with architecture (A), (B), or (C).
//!
//! - Visual sorting of instances (depth management).
//!   Simple and fast. As fast as architecture (E), as it does not require upfront CPU-side buffer
//!   sorting.

pub use ensogl_core::*;

/// Data type declarations.
pub mod data {
    pub use ensogl_core::data::*;
    pub use ensogl_text as text;
}

/// Graphical interface related components, like buttons, sliders, or text areas.
pub mod gui {
    pub use ensogl_core::gui::*;
    pub use ensogl_text::component as text;
}
