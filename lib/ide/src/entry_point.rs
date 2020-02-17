//! This module defines IDE's entry point function.

use super::view::project::ProjectView;

/// This function is the IDE entry point responsible for creating the ProjectView.
pub fn entry_point() {
    ProjectView::new().forget();
}
