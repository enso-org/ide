//! This module defines code building utilities. The current form of the code builder is not very
//! generic, it mainly targets code which uses indentation and uses `;` as terminator, but it is
//! easily extendable in case other targets will be needed in the future.

#![feature(specialization)]
#![allow(missing_docs)]


use basegl_prelude::*;
use std::fmt::Write;


// ===============
// === Builder ===
// ===============

/// The main code building structure. It is like a string-builder but also knows about the current
/// indentation and contains information if it is needed to add a space when adding a new symbol.
/// For example, you can add `foo` and add `bar` symbols and the resulting code will contain
/// `foo bar` instead of (obviously wrong) `foobar`.
#[derive(Clone,Debug)]
pub struct CodeBuilder {
    pub spaces_in_indent : usize,
    pub indent           : usize,
    pub spaced           : bool,
    pub buffer           : String
}

impl CodeBuilder {
    /// Interface to the underlying `Writer` implementation. Please note that this function does not
    /// take into account the indentation nor spacing, so other smart code constructors are
    /// recommended in most cases.
    pub fn write<S:Str>(&mut self, s:S) {
        self.write_str(s.as_ref()).unwrap();
    }

    /// Increase the indentation of the code. Will be used by the next `newline` call.
    pub fn inc_indent(&mut self) {
        self.indent += 1;
    }

    /// Decrease the indentation of the code. Will be used by the next `newline` call.
    pub fn dec_indent(&mut self) {
        self.indent -= 1;
    }

    /// Create a new line and insert appropriate indentation.
    pub fn newline(&mut self) {
        let space_count = self.spaces_in_indent * self.indent;
        self.write(format!("\n{}"," ".repeat(space_count)));
        self.spaced = true;
    }

    /// Inserts a terminator symbol `;`.
    pub fn terminator(&mut self) {
        self.write(";");
        self.spaced = false;
    }

    /// Adds a new element to the builder. The element can be any form of string, slice, or any
    /// other object which implements the `HasCodeRepr` trait.
    pub fn add<T>(&mut self, t:T) -> &mut Self
        where Self: AddToBuilder<T> {
        self.add_to_builder(t)
    }

    /// Adds a new element and assumes it is spaced (there is no need to add a space before
    /// inserting the next element).
    pub fn add_spaced<T>(&mut self, t:T)
        where Self: AddToBuilder<T> {
        self.add(t);
        self.spaced = true;
    }

    /// Specialization of the `add` method for strings.
    fn add_str<S:Str>(&mut self, s:S) {
        if !self.spaced {
            self.write(" ");
        }
        self.spaced = false;
        self.write(s);
    }
}

// === AddToBuilder ===

/// A helper trait for handling polymorphic input to the `add` method. It allows adding all kind of
/// strings (including slices) and also any objects that implement the `Printer` trait.
pub trait AddToBuilder<T> {
    fn add_to_builder(&mut self, t:T) -> &mut Self;
}

impl<T: HasCodeRepr> AddToBuilder<&T> for CodeBuilder {
    default fn add_to_builder(&mut self, t:&T) -> &mut Self {
        t.build(self);
        self
    }
}

impl AddToBuilder<&String> for CodeBuilder {
    fn add_to_builder(&mut self, t:&String) -> &mut Self {
        self.add_str(t);
        self
    }
}

impl AddToBuilder<&str> for CodeBuilder {
    fn add_to_builder(&mut self, t:&str) -> &mut Self {
        self.add_str(t);
        self
    }
}

impl AddToBuilder<String> for CodeBuilder {
    fn add_to_builder(&mut self, t:String) -> &mut Self {
        self.add_str(t);
        self
    }
}

// === Instances ===

impl Default for CodeBuilder {
    fn default() -> Self {
        let spaces_in_indent = 4;
        let indent           = 0;
        let spaced           = true;
        let buffer           = default();
        Self {spaces_in_indent,indent,spaced,buffer}
    }
}

impl Write for CodeBuilder {
    fn write_str(&mut self, str:&str) -> fmt::Result {
        self.buffer.write_str(str)
    }
}



// ===================
// === HasCodeRepr ===
// ===================

/// Trait implemented by all objects that can have a code representation.
pub trait HasCodeRepr {

    /// Adds the current object to the code builder.
    fn build(&self, builder:&mut CodeBuilder);

    /// Convert the object to code representation.
    fn to_code(&self) -> String {
        let mut builder = default();
        self.build(&mut builder);
        builder.buffer
    }
}

impl<T:HasCodeRepr> HasCodeRepr for Option<T> {
    fn build(&self, builder:&mut CodeBuilder) {
        self.iter().for_each(|t| t.build(builder));
    }
}

impl HasCodeRepr for usize {
    fn build(&self, builder:&mut CodeBuilder) {
        builder.add(self.to_string());
    }
}
