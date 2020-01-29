//! This module contains structures describing alignment.

use crate::prelude::*;

// =================
// === Alignment ===
// =================

/// Structure describing horizontal and vertical alignment separately.
#[derive(Clone,Debug)]
pub struct Alignment {
    /// Horizontal alignment.
    pub horizontal: HorizontalAlignment,
    /// Vertical alignment.
    pub vertical: VerticalAlignment,
}

/// Possible values of horizontal alignment.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum HorizontalAlignment {Left,Center,Right}

/// Possible values of vertical alignment.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum VerticalAlignment {Top,Center,Bottom}

impl Default for HorizontalAlignment { fn default() -> Self { Self::Center } }
impl Default for VerticalAlignment   { fn default() -> Self { Self::Center } }
impl Default for Alignment {
    fn default() -> Self {
        let horizontal = default();
        let vertical   = default();
        Self {horizontal,vertical}
    }
}
