//! General purpose code for dealing with environment variables.

use enso_prelude::*;

use std::str::FromStr;

/// Gets the string with conents of given environment variable.
/// If the variable wasn't set, returns a default value from a second argument.
pub fn env_var_or(varname:&str, default_value:&str) -> String {
    std::env::var(varname).unwrap_or_else(|_| default_value.into())
}

/// Parses contents of the given environment variable.
/// If the variable is not present or fails to parse, default value is silently
/// returned.
pub fn parse_var_or<T:FromStr>(varnmae:&str, default_value:T) -> T {
    if let Ok(value) = std::env::var(varnmae) {
        if let Ok(parsed) = value.parse::<T>() {
            return parsed;
        }
    }
    default_value
}