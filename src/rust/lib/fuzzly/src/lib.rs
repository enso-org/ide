//! Fuzzly Search Utilities.
//!
//! This crate is designed to be used in various search engines; when you get the list of names
//! matching the given pattern, the next step is to order the items, so the best matches
//! are listed first. In such case the `find_best_subsequence` function may be used to score (order
//! priority) for each element.
//!
//! The metrics used for scoring may be adjusted by implementing `Metric` trait, or by customizing
//! parameters of metrics defined in `metric` module.
#![feature(option_result_contains)]

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod subsequence_graph;
pub mod metric;
pub mod score;

pub use enso_prelude as prelude;
pub use metric::Metric;
pub use subsequence_graph::Graph as SubsequenceGraph;
pub use score::Subsequence;
pub use score::matches;
pub use score::find_best_subsequence;
