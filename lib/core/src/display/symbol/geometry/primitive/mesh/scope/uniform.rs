use crate::prelude::*;

use enum_dispatch::*;
use nalgebra::Matrix4;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use web_sys::WebGlUniformLocation;

use crate::system::gpu::data::GpuData;
use crate::system::gpu::data::Empty;
use crate::system::gpu::data::ContextUniformOps;
use crate::system::web::Logger;
use crate::display::render::webgl::Context;



macro_rules! shared_bracket_impl {
    ([impl [$($impl_params:tt)*] $name:ident $name_mut:ident $([$($params:tt)*])?] [
        $( pub fn $fn_name:ident $([$($fn_params:tt)*])? ($($fn_args:tt)*) $(-> $fn_type:ty)? {
               $($fn_body:tt)*
        })*
    ]) => {
        impl <$($impl_params)*> $name_mut $(<$($params)*>)? {
            $(pub fn $fn_name $(<$($fn_params)*>)* ($($fn_args)*) $(-> $fn_type)? {$($fn_body)*})*
        }

        impl <$($impl_params)*> $name $(<$($params)*>)? {
            $(shared_bracket_fn! {
                $name_mut :: pub fn $fn_name [$($($fn_params)*)*] ($($fn_args)*) $(-> $fn_type)?
            })*
        }
    };
}

macro_rules! shared_bracket_fn {
    ( $base:ident :: pub fn new $([$($params:tt)*])?
      ($($arg:ident : $arg_type:ty),*) $(-> $type:ty)? ) => {
        pub fn new $(<$($params)*>)* ($($arg : $arg_type),*) $(-> $type)? {
            Self { rc: Rc::new(RefCell::new($base::new($($arg),*))) }
        }
    };
    ( $base:ident :: pub fn $name:ident $([$($params:tt)*])?
      (&self $(,$($arg:ident : $arg_type:ty),+)?) $(-> $type:ty)? ) => {
        pub fn $name $(<$($params)*>)* (&self $(,$($arg : $arg_type),*)?) $(-> $type)? {
            self.rc.borrow().$name($($($arg),*)?)
        }
    };
    ( $base:ident :: pub fn $name:ident $([$($params:tt)*])?
      (&mut self $(,$($arg:ident : $arg_type:ty),+)?) $(-> $type:ty)? ) => {
        pub fn $name $(<$($params)*>)* (&self $(,$($arg : $arg_type),*)?) $(-> $type)? {
            self.rc.borrow_mut().$name($($($arg),*)?)
        }
    };
}



