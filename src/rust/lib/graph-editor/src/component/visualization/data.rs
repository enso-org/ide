//! Definition of data understandable by visualizations.

use crate::prelude::*;



// ============
// === Json ===
// ============

/// Json representation with a fast clone operation. Used for transmitting visualization data via
/// FRP networks.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Json {
    rc : Rc<serde_json::Value>
}

impl Deref for Json {
    type Target = serde_json::Value;
    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}

impl From<serde_json::Value> for Json {
    fn from(t:serde_json::Value) -> Self {
        let rc = Rc::new(t);
        Self {rc}
    }
}



// ============
// === Data ===
// ============

/// Wrapper for data that can be consumed by a visualization.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum Data {
    Json { content : Json },
    Binary, // TODO replace with actual binary data stream.
}

impl Default for Data {
    fn default() -> Self {
        let content = default();
        Self::Json {content}
    }
}

impl From<serde_json::Value> for Data {
    fn from(t:serde_json::Value) -> Self {
        let content = t.into();
        Self::Json {content}
    }
}



// ==============
// === Errors ===
// ==============

/// Indicates a problem with the provided data. That is, the data has the wrong format, or maybe
/// violates some other assumption of the visualization.
#[derive(Copy,Clone,Debug)]
pub enum DataError {
    /// Indicates that that the provided data type does not match the expected data type.
    InvalidDataType,
    /// The data caused an error in the computation of the visualization.
    InternalComputationError,
}



// =============================
// === Sample Data Generator ===
// =============================

/// The `MockDataGenerator3D` creates sample data in the format of `Vec<Vector3<f32>>`. The data
/// is changing incrementally on every call. The data is meant to be interpreted as a number of
/// circles defined through x-coordinate, y-coordinate and radius which respectively correspond to
/// the `Vectors3`s x/y/z values.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct MockDataGenerator3D {
    counter: Rc<Cell<f32>>
}

impl MockDataGenerator3D {
    /// Generate new data set.
    pub fn generate_data(&self) -> Vec<Vector3<f32>> {
        let current_value = self.counter.get();
        self.counter.set(current_value + 0.1);

        let delta1 = current_value.sin() * 10.0;
        let delta2 = current_value.cos() * 10.0;

        vec![
            Vector3::new(25.0,                 75.0,          25.0 + delta1),
            Vector3::new(25.0,                 25.0,          25.0 + delta2),
            Vector3::new(75.0 - 12.5,          75.0 + delta1, 5.0          ),
            Vector3::new(75.0 + 12.5,          75.0 + delta2, 15.0         ),
            Vector3::new(75.0 - 12.5 + delta1, 25.0 + delta2, 5.0          ),
            Vector3::new(75.0 + 12.5 + delta2, 25.0 + delta1, 15.0         ),
        ]
    }
}
