//! This module defines the visualization widgets and related functionality.
//!
//! The overall architecture of visualizations consists of three parts:
//! (1) the `DataRenderer` is a trait that sits at the core of the visualisation system. A
//! `DataRenderer` provides the `display::Object` that shows the actual visualization. It is fed
//! with data and provides updates about its state as well as data output.
//!
//! (2) the `Visualization` wraps the `DataRenderer` and implements the generic tasks that are the
//! same for all visualisations. That is, interfacing with the other UI elements, providing data
//! updates to the `DataRenderer`, and propagating information about the state changes in the
//! `DataRenderer`.
//!
//! (3) the `Container` sits on top of the Visualisation and provides UI elements that facilitate
//! generic interactions, for example, selecting a specific visualisation or setting input data for
//! a `Visualisation`. The `Container` also provides the FRP API that allows internal interaction
//! with the `Visualisation`.
//!
//! In addition this module also contains a `Data` struct that provides a dynamically typed way to
//! handle data for visualisations. This allows the `Visualisation` struct to be without type
//! parameters and simplifies the FRP communication and complexity of the node system.
pub mod class;
pub mod container;
pub mod renderer;
pub mod data;

pub use class::*;
pub use data::*;
pub use container::*;
pub use renderer::*;
