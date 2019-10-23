#[macro_export]
macro_rules! closure {
    ($name:ident 
        <$($param:ident : $param_type:ty),*> 
        ($($arg:ident   : $arg_type:ty),*)
        |$($larg:ident  : $larg_type:ty),*|
        $body:tt
    ) => { 
        closure!( $name<$($param:$param_type),*>
            ($($arg:$arg_type),*)
            ($($larg:$larg_type)*)
            $body
        );
    };
    ($name:ident 
        <$($param:ident : $param_type:ty),*> 
        ($($arg:ident   : $arg_type:ty),*)
        || $body:tt) => {
        closure!($name<$($param:$param_type),*>($($arg:$arg_type),*)()$body);
    };
    ($name:ident 
        <$($param:ident : $param_type:ty),*> 
        ($($arg:ident   : $arg_type:ty),*)
        ($($larg:ident  : $larg_type:ty),*)
        $body:tt
    ) => { paste::item! {
        pub type [<Closure_ $name>]<$($param),*> = 
            impl Fn($($larg_type),*) + Clone;
        pub fn $name<$($param:$param_type),*>
        ($($arg:$arg_type),*) -> [<Closure_ $name>]<$($param),*> {
            move |$($larg),*| $body
        }
    }};
}