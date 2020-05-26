#![feature(trait_alias)]
#![feature(set_stdio)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod disabled;
pub mod enabled;

use enso_prelude::*;



// ==============
// === LogMsg ===
// ==============

pub trait LogMsg {
    fn with_log_msg<F: FnOnce(&str) -> T, T>(&self, f:F) -> T;
}

impl LogMsg for &str {
    fn with_log_msg<F: FnOnce(&str) -> T, T>(&self, f:F) -> T {
        f(self)
    }
}

impl<F: Fn() -> S, S:Str> LogMsg for F {
    fn with_log_msg<G: FnOnce(&str) -> T, T>(&self, f:G) -> T {
        f(self().as_ref())
    }
}



// ==============
// === Logger ===
// ==============

pub trait LoggerApi {
    /// Creates a new logger. Path should be a unique identifier of this logger.
    fn new<T:Str>(path:T) -> Self;
    /// Creates a new logger with this logger as a parent.
    fn sub<T:Str>(&self, path:T) -> Self;
    /// Evaluates function `f` and visually groups all logs will occur during its execution.
    fn group<M:LogMsg,T,F:FnOnce() -> T>(&self, msg: M, f:F) -> T;
    /// Log with stacktrace and level:info.
    fn trace<M:LogMsg>(&self, msg:M);
    /// Log with level:debug
    fn debug<M:LogMsg>(&self, msg:M);
    /// Log with level:info.
    fn info<M:LogMsg>(&self, msg:M);
    /// Log with level:warning.
    fn warning<M:LogMsg>(&self, msg:M);
    /// Log with level:error.
    fn error<M:LogMsg>(&self, msg:M);
    /// Visually groups all logs between group_begin and group_end.
    fn group_begin<M:LogMsg>(&self, msg:M);
    /// Visually groups all logs between group_begin and group_end.
    fn group_end(&self);
}



/// ==============
/// === Macros ===
/// ==============

#[macro_export]
macro_rules! fmt {
    ($($arg:tt)*) => (||(format!($($arg)*)))
}

#[macro_export]
macro_rules! group {
    ($logger:expr, $message:tt, {$($body:tt)*}) => {{
        let __logger = $logger.clone();
        __logger.group_begin(|| iformat!{$message});
        let out = {$($body)*};
        __logger.group_end();
        out
    }};
}

#[macro_export]
macro_rules! log_template {
    ($method:ident $logger:expr, $message:tt $($rest:tt)*) => {
        $crate::log_template_impl! {$method $logger, iformat!($message) $($rest)*}
    };
}


#[macro_export]
macro_rules! log_template_impl {
    ($method:ident $logger:expr, $expr:expr) => {{
        $logger.$method(|| $expr);
    }};
    ($method:ident $logger:expr, $expr:expr, $body:tt) => {{
        let __logger = $logger.clone();
        __logger.group_begin(|| $expr);
        let out = $body;
        __logger.group_end();
        out
    }};
}

#[macro_export]
macro_rules! with_internal_bug_message { ($f:ident $($args:tt)*) => { $crate::$f! {
"This is a bug. Please report it and and provide us with as much information as \
possible at https://github.com/luna/enso/issues. Thank you!"
$($args)*
}};}

#[macro_export]
macro_rules! log_internal_bug_template {
    ($($toks:tt)*) => {
        $crate::with_internal_bug_message! { log_internal_bug_template_impl $($toks)* }
    };
}

#[macro_export]
macro_rules! log_internal_bug_template_impl {
    ($note:tt $method:ident $logger:expr, $message:tt $($rest:tt)*) => {
        $crate::log_template_impl! {$method $logger,
            format!("Internal Error. {}\n\n{}",iformat!($message),$note) $($rest)*
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($toks:tt)*) => {
        $crate::log_template! {trace $($toks)*}
    };
}

#[macro_export]
macro_rules! debug {
    ($($toks:tt)*) => {
        $crate::log_template! {debug $($toks)*}
    };
}

#[macro_export]
macro_rules! info {
    ($($toks:tt)*) => {
        $crate::log_template! {info $($toks)*}
    };
}

#[macro_export]
macro_rules! warning {
    ($($toks:tt)*) => {
        $crate::log_template! {warning $($toks)*}
    };
}

#[macro_export]
macro_rules! error {
    ($($toks:tt)*) => {
        $crate::log_template! {error $($toks)*}
    };
}

#[macro_export]
macro_rules! internal_warning {
    ($($toks:tt)*) => {
        $crate::log_internal_bug_template! {warning $($toks)*}
    };
}
