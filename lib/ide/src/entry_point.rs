#![allow(missing_docs)]

use basegl::system::web;

use super::view::project_view::ProjectView;

pub fn entry_point() {
    ProjectView::new().forget();
}