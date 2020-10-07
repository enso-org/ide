#[macro_export]
macro_rules! define_endpoints2 {
    (
        Input  { $($(#[doc=$($in_doc :tt)*])* $in_field  : ident ($($in_field_type  : tt)*)),* $(,)? }
        Output { $($(#[doc=$($out_doc:tt)*])* $out_field : ident ($($out_field_type : tt)*)),* $(,)? }
    ) => {
        use enso_frp::IntoParam;

        /// Frp network and endpoints.
        #[derive(Debug,Clone,CloneRef)]
        #[allow(missing_docs)]
        pub struct Frp {
            pub network : frp::Network,
            pub output  : FrpEndpoints,
        }

        impl Frp {
            /// Constructor.
            pub fn new(network:frp::Network, output:FrpEndpoints) -> Self {
                Self {network,output}
            }

            /// Create Frp with network, inputs and outputs.
            pub fn new_network() -> Self {
                let network       = frp::Network::new();
                let frp_inputs    = FrpInputs::new(&network);
                let frp_endpoints = FrpEndpoints::new(&network,frp_inputs.clone_ref());
                Self::new(network,frp_endpoints)
            }
        }

        impl Deref for Frp {
            type Target = FrpEndpoints;
            fn deref(&self) -> &Self::Target {
                &self.output
            }
        }

        /// Frp inputs.
        #[derive(Debug,Clone,CloneRef)]
        #[allow(missing_docs)]
        #[allow(unused_parens)]
        pub struct FrpInputs {
            $( $(#[doc=$($in_doc)*])* pub $in_field : frp::Source<($($in_field_type)*)>),*
        }

        #[allow(unused_parens)]
        impl FrpInputs {
            /// Constructor.
            pub fn new(network:&frp::Network) -> Self {
                frp::extend! { TRACE_ALL network
                    $($in_field <- source();)*
                }
                Self { $($in_field),* }
            }

            $($crate::define_endpoints_emit_alias!{$in_field ($($in_field_type)*)})*
        }

        /// Frp outputs.
        #[derive(Debug,Clone,CloneRef)]
        #[allow(missing_docs)]
        pub struct FrpEndpoints {
            pub input         : FrpInputs,
            pub(crate) source : FrpOutputsSource,
            pub status_map    : Rc<RefCell<HashMap<String,frp::Sampler<bool>>>>,
            pub command_map   : Rc<RefCell<HashMap<String,frp::Source<()>>>>,
            $($(#[doc=$($out_doc)*])* pub $out_field  : frp::Sampler<$($out_field_type)*>),*
        }

        impl Deref for FrpEndpoints {
            type Target = FrpInputs;
            fn deref(&self) -> &Self::Target {
                &self.input
            }
        }

        impl FrpEndpoints {
            /// Constructor.
            pub fn new(network:&frp::Network, input:FrpInputs) -> Self {
                let source = FrpOutputsSource::new(network);
                let mut status_map  : HashMap<String,frp::Sampler<bool>> = default();
                let mut command_map : HashMap<String,frp::Source<()>> = default();
                frp::extend! { TRACE_ALL network
                    $($out_field <- source.$out_field.sampler();)*
                }
                //$(let $out_field : frp::Stream<$($out_field_type)*> = source.$out_field.clone_ref().into();)*
                $($crate::build_status_map!{status_map $out_field ($($out_field_type)*) $out_field })*
                $($crate::build_command_map!{command_map $in_field ($($in_field_type)*) input.$in_field })*
                let status_map = Rc::new(RefCell::new(status_map));
                let command_map = Rc::new(RefCell::new(command_map));
                Self {source,input,status_map,command_map,$($out_field),*}
            }
        }

        /// Frp output setters.
        #[derive(Debug,Clone,CloneRef)]
        pub(crate) struct FrpOutputsSource {
            $($out_field : frp::Any<$($out_field_type)*>),*
        }

        impl FrpOutputsSource {
            /// Constructor.
            pub fn new(network:&frp::Network) -> Self {
                frp::extend! { network
                    $($out_field <- any(...);)*
                }
                Self {$($out_field),*}
            }
        }

        impl application::command::CommandApi2 for Frp {
            fn command_api(&self) -> Rc<RefCell<HashMap<String,frp::Source<()>>>> {
                self.command_map.clone()
            }

            fn status_api(&self) -> Rc<RefCell<HashMap<String,frp::Sampler<bool>>>> {
                self.status_map.clone()
            }
        }
    };
}


#[macro_export]
macro_rules! build_status_map {
    ($map:ident $field:ident (bool) $frp:expr) => {
        $map.insert(stringify!($field).into(),$frp.clone_ref());
    };
    ($($ts:tt)*) => {}
}

#[macro_export]
macro_rules! build_command_map {
    ($map:ident $field:ident () $frp:expr) => {
        $map.insert(stringify!($field).into(),$frp.clone_ref());
    };
    ($($ts:tt)*) => {}
}


/// Defines a method which is an alias to FRP emit method. Used internally by the `define_endpoints`
/// macro.
#[macro_export]
macro_rules! define_endpoints_emit_alias {
    ($field:ident ()) => {
        #[allow(missing_docs)]
        pub fn $field(&self) {
            self.$field.emit(());
        }
    };

    ($field:ident ($t1:ty,$t2:ty)) => {
        #[allow(missing_docs)]
        pub fn $field
        ( &self
        , t1:impl IntoParam<$t1>
        , t2:impl IntoParam<$t2>
        ) {
            let t1 = t1.into_param();
            let t2 = t2.into_param();
            self.$field.emit((t1,t2));
        }
    };

    ($field:ident ($t1:ty,$t2:ty,$t3:ty)) => {
        #[allow(missing_docs)]
        pub fn $field
        ( &self
        , t1:impl IntoParam<$t1>
        , t2:impl IntoParam<$t2>
        , t3:impl IntoParam<$t3>
        ) {
            let t1 = t1.into_param();
            let t2 = t2.into_param();
            let t3 = t3.into_param();
            self.$field.emit((t1,t2,t3));
        }
    };

    ($field:ident ($t1:ty,$t2:ty,$t3:ty,$t4:ty)) => {
        #[allow(missing_docs)]
        pub fn $field
        ( &self
        , t1:impl IntoParam<$t1>
        , t2:impl IntoParam<$t2>
        , t3:impl IntoParam<$t3>
        , t4:impl IntoParam<$t4>
        ) {
            let t1 = t1.into_param();
            let t2 = t2.into_param();
            let t3 = t3.into_param();
            let t4 = t4.into_param();
            self.$field.emit((t1,t2,t3,t4));
        }
    };

    ($field:ident ($t1:ty,$t2:ty,$t3:ty,$t4:ty,$t5:ty)) => {
        #[allow(missing_docs)]
        pub fn $field
        ( &self
        , t1:impl IntoParam<$t1>
        , t2:impl IntoParam<$t2>
        , t3:impl IntoParam<$t3>
        , t4:impl IntoParam<$t4>
        , t5:impl IntoParam<$t5>
        ) {
            let t1 = t1.into_param();
            let t2 = t2.into_param();
            let t3 = t3.into_param();
            let t4 = t4.into_param();
            let t5 = t5.into_param();
            self.$field.emit((t1,t2,t3,t4,t5));
        }
    };

    ($field:ident $t1:ty) => {
        #[allow(missing_docs)]
        pub fn $field(&self,t1:impl IntoParam<$t1>) {
            self.$field.emit(t1);
        }
    };
}
