//! FRP utilities for defining application components.

/// Generate a set of structures allowing for nice management of FRP inputs, outputs, and commands.
///
/// Given the definition:
///
/// ```compile_fail
/// define_endpoints! {
///     Commands { Commands }
///     Input {
///         input1 (f32),
///         input2 (),
///     }
///     Output {
///         output1 (String),
///         output2 (),
///     }
/// }
/// ```
///
/// There would be generated a bunch of structures, the main structure contains FRP network and
/// another struct with all the endpoints, to which it dereferences to:
///
/// ```compile_fail
/// #[derive(Debug,Clone,CloneRef)]
/// pub struct Frp {
///     pub network : frp::Network,
///     output      : FrpEndpoints,
/// }
///
/// impl Frp {
///     pub fn new(network:frp::Network, output:FrpEndpoints) -> Self {
///         Self {network,output}
///     }
/// }
///
/// impl Deref for Frp {
///     type Target = FrpEndpoints;
///     fn deref(&self) -> &Self::Target {
///         &self.output
///     }
/// }
/// ```
///
/// The target structure contains all outputs as `frp::Stream` values. As a reminder, you are
/// allowed to read from streams but you are not allowed to write to them, which makes perfect
/// sense in the context of output values.
///
/// ```compile_fail
/// #[derive(Debug, Clone, CloneRef)]
/// #[allow(missing_docs)]
/// pub struct FrpEndpoints {
///     pub input         : FrpInputs,
///     pub(crate) source : FrpOutputsSource,
///     pub output1       : frp::Stream<String>,
///     pub output2       : frp::Stream<>,
/// }
/// impl Deref for FrpEndpoints {
///     type Target = FrpInputs;
///     fn deref(&self) -> &Self::Target {
///         &self.input
///     }
/// }
/// impl FrpEndpoints {
///     pub fn new(network: &frp::Network, input: FrpInputs) -> Self {
///         let source = FrpOutputsSource::new(network);
///         let output1 = source.output1.clone_ref().into();
///         let output2 = source.output2.clone_ref().into();
///         Self { source, input, output1, output2 }
///     }
/// }
/// ```
///
/// However, as the owner of this FRP network, you want to emit values to the output streams.
/// Exactly for this purpose there is `FrpOutputsSource` struct defined, which is private and cannot
/// be accessed from outside of this crate. The structure contains the same FRP endpoints but in a
/// form allowing for both emiting and attaching values (see FRP documentation to learn more):
///
/// ```compile_fail
/// #[derive(Debug,Clone,CloneRef)]
/// pub(crate) struct FrpOutputsSource {
///     output1: frp::Any<String>,
///     output2: frp::Any<>,
/// }
///
/// impl FrpOutputsSource {
///     pub fn new(network: &frp::Network) -> Self {
///         frp::extend! { network
///             output1  <- any(...);
///             output2  <- any(...);
///         }
///         Self {output1,output2}
///     }
/// }
/// ```
///
/// Moreover, the above presented `FrpEndpoints` structure contains the `input` field which
/// describes all FRP input endpoints (input API of this FRP network). The struct contains all the
/// fields declared in the macro usage in a form of `frp::Source` values (allowing emiting values
/// on demand).
///
/// ```compile_fail
/// #[derive(Debug, Clone, CloneRef)]
/// #[allow(missing_docs)]
/// #[allow(unused_parens)]
/// pub struct FrpInputs {
///     pub command: Commands,
///     pub input1: frp::Source<(   f32   )>,
///     pub input2: frp::Source<()>,
/// }
/// impl Deref for FrpInputs {
///     type Target = Commands;
///     fn deref(&self) -> &Self::Target {
///         &self.command
///     }
/// }
///
/// impl FrpInputs {
///     pub fn new(network: &frp::Network) -> Self {
///         let command = Commands::new(network);
///         frp::extend! { network
///             input1 <- source();
///             input2 <- source();
///         }
///         Self {command,input1,input2}
///     }
/// ```
///
/// Moreover, for each input value, there is a sugar-method generated which allows for nice call
/// syntax. Thanks to that and all the derefs defined above, if your component dereferences to the
/// generated FRP struct (and it should!), then instead of calling
/// `my_comp.frp.input.clear_all.emit(())`, you can just call `my_comp.clear_all()`.
///
/// ```compile_fail
/// impl FrpInputs {
///     #[allow(missing_docs)]
///     pub fn input1(&self, t1:impl IntoParam<f32>) {
///         self.input1.emit(t1);
///     }
///     #[allow(missing_docs)]
///     pub fn input2(&self) {
///         self.input2.emit(());
///     }
/// }
/// ```
///
/// Last thing to note here is that if you declared the usage of commands in the usage of the macro,
/// the `FrpInputs` struct will contain commands as a field and will dereference to it. To learn
/// more how to define commands, see the docs of the `def_command_api` macro and example usages
/// in components defined by using of this API.
#[macro_export]
macro_rules! define_endpoints {
    (
        $(Input  {
            $($(#[doc=$($in_doc :tt)*])*
            $in_field : ident ($($in_field_type : tt)*)),* $(,)?
        })?

        $(Output {
            $($(#[doc=$($out_doc:tt)*])*
            $out_field : ident ($($out_field_type : tt)*)),* $(,)?
        })?
    ) => {
        $crate::define_endpoints! {
            NORMALIZED

            Input  {
                /// Active state setter. You should not need to call it directly. It is meant to be
                /// controlled by the focus manager.
                set_active(bool),
                $($($(#[doc=$($in_doc )*])*
                $in_field ($($in_field_type )*)),*)?
            }

            Output {
                /// Active state checker.
                active(bool),
                $($($(#[doc=$($out_doc)*])*
                $out_field ($($out_field_type)*)),*)?
            }
        }
    };

    (
        NORMALIZED

        Input  {
            $($(#[doc=$($in_doc :tt)*])*
            $in_field : ident ($($in_field_type : tt)*)),* $(,)?
        }

        Output {
            $($(#[doc=$($out_doc:tt)*])*
            $out_field : ident ($($out_field_type : tt)*)),* $(,)?
        }
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
                frp::extend! { network
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
                frp::extend! { network
                    $($out_field <- source.$out_field.sampler();)*
                    source.active <+ input.set_active;
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

        impl $crate::application::command::CommandApi for Frp {
            fn command_api(&self) -> Rc<RefCell<HashMap<String,frp::Source<()>>>> {
                self.command_map.clone()
            }

            fn status_api(&self) -> Rc<RefCell<HashMap<String,frp::Sampler<bool>>>> {
                self.status_map.clone()
            }
        }
    };
}

/// Internal helper of `define_endpoints` macro.
#[macro_export]
macro_rules! build_status_map {
    ($map:ident $field:ident (bool) $frp:expr) => {
        $map.insert(stringify!($field).into(),$frp.clone_ref());
    };
    ($($ts:tt)*) => {}
}

/// Internal helper of `define_endpoints` macro.
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
