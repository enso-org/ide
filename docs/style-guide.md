# Rust Style Guide

## Motivation - why not to completely rely a formatting tool?

The docs of rustfmt state that "formatting code is a mostly mechanical task 
which takes both time and mental effort. By using an automatic formatting tool, 
a programmer is relieved of this task and can concentrate on more important 
things.". While in many cases it is true, if the uthor of code does not take 
extra effort to make his code pretty by refactoring long lines to variables, 
moving code to specific modules, or sections, the formatting tool will result in
a code which is hard to read and hard to write. Thus, it is important to take 
write the code in such way that we can be proud of its quality.

Because `rustfmt` does not support multiple of our requirements, we have created
a guide how to format Rust code in this project. Please read it carefully. We 
hope that in the future, many of the things described below will be possible 
while using `rustfmt` (and we encourage you to contribute there!), however, 
even if it happens, many parts of this guide will still be valid and will need 
to be handled manually.



## Styling rules

### Code width
Each line in a source file should have max of 80 chars of text (including 
comments).


### Multiline Expressions
Most (preferably all) expressions should be single line. Multiline expression is
hard to read and introduces noise in the code. Often, it is also an indicator of
a code which is not properly refactored. Try to refactor parts of multiline 
expressions to well-named variables, and divide them to several single-line 
expressions.

Example of bad formatted code:
```rust
pub fn new() -> Self {
    let shape_dirty = ShapeDirty::new(logger.sub("shape_dirty"),
        on_dirty.clone());
    let dirty_flag = MeshRegistryDirty::new(logger.sub("mesh_registry_dirty"),
        on_dirty);
    Self { dirty_flag, dirty_flag }
}
```
Example of properly formatted code:

```rust
pub fn new() -> Self {
    let sub_logger  = logger.sub("shape_dirty");
    let shape_dirty = ShapeDirty::new(sub_logger,on_dirty.clone());
    let sub_logger  = logger.sub("mesh_registry_dirty");
    let dirty_flag  = MeshRegistryDirty::new(sub_logger,on_dirty);
    Self {shape_dirty,dirty_flag}
}
```


### Vertical alignment

The following elements should be aligned vertically in subsequent lines:
- assignment operators (`=`),
- type operators (`:`),
- match arrows (`=>`), 
- similar parameters or types.

Examples: 
```rust
impl Printer for GlobalVarStorage {
    fn print(&self, builder:&mut Builder) {
        match self {
            Self::ConstStorage      => build!(builder,"const"),
            Self::UniformStorage    => build!(builder,"uniform"),
            Self::InStorage  (qual) => build!(builder,"in",qual),
            Self::OutStorage (qual) => build!(builder,"out",qual),
        }
    }
}
```


### Spaces 
- Type operator is not spaced: `fn test(foo:String, bar:Int) { ... }`.
- Commas between comples expressions (including arg list) are spaced.
- Commas between simple elements are not spaced: `Result<Self,Error>`.


### Function definitions
The following examples show proper function styles:

```rust
pub fn new<Dom: Str>(dom:Dom, logger:Logger) -> Result<Self,Error> {
    ...
}
```

```rust
pub fn new<Dom: Str>
(dom:Dom, logger:Logger, on_dirty:OnDirty) -> Result<Self,Error> {
    ...
}
```

```rust
pub fn new<Dom: Str>
(dom:Dom, logger:Logger, on_dirty:OnDirty, on_remove:OnRemove) 
-> Result<Self,Error> {
    ...
}
```

```rust
pub fn new<Dom: Str>
( dom        : Dom
, logger     : Logger
, on_dirty   : OnDirty
, on_remove  : OnRemove
, on_replace : OnReplace
) -> Result<Self,Error> {
    ...
}
```


### Impl definitions
Sometimes when browsing code it is hard to understand where is the header of 
an impl declaration. Thus the following style allows for such a fast discovery. 
All of the following codes are correct:

```rust
// No constraints
impl<T> Printer for Option<T> {
    ...
}
```

```rust
// Some constraints
impl<T:Printer> 
Printer for Option<T> {
    ...
}
```

```rust
// Constraints in where block
impl<T> Printer for Option<T> 
where T: Printer {
    ...
}
```