macro_rules! shared_bracket_normalized {
    ( [$name:ident] [
        $(#[$($meta:tt)*])*
        pub struct $name_mut:ident $params:tt {
            $($field:ident : $field_type:ty),* $(,)?
        }

        $(impl $([$($impl_params:tt)*])? {$($impl_body:tt)*})*
    ]) => {
        shared_struct! {
            $(#[$($meta)*])*
            pub struct $name $name_mut $params {
                $($field : $field_type),*
            }
        }

        $(angles_to_brackets_shallow! {shared_bracket_impl
            [impl [$($($impl_params)*)?] $name $name_mut $params] $($impl_body)*
        })*
    };
}

macro_rules! shared_struct {
    (
        $(#[$($meta:tt)*])*
        pub struct $name:ident $name_mut:ident [$($params:tt)*] {
            $($field:ident : $field_type:ty),* $(,)?
        }
    ) => {
        $(#[$($meta)*])*
        pub struct $name <$($params)*> { rc: Rc<RefCell<$name_mut<$($params)*>>> }

        $(#[$($meta)*])*
        pub struct $name_mut <$($params)*> { $($field : $field_type),* }
    };
}


macro_rules! angles_to_brackets_shallow {
    ($f:ident $f_arg:tt $($in:tt)*) => {
        _angles_to_brackets_shallow! { $f $f_arg [] [] [] $($in)* }
    }
}

macro_rules! _angles_to_brackets_shallow {
    ( $f:ident $f_arg:tt []                        [$($out:tt)*] []                                ) => { $f! { $f_arg [$($out)*] } };
    ( $f:ident $f_arg:tt []                        [$($out:tt)*] [$($cout:tt)*]                    ) => { _angles_to_brackets_shallow! { $f $f_arg []                     [$($out)* $($cout)*]        []                          } };
    ( $f:ident $f_arg:tt []                        [$($out:tt)*] [$($cout:tt)*] <     $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [.]                    [$($out)* $($cout)*]        []                $($rest)* } };
    ( $f:ident $f_arg:tt []                        $out:tt       [$($cout:tt)*] <<    $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [. .]                  $out                        [$($cout)* <]     $($rest)* } };
    ( $f:ident $f_arg:tt []                        $out:tt       [$($cout:tt)*] <<<   $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [. . .]                $out                        [$($cout)* <<]    $($rest)* } };
    ( $f:ident $f_arg:tt []                        $out:tt       [$($cout:tt)*] <<<<  $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [. . . .]              $out                        [$($cout)* <<<]   $($rest)* } };
    ( $f:ident $f_arg:tt []                        $out:tt       [$($cout:tt)*] <<<<< $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [. . . . .]            $out                        [$($cout)* <<<<]  $($rest)* } };
    ( $f:ident $f_arg:tt [$($depth:tt)*]           $out:tt       [$($cout:tt)*] <     $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)* .]         $out                        [$($cout)* <]     $($rest)* } };
    ( $f:ident $f_arg:tt [$($depth:tt)*]           $out:tt       [$($cout:tt)*] <<    $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)* . .]       $out                        [$($cout)* <<]    $($rest)* } };
    ( $f:ident $f_arg:tt [$($depth:tt)*]           $out:tt       [$($cout:tt)*] <<<   $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)* . . .]     $out                        [$($cout)* <<<]   $($rest)* } };
    ( $f:ident $f_arg:tt [$($depth:tt)*]           $out:tt       [$($cout:tt)*] <<<<  $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)* . . . .]   $out                        [$($cout)* <<<<]  $($rest)* } };
    ( $f:ident $f_arg:tt [$($depth:tt)*]           $out:tt       [$($cout:tt)*] <<<<< $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)* . . . . .] $out                        [$($cout)* <<<<<] $($rest)* } };
    ( $f:ident $f_arg:tt [.]                       [$($out:tt)*] $cout:tt       >     $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg []                     [$($out)* $cout]            []                $($rest)* } };
    ( $f:ident $f_arg:tt [. .]                     [$($out:tt)*] [$($cout:tt)*] >>    $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg []                     [$($out)* [$($cout)* >]]    []                $($rest)* } };
    ( $f:ident $f_arg:tt [. . .]                   [$($out:tt)*] [$($cout:tt)*] >>>   $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg []                     [$($out)* [$($cout)* >>]]   []                $($rest)* } };
    ( $f:ident $f_arg:tt [. . . .]                 [$($out:tt)*] [$($cout:tt)*] >>>>  $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg []                     [$($out)* [$($cout)* >>>]]  []                $($rest)* } };
    ( $f:ident $f_arg:tt [. . . . .]               [$($out:tt)*] [$($cout:tt)*] >>>>> $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg []                     [$($out)* [$($cout)* >>>>]] []                $($rest)* } };
    ( $f:ident $f_arg:tt [. $($depth:tt)*]         $out:tt       [$($cout:tt)*] >     $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)*]           $out                        [$($cout)* >]     $($rest)* } };
    ( $f:ident $f_arg:tt [. . $($depth:tt)*]       $out:tt       [$($cout:tt)*] >>    $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)*]           $out                        [$($cout)* >>]    $($rest)* } };
    ( $f:ident $f_arg:tt [. . . $($depth:tt)*]     $out:tt       [$($cout:tt)*] >>>   $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)*]           $out                        [$($cout)* >>>]   $($rest)* } };
    ( $f:ident $f_arg:tt [. . . . $($depth:tt)*]   $out:tt       [$($cout:tt)*] >>>>  $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)*]           $out                        [$($cout)* >>>>]  $($rest)* } };
    ( $f:ident $f_arg:tt [. . . . . $($depth:tt)*] $out:tt       [$($cout:tt)*] >>>>> $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg [$($depth)*]           $out                        [$($cout)* >>>>>] $($rest)* } };

    // Function output handling
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] -> $t1:tt {$($b:tt)*} $($rest:tt)* )                                                         => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* -> $t1 {$($b)*}]                                 $($rest)* } };
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] -> $t1:tt $t2:tt {$($b:tt)*} $($rest:tt)* )                                                  => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* -> $t1 $t2 {$($b)*}]                             $($rest)* } };
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] -> $t1:tt $t2:tt $t3:tt {$($b:tt)*} $($rest:tt)* )                                           => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* -> $t1 $t2 $t3 {$($b)*}]                         $($rest)* } };
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] -> $t1:tt $t2:tt $t3:tt $t4:tt {$($b:tt)*} $($rest:tt)* )                                    => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* -> $t1 $t2 $t3 $t4 {$($b)*}]                     $($rest)* } };
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] -> $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt {$($b:tt)*} $($rest:tt)* )                             => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* -> $t1 $t2 $t3 $t4 $t5 {$($b)*}]                 $($rest)* } };
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] -> $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt {$($b:tt)*} $($rest:tt)* )                      => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* -> $t1 $t2 $t3 $t4 $t5 $t6 {$($b)*}]             $($rest)* } };
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] -> $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt {$($b:tt)*} $($rest:tt)* )               => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* -> $t1 $t2 $t3 $t4 $t5 $t6 $t7 {$($b)*}]         $($rest)* } };
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] -> $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt {$($b:tt)*} $($rest:tt)* )        => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* -> $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 {$($b)*}]     $($rest)* } };
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] -> $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt {$($b:tt)*} $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* -> $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 {$($b)*}] $($rest)* } };

    // Any token handling
    ( $f:ident $f_arg:tt $depth:tt $out:tt [$($cout:tt)*] $t:tt $($rest:tt)* ) => { _angles_to_brackets_shallow! { $f $f_arg $depth $out [$($cout)* $t] $($rest)* } };
}

macro_rules! shared_bracket {
    ([$name:ident] [$($in:tt)*]) => {
        normalize_input! { shared_bracket_normalized [$name] $($in)* }
    }
}

macro_rules! shared {
    ($name:ident $($in:tt)*) => {
        angles_to_brackets_shallow! { shared_bracket [$name] $($in)* }
    }
}


macro_rules! normalize_input {
    ($f:ident $f_args:tt $($in:tt)*) => {
        _normalize_input! { $f $f_args [] $($in)* }
    }
}

macro_rules! _normalize_input {
    // Finish.
    ( $f:ident $f_args:tt $out:tt ) => {
        $f! { $f_args $out }
    };

    // Structs.
    ( $f:ident $f_args:tt [$($out:tt)*]
      $(#[$($meta:tt)*])*
      pub struct $name:tt $([$($params:tt)*])? {$($body:tt)*}
      $($rest:tt)*
    ) => {
        _normalize_input! { $f $f_args
        [$($out)*
        $(#[$($meta)*])*
        pub struct $name [$($($params)*)?] {$($body)*}
        ] $($rest)* }
    };

    // Any token.
    ( $f:ident $f_args:tt [$($out:tt)*] $in:tt $($rest:tt)* ) => {
        _normalize_input! { $f $f_args [$($out)* $in] $($rest)* }
    };
}



// ====================
// === UniformScope ===
// ====================


shared! { UniformScope

#[derive(Clone,Debug)]
pub struct UniformScopeData {
    map    : HashMap<String,AnyUniform>,
    logger : Logger,
}

impl {
    pub fn new(logger: Logger) -> Self {
        let map = default();
        Self {map,logger}
    }

    pub fn get<Name:Str>(&self, name:Name) -> Option<AnyUniform> {
        self.map.get(name.as_ref()).cloned()
    }

    pub fn contains<Name:Str>(&self, name:Name) -> bool {
        self.map.contains_key(name.as_ref())
    }

    pub fn add<Name:Str, Value:UniformValue>
    (&mut self, name:Name, value:Value) -> Option<Uniform<Value>> {
        self.add_or_else(name,value,|t| Some(t), |_| None)
    }

    pub fn add_or_panic<Name:Str, Value:UniformValue>
    (&mut self, name:Name, value:Value) -> Uniform<Value> {
        self.add_or_else(name,value,|t|{t},|name| {
            panic!("Trying to override uniform '{}'.", name.as_ref())
        })
    }
}}

impl UniformScopeData {
    pub fn add_or_else<Name:Str, Value:UniformValue, Ok:Fn(Uniform<Value>)->T, Fail:Fn(Name)->T, T>
    (&mut self, name:Name, value:Value, ok:Ok, fail:Fail) -> T {
        if self.map.contains_key(name.as_ref()) { fail(name) } else {
            let uniform     = Uniform::new(value);
            let any_uniform = uniform.clone().into();
            self.map.insert(name.into(),any_uniform);
            ok(uniform)
        }
    }
}


pub trait UniformValue = GpuData where
    AnyUniform : From<Uniform<Self>>,
    Context    : ContextUniformOps<Self>;




// ===================
// === UniformData ===
// ===================

shared! { Uniform

#[derive(Clone,Debug)]
pub struct UniformData<Value> {
    value: Value,
    dirty: bool,
}

impl<Value:UniformValue> {
    pub fn new(value:Value) -> Self {
        let dirty = false;
        Self {value,dirty}
    }

    pub fn set(&mut self, value:Value) {
        self.set_dirty();
        self.value = value;
    }

    pub fn modify<F:FnOnce(&mut Value)>(&mut self, f:F) {
        self.set_dirty();
        f(&mut self.value);
    }

    pub fn check_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn unset_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn upload(&self, context:&Context, location:&WebGlUniformLocation) {
        context.set_uniform(location,&self.value);
    }
}}





// ==================
// === AnyUniform ===
// ==================

#[enum_dispatch(AnyUniformOps)]
#[derive(Clone,Debug)]
pub enum AnyUniform {
    Variant_i32           (Uniform<i32>),
    Variant_f32           (Uniform<f32>),
    Variant_Vector3_of_f32(Uniform<Vector3<f32>>),
    Variant_Matrix4_of_f32(Uniform<Matrix4<f32>>)
}

#[enum_dispatch]
pub trait AnyUniformOps {
    fn upload(&self, context:&Context, location:&WebGlUniformLocation);
}
