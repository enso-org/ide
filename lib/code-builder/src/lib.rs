#![feature(specialization)]

use basegl_prelude::*;
use std::fmt::Write;

// ===============
// === Builder ===
// ===============

#[derive(Clone,Debug)]
pub struct Builder {
    pub spaces_in_indent : usize,
    pub indent           : usize,
    pub spaced           : bool,
    pub buffer           : String
}

impl Builder {
    pub fn write<S:Str>(&mut self, s:S) {
        self.write_str(s.as_ref()).unwrap();
    }

    pub fn inc_indent(&mut self) {
        self.indent += 1;
    }

    pub fn dec_indent(&mut self) {
        self.indent -= 1;
    }

    pub fn newline(&mut self) {
        let space_count = self.spaces_in_indent * self.indent;
        self.write(format!("\n{}"," ".repeat(space_count)));
        self.spaced = true;
    }

    pub fn terminator(&mut self) {
        self.write(";");
        self.spaced = false;
    }

    pub fn add<T>(&mut self, t:T)
        where Self: AddToBuilder<T> {
        self.add_to_builder(t)
    }

    pub fn add_spaced<T>(&mut self, t:T)
        where Self: AddToBuilder<T> {
        self.add(t);
        self.spaced = true;
    }

    pub fn add_str<S:Str>(&mut self, s:S) {
        if !self.spaced {
            self.write(" ");
        }
        self.spaced = false;
        self.write(s);
    }
}

// === AddToBuilder ===

pub trait AddToBuilder<T> {
    fn add_to_builder(&mut self, t:T);
}

impl<T:Printer> AddToBuilder<&T> for Builder {
    default fn add_to_builder(&mut self, t:&T) {
        t.print(self)
    }
}

impl AddToBuilder<&String> for Builder {
    default fn add_to_builder(&mut self, t:&String) {
        self.add_str(t)
    }
}

impl AddToBuilder<&str> for Builder {
    default fn add_to_builder(&mut self, t:&str) {
        self.add_str(t)
    }
}

impl AddToBuilder<String> for Builder {
    default fn add_to_builder(&mut self, t:String) {
        self.add_str(t)
    }
}

// === Instances ===

impl Default for Builder {
    fn default() -> Self {
        let spaces_in_indent = 4;
        let indent           = 0;
        let spaced           = true;
        let buffer           = default();
        Self {spaces_in_indent,indent,spaced,buffer}
    }
}

impl Write for Builder {
    fn write_str(&mut self, str:&str) -> fmt::Result {
        self.buffer.write_str(str)
    }
}

// === Smart Builder ===

#[macro_export]
macro_rules! build {
    ($builder:expr, $($expr:expr),*) => {
        {$($builder.add($expr));*}
    }
}


// ===============
// === Printer ===
// ===============

pub trait Printer {
    fn print(&self, builder:&mut Builder);
    fn code(&self) -> String {
        let mut builder = default();
        self.print(&mut builder);
        builder.buffer
    }
}

impl<T:Printer> Printer for Option<T> {
    fn print(&self, builder:&mut Builder) {
        self.iter().for_each(|t| t.print(builder));
    }
}

impl Printer for usize {
    fn print(&self, builder:&mut Builder) {
        builder.add(self.to_string())
    }
}