#![allow(missing_docs)]

use super::view::project::ProjectView;

pub fn entry_point() {
    ProjectView::new().forget();
}
