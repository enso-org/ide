#![feature(weak_counts)]

pub mod controller;
pub mod todo;
pub mod entry_point;
pub mod project_view;
pub mod view_layout;
pub mod text_editor;

pub mod prelude {
    pub use enso_prelude::*;

    pub use futures::Future;
    pub use futures::FutureExt;
    pub use futures::Stream;
    pub use futures::StreamExt;
}
