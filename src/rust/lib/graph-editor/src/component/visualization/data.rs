//! This module defines the `data` trait and related functionality.

use crate::prelude::*;

use std::any;
use serde::Deserialize;

// ======================================
// === Wrapper for Visualisation Data ===
// =======================================

/// Type indicator
/// TODO[mm] use enso types?
pub type DataType = any::TypeId;

/// Wrapper for data that can be consumed by a visualisation.
/// TODO[mm] consider static versus dynamic typing for visualizations and data!
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub enum Data {
    JSON   { content : Rc<serde_json::Value> },
    Binary { content : Rc<dyn Any>           },
}

impl Data {
    /// Returns the data as as JSON. If the data cannot be returned as JSON, it will return a
    /// `DataError` instead.
    pub fn as_json(&self) -> Result<Rc<serde_json::Value>, DataError> {
        match &self {
            Data::JSON { content } => Ok(Rc::clone(content)),
            _ => { Err(DataError::InvalidDataType{})  },
        }
    }

    /// Returns the wrapped data in Rust format. If the data cannot be returned as rust datatype, a
    /// `DataError` is returned instead.
    pub fn as_binary<T>(&self) -> Result<Rc<T>, DataError>
        where for<'de> T:Deserialize<'de> + 'static {
        match &self {
            Data::JSON { content } => {
                // We try to deserialize here. Just in case it works.
                let value : serde_json::Value = content.as_ref().clone();
                if let Ok(result) = serde_json::from_value(value) {
                    Ok(Rc::new(result))
                } else {
                    Err(DataError::InvalidDataType)
                }
            },
            Data::Binary { content } => { Rc::clone(content).downcast()
                .or(Err(DataError::InvalidDataType))},
        }
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
    /// TODO add expected/received data types as internal members.
    InvalidDataType
}

