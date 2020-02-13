#![allow(missing_docs)]

use super::view::project_view::ProjectView;

pub fn entry_point() {
    ProjectView::new().forget();
}
