//! Client library for the JSON-RPC-based File Manager service.

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
        impl Client {
            /// Remote call to the method on the File Manager Server.
            pub fn $name
            (&mut self, $($arg:$type),*) -> impl Future<Output=Result<$out>> {
                let input = [<$name_typename Input>] { $($arg:$arg),* };
                self.handler.open_request(input)
            }
        }

        impl Handle {
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
