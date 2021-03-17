//! The whole content of this file is copied from `js_sys` just to modify `new_with_args and `new_no_args` such that they catch syntax errors.
//! This is supposed to be temporary solution.
//! I opened a GitHub issue suggesting that this will be included in wasm-bindgen (https://github.com/rustwasm/wasm-bindgen/issues/2496).


use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, is_type_of = JsValue::is_function)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Function;

    /// The `Function` constructor creates a new `Function` object. Calling the
    /// constructor directly can create functions dynamically, but suffers from
    /// security and similar (but far less significant) performance issues
    /// similar to `eval`. However, unlike `eval`, the `Function` constructor
    /// allows executing code in the global scope, prompting better programming
    /// habits and allowing for more efficient code minification.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function)
    #[wasm_bindgen(constructor, catch)]
    pub fn new_with_args(args: &str, body: &str) -> Result<Function, JsValue>;

    /// The `Function` constructor creates a new `Function` object. Calling the
    /// constructor directly can create functions dynamically, but suffers from
    /// security and similar (but far less significant) performance issues
    /// similar to `eval`. However, unlike `eval`, the `Function` constructor
    /// allows executing code in the global scope, prompting better programming
    /// habits and allowing for more efficient code minification.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function)
    #[wasm_bindgen(constructor, catch)]
    pub fn new_no_args(body: &str) -> Result<Function, JsValue>;

    /// The `apply()` method calls a function with a given this value, and arguments provided as an array
    /// (or an array-like object).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/apply)
    #[wasm_bindgen(method, catch)]
    pub fn apply(this: &Function, context: &JsValue, args: &Array) -> Result<JsValue, JsValue>;

    /// The `call()` method calls a function with a given this value and
    /// arguments provided individually.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call)
    #[wasm_bindgen(method, catch, js_name = call)]
    pub fn call0(this: &Function, context: &JsValue) -> Result<JsValue, JsValue>;

    /// The `call()` method calls a function with a given this value and
    /// arguments provided individually.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call)
    #[wasm_bindgen(method, catch, js_name = call)]
    pub fn call1(this: &Function, context: &JsValue, arg1: &JsValue) -> Result<JsValue, JsValue>;

    /// The `call()` method calls a function with a given this value and
    /// arguments provided individually.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call)
    #[wasm_bindgen(method, catch, js_name = call)]
    pub fn call2(
        this: &Function,
        context: &JsValue,
        arg1: &JsValue,
        arg2: &JsValue,
    ) -> Result<JsValue, JsValue>;

    /// The `call()` method calls a function with a given this value and
    /// arguments provided individually.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call)
    #[wasm_bindgen(method, catch, js_name = call)]
    pub fn call3(
        this: &Function,
        context: &JsValue,
        arg1: &JsValue,
        arg2: &JsValue,
        arg3: &JsValue,
    ) -> Result<JsValue, JsValue>;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind(this: &Function, context: &JsValue) -> Function;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind0(this: &Function, context: &JsValue) -> Function;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind1(this: &Function, context: &JsValue, arg1: &JsValue) -> Function;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind2(this: &Function, context: &JsValue, arg1: &JsValue, arg2: &JsValue) -> Function;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind3(
        this: &Function,
        context: &JsValue,
        arg1: &JsValue,
        arg2: &JsValue,
        arg3: &JsValue,
    ) -> Function;

    /// The length property indicates the number of arguments expected by the function.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/length)
    #[wasm_bindgen(method, getter, structural)]
    pub fn length(this: &Function) -> u32;

    /// A Function object's read-only name property indicates the function's
    /// name as specified when it was created or "anonymous" for functions
    /// created anonymously.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/name)
    #[wasm_bindgen(method, getter, structural)]
    pub fn name(this: &Function) -> JsString;

    /// The `toString()` method returns a string representing the source code of the function.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/toString)
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Function) -> JsString;
}

impl Function {
    /// Returns the `Function` value of this JS value if it's an instance of a
    /// function.
    ///
    /// If this JS value is not an instance of a function then this returns
    /// `None`.
    #[deprecated(note = "recommended to use dyn_ref instead which is now equivalent")]
    pub fn try_from(val: &JsValue) -> Option<&Function> {
        val.dyn_ref()
    }
}