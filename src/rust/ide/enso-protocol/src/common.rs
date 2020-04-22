//! Commons of JSON-RPC-based Enso services.

/// Time in UTC time zone represented as ISO-8601 string.
pub type UTCDateTime = chrono::DateTime<chrono::FixedOffset>;



// ====================
// === Helper macro ===
// ====================

/// Macro that generates a asynchronous method making relevant RPC call to the
/// server. First three args is the name appropriately in CamelCase,
/// snake_case, camelCase. Then goes the function signature, in form of
/// `(arg:Arg) -> Ret`.
///
/// Macro generates:
/// * a method in Client named `snake_case` that takes `(arg:Arg)` and returns
/// `Future<Ret>`.
/// * a structure named `CamelCase` that stores function arguments as fields and
///   its JSON serialization conforms to JSON-RPC (yielding `method` and
///   `params` fields).
/// * `snakeCase` is the name of the remote method.
#[macro_export]
macro_rules! make_rpc_method {
    ( $name_typename:ident
      $name:ident
      $name_ext:ident
      ($($arg:ident : $type:ty),* $(,)?) -> $out:ty   ) => {
    paste::item! {
        impl ClientData {
            /// Remote call to the method on the File Manager Server.
            pub fn $name
            (&mut self, $($arg:$type),*) -> impl Future<Output=Result<$out>> {
                let input = [<$name_typename Input>] { $($arg:$arg),* };
                self.handler.open_request(input)
            }
        }

        impl Client {
            /// Remote call to the method on the File Manager Server.
            pub fn $name
            (&self, $($arg:$type),*) -> impl Future<Output=Result<$out>> {
                self.with_borrowed(|client| client.$name  ($($arg),*))
            }
        }

        /// Structure transporting method arguments.
        #[derive(Serialize,Deserialize,Debug,PartialEq)]
        #[serde(rename_all = "camelCase")]
        struct [<$name_typename Input>] {
            $($arg : $type),*
        }

        impl json_rpc::RemoteMethodCall for [<$name_typename Input>] {
            const NAME:&'static str = stringify!($name_ext);
            type Returned = $out;
        }
    }}
}

#[macro_export]
macro_rules! make_param_map {
    (,$ty:ty) => {
        $ty
    };
    (,$($ty:ty),+) => {
        ($($ty),+)
    }
}

#[macro_export]
macro_rules! make_arg {
    ($name:ident) => {
        $name
    };
    ($($name:ident),+) => {
        ($($name),+)
    }
}

/// This macro reads an impl block and generates asynchronous methods for RPCs. Each method
/// should be signed with `CamelCase` and `camelCase` attributes. e.g.:
/// ```rust,compile_fail
/// make_rpc_method!{
///     impl {
///         #[CamelCase=CallMePlease,camelCase=callMePlease]
///         fn call_me_please(&self, my_number_is:String) -> ();
///     }
/// }
/// ```
///
/// This macro generates both `Client`, with the actual `RPC` methods and `Mock`, with mocked
/// method with return types setup by:
/// ```rust,compile_fail
///     fn set_call_me_please_result
///     (&mut self, my_number_is:String,result:json_rpc::api::Result<()>) { /* impl */ }
/// ```
#[macro_export]
macro_rules! make_rpc_methods {
    (
    $(#[doc = $impl_doc:expr])+
    impl {
        $(
        $(#[doc = $doc:expr])+
        #[CamelCase=$CamelCase:ident,camelCase=$camelCase:ident]
        fn $method:ident(&self $(,$param_name:ident:$param_ty:ty)+) -> $result:ty;
        )*
    }) => {
        $(#[doc = $impl_doc])+
        pub trait Interface {
            $(
            $(#[doc = $doc])+
            fn $method(&self $(,$param_name:$param_ty)+) -> std::pin::Pin<Box<dyn Future<Output=Result<$result>>>>;
            )*
        }

        $(make_rpc_method!($CamelCase $method $camelCase ($($param_name:$param_ty),+) -> $result);)*

        paste::item!{
            /// Mock used for tests.
            #[derive(Debug,Default)]
            pub struct Mock {
                $([<$method _result>] : RefCell<HashMap<make_param_map!($(,$param_ty)+),Result<$result>>>,)*
            }

            impl Interface for Mock {
                $(
                    fn $method(&self $(,$param_name:$param_ty)+) -> std::pin::Pin<Box<dyn Future<Output=Result<$result>>>> {
                        let mut result = self.[<$method _result>].borrow_mut();
                        let result     = result.remove(&make_arg!($($param_name),+)).unwrap();
                        Box::pin(async move { result })
                    }
                )*
            }

            impl Mock {
                $(
                    /// Sets `$method`'s result to be returned when it is called.
                    pub fn [<set_ $method _result>]
                    (&self $(,$param_name:$param_ty)+, result:Result<$result>) {
                        self.[<$method _result>].borrow_mut().insert(make_arg!($($param_name),+),result);
                    }
                )*
            }

            shared! { Client
                $(#[doc = $impl_doc])+
                #[derive(Debug)]
                pub struct ClientData {
                    /// JSON-RPC protocol handler.
                    handler : Handler<Notification>,
                }

                impl {
                    /// Create a new Project Manager client that will use given transport.
                    pub fn new(transport:impl json_rpc::Transport + 'static) -> Self {
                        let handler = Handler::new(transport);
                        Self { handler }
                    }

                    /// Asynchronous event stream with notification and errors.
                    ///
                    /// On a repeated call, previous stream is closed.
                    pub fn events(&mut self) -> impl Stream<Item = Event> {
                        self.handler.handler_event_stream()
                    }

                    /// Returns a future that performs any background, asynchronous work needed
                    /// for this Client to correctly work. Should be continually run while the
                    /// `Client` is used. Will end once `Client` is dropped.
                    pub fn runner(&mut self) -> impl Future<Output = ()> {
                        self.handler.runner()
                    }
                }

            }

        }
    }
}
