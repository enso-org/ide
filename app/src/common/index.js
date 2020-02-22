(function webpackUniversalModuleDefinition(root, factory) {
	if(typeof exports === 'object' && typeof module === 'object')
		module.exports = factory();
	else if(typeof define === 'function' && define.amd)
		define([], factory);
	else {
		var a = factory();
		for(var i in a) (typeof exports === 'object' ? exports : root)[i] = a[i];
	}
})(window, function() {
return /******/ (function(modules) { // webpackBootstrap
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId]) {
/******/ 			return installedModules[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			i: moduleId,
/******/ 			l: false,
/******/ 			exports: {}
/******/ 		};
/******/
/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/
/******/ 		// Flag the module as loaded
/******/ 		module.l = true;
/******/
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/
/******/
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = modules;
/******/
/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = installedModules;
/******/
/******/ 	// define getter function for harmony exports
/******/ 	__webpack_require__.d = function(exports, name, getter) {
/******/ 		if(!__webpack_require__.o(exports, name)) {
/******/ 			Object.defineProperty(exports, name, { enumerable: true, get: getter });
/******/ 		}
/******/ 	};
/******/
/******/ 	// define __esModule on exports
/******/ 	__webpack_require__.r = function(exports) {
/******/ 		if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 			Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 		}
/******/ 		Object.defineProperty(exports, '__esModule', { value: true });
/******/ 	};
/******/
/******/ 	// create a fake namespace object
/******/ 	// mode & 1: value is a module id, require it
/******/ 	// mode & 2: merge all properties of value into the ns
/******/ 	// mode & 4: return value when already ns object
/******/ 	// mode & 8|1: behave like require
/******/ 	__webpack_require__.t = function(value, mode) {
/******/ 		if(mode & 1) value = __webpack_require__(value);
/******/ 		if(mode & 8) return value;
/******/ 		if((mode & 4) && typeof value === 'object' && value && value.__esModule) return value;
/******/ 		var ns = Object.create(null);
/******/ 		__webpack_require__.r(ns);
/******/ 		Object.defineProperty(ns, 'default', { enumerable: true, value: value });
/******/ 		if(mode & 2 && typeof value != 'string') for(var key in value) __webpack_require__.d(ns, key, function(key) { return value[key]; }.bind(null, key));
/******/ 		return ns;
/******/ 	};
/******/
/******/ 	// getDefaultExport function for compatibility with non-harmony modules
/******/ 	__webpack_require__.n = function(module) {
/******/ 		var getter = module && module.__esModule ?
/******/ 			function getDefault() { return module['default']; } :
/******/ 			function getModuleExports() { return module; };
/******/ 		__webpack_require__.d(getter, 'a', getter);
/******/ 		return getter;
/******/ 	};
/******/
/******/ 	// Object.prototype.hasOwnProperty.call
/******/ 	__webpack_require__.o = function(object, property) { return Object.prototype.hasOwnProperty.call(object, property); };
/******/
/******/ 	// __webpack_public_path__
/******/ 	__webpack_require__.p = "";
/******/
/******/
/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(__webpack_require__.s = "./src/index.js");
/******/ })
/************************************************************************/
/******/ ({

/***/ "./src/animation.js":
/*!**************************!*\
  !*** ./src/animation.js ***!
  \**************************/
/*! exports provided: ease_in_out_cubic, ease_in_out_quad, ease_out_quart */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"ease_in_out_cubic\", function() { return ease_in_out_cubic; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"ease_in_out_quad\", function() { return ease_in_out_quad; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"ease_out_quart\", function() { return ease_out_quart; });\n/// This module defines a simple set of animation utils. Follow the link to learn more:\n/// https://easings.net/en\n\n\n\n// =================\n// === Animation ===\n// =================\n\nfunction ease_in_out_cubic(t) {\n    return t<.5 ? 4*t*t*t : 1 - (-2*t+2) * (-2*t+2) * (-2*t+2) / 2\n}\n\nfunction ease_in_out_quad(t) {\n    return t<.5 ? 2*t*t : 1 - (-2*t+2)*(-2*t+2) / 2\n}\n\nfunction ease_out_quart(t) {\n    return 1-(--t)*t*t*t\n}\n\n\n//# sourceURL=webpack:///./src/animation.js?");

/***/ }),

/***/ "./src/html_utils.js":
/*!***************************!*\
  !*** ./src/html_utils.js ***!
  \***************************/
/*! exports provided: remove_node, new_top_level_div, log_group_collapsed */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"remove_node\", function() { return remove_node; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"new_top_level_div\", function() { return new_top_level_div; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"log_group_collapsed\", function() { return log_group_collapsed; });\n// ==================\n// === HTML Utils ===\n// ==================\n\n/// Remove the given node if it exists.\nfunction remove_node(node) {\n    if (node) {\n        node.parentNode.removeChild(node)\n    }\n}\n\n/// Creates a new top-level div which occupy full size of its parent's space.\nfunction new_top_level_div() {\n    let node = document.createElement('div')\n    node.style.width  = '100%'\n    node.style.height = '100%'\n    document.body.appendChild(node)\n    return node\n}\n\n/// Log subsequent messages in a group.\nasync function log_group_collapsed(msg,f) {\n    console.groupCollapsed(msg)\n    let out\n    try {\n        out = await f()\n    } catch (error) {\n        console.groupEnd()\n        throw error\n    }\n    console.groupEnd()\n    return out\n}\n\n\n//# sourceURL=webpack:///./src/html_utils.js?");

/***/ }),

/***/ "./src/index.js":
/*!**********************!*\
  !*** ./src/index.js ***!
  \**********************/
/*! no exports provided */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _loader__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./loader */ \"./src/loader.js\");\n/* harmony import */ var _html_utils__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./html_utils */ \"./src/html_utils.js\");\n/// This module is responsible for loading the WASM binary, its dependencies, and providing the\n/// user with a visual representation of this process (welcome screen). It also implements a view\n/// allowing to choose a debug rendering test from.\n\n\n\n\n\n\n// ========================\n// === Content Download ===\n// ========================\n\nlet incorrect_mime_type_warning = `\n'WebAssembly.instantiateStreaming' failed because your server does not serve wasm with\n'application/wasm' MIME type. Falling back to 'WebAssembly.instantiate' which is slower.\n`\n\nfunction wasm_instantiate_streaming(resource,imports) {\n    return WebAssembly.instantiateStreaming(resource,imports).catch(e => {\n        return wasm_fetch.then(r => {\n            if (r.headers.get('Content-Type') != 'application/wasm') {\n                console.warn(`${incorrect_mime_type_warning} Original error:\\n`, e)\n                return r.arrayBuffer()\n            } else {\n                throw(\"Server not configured to serve WASM with 'application/wasm' mime type.\")\n            }\n        }).then(bytes => WebAssembly.instantiate(bytes,imports))\n    })\n}\n\n\n/// Downloads the WASM binary and its dependencies. Displays loading progress bar unless provided\n/// with `{no_loader:true}` option.\nasync function download_content(cfg) {\n    let wasm_glue_fetch = await fetch('/assets/wasm_imports.js')\n    let wasm_fetch      = await fetch('/assets/gui.wasm')\n    let loader          = new _loader__WEBPACK_IMPORTED_MODULE_0__[\"Loader\"]([wasm_glue_fetch,wasm_fetch],cfg)\n\n    loader.done.then(() => {\n        console.groupEnd()\n        console.log(\"Download finished. Finishing WASM compilation.\")\n    })\n\n    let download_size = loader.show_total_bytes();\n    let download_info = `Downloading WASM binary and its dependencies (${download_size}).`\n    let wasm_loader   = _html_utils__WEBPACK_IMPORTED_MODULE_1__[\"log_group_collapsed\"](download_info, async () => {\n        let wasm_glue_js = await wasm_glue_fetch.text()\n        let wasm_glue    = Function(\"let exports = {};\" + wasm_glue_js + \"; return exports\")()\n        let imports      = wasm_glue.wasm_imports()\n        console.log(\"WASM dependencies loaded.\")\n        console.log(\"Starting online WASM compilation.\")\n        let wasm_loader       = await wasm_instantiate_streaming(wasm_fetch,imports)\n        wasm_loader.wasm_glue = wasm_glue\n        return wasm_loader\n    })\n\n    let wasm = await wasm_loader.then(({instance,module,wasm_glue}) => {\n        let wasm = instance.exports;\n        wasm_glue.after_load(wasm,module)\n        return wasm\n    });\n    console.log(\"WASM Compiled.\")\n\n    await loader.initialized\n    return {wasm,loader}\n}\n\n\n\n// ====================\n// === Debug Screen ===\n// ====================\n\n/// The name of the main scene in the WASM binary.\nlet main_scene_name = 'ide'\n\n/// Prefix name of each scene defined in the WASM binary.\nlet wasm_fn_pfx = \"run_example_\"\n\n\n/// Displays a debug screen which allows the user to run one of predefined debug examples.\nfunction show_debug_screen(wasm,msg) {\n    let names = []\n    for (let fn of Object.getOwnPropertyNames(wasm)) {\n        if (fn.startsWith(wasm_fn_pfx)) {\n            let name = fn.replace(wasm_fn_pfx,\"\")\n            names.push(name)\n        }\n    }\n\n    if(msg===\"\" || msg===null || msg===undefined) { msg = \"\" }\n    let debug_screen_div = _html_utils__WEBPACK_IMPORTED_MODULE_1__[\"new_top_level_div\"]()\n    let newDiv     = document.createElement(\"div\")\n    let newContent = document.createTextNode(msg + \"Choose an example:\")\n    let currentDiv = document.getElementById(\"app\")\n    let ul         = document.createElement('ul')\n    newDiv.appendChild(newContent)\n    debug_screen_div.appendChild(newDiv)\n    newDiv.appendChild(ul)\n\n    for (let name of names) {\n        let li       = document.createElement('li')\n        let a        = document.createElement('a')\n        let linkText = document.createTextNode(name)\n        ul.appendChild(li)\n        a.appendChild(linkText)\n        a.title   = name\n        a.href    = \"javascript:{}\"\n        a.onclick = () => {\n            _html_utils__WEBPACK_IMPORTED_MODULE_1__[\"remove_node\"](debug_screen_div)\n            let fn_name = wasm_fn_pfx + name\n            let fn = wasm[fn_name]\n            fn()\n        }\n        li.appendChild(a)\n    }\n}\n\n\n\n// ========================\n// === Main Entry Point ===\n// ========================\n\n/// Main entry point. Loads WASM, initializes it, chooses the scene to run.\nasync function main() {\n    let target = window.location.href.split('/')\n    target.splice(0,3)\n\n    let debug_mode    = target[0] == \"debug\"\n    let debug_target  = target[1]\n    let no_loader     = debug_mode && debug_target\n    let {wasm,loader} = await download_content({no_loader})\n\n    if (debug_mode) {\n        loader.destroy()\n        if (debug_target) {\n            let fn_name = wasm_fn_pfx + debug_target\n            let fn      = wasm[fn_name]\n            if (fn) { fn() } else {\n                show_debug_screen(wasm,\"WASM function '\" + fn_name + \"' not found! \")\n            }\n        } else {\n            show_debug_screen(wasm)\n        }\n    } else {\n        wasm[wasm_fn_pfx + main_scene_name]()\n    }\n}\n\nmain()\n\n\n//# sourceURL=webpack:///./src/index.js?");

/***/ }),

/***/ "./src/loader.js":
/*!***********************!*\
  !*** ./src/loader.js ***!
  \***********************/
/*! exports provided: ProgressIndicator, Loader */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"ProgressIndicator\", function() { return ProgressIndicator; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"Loader\", function() { return Loader; });\n/* harmony import */ var _animation__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./animation */ \"./src/animation.js\");\n/* harmony import */ var _html_utils__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./html_utils */ \"./src/html_utils.js\");\n/* harmony import */ var _math__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./math */ \"./src/math.js\");\n/* harmony import */ var _svg__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./svg */ \"./src/svg.js\");\n\n\n\n\n\n\n\n// =========================\n// === ProgressIndicator ===\n// =========================\n\nlet bg_color     = document.body.style.backgroundColor\nlet loader_color = \"#303030\"\n\n/// Visual representation of the loader.\nclass ProgressIndicator {\n    constructor(cfg) {\n        this.dom                = _html_utils__WEBPACK_IMPORTED_MODULE_1__[\"new_top_level_div\"]()\n        this.dom.id             = 'loader'\n        this.dom.style.position = 'absolute'\n        this.dom.style.zIndex   = -1\n\n        let center = document.createElement('div')\n        center.style.width          = '100%'\n        center.style.height         = '100%'\n        center.style.display        = 'flex'\n        center.style.justifyContent = 'center'\n        center.style.alignItems     = 'center'\n        this.dom.appendChild(center)\n\n        let progress_bar_svg   = this.init_svg()\n        let progress_bar       = document.createElement('div')\n        progress_bar.innerHTML = progress_bar_svg\n        center.appendChild(progress_bar)\n\n        this.progress_indicator        = document.getElementById(\"progress_indicator\")\n        this.progress_indicator_mask   = document.getElementById(\"progress_indicator_mask\")\n        this.progress_indicator_corner = document.getElementById(\"progress_indicator_corner\")\n\n        this.set(0)\n        this.set_opacity(0)\n\n        if(!cfg.no_loader) {\n            this.initialized = this.animate_show()\n        } else {\n            this.initialized = new Promise((resolve) => {resolve()})\n        }\n        this.animate_rotation()\n        this.destroyed = false\n    }\n\n    /// Initializes the SVG view.\n    init_svg() {\n        let width        = 128\n        let height       = 128\n        let alpha        = 0.9\n        let inner_radius = 48\n        let outer_radius = 60\n        let mid_radius = (inner_radius + outer_radius) / 2\n        let bar_width = outer_radius - inner_radius\n\n        return _svg__WEBPACK_IMPORTED_MODULE_3__[\"new_svg\"](width,height,`\n            <defs>\n                <g id=\"progress_bar\">\n                    <circle fill=\"${loader_color}\" r=\"${outer_radius}\"                               />\n                    <circle fill=\"${bg_color}\"     r=\"${inner_radius}\"                               />\n                    <path   fill=\"${bg_color}\"     opacity=\"${alpha}\" id=\"progress_indicator_mask\"   />\n                    <circle fill=\"${loader_color}\" r=\"${bar_width/2}\" id=\"progress_indicator_corner\" />\n                    <circle fill=\"${loader_color}\" r=\"${bar_width/2}\" cy=\"-${mid_radius}\"            />\n                </g>\n            </defs>\n            <g transform=\"translate(${width/2},${height/2})\">\n                <g transform=\"rotate(0,0,0)\" id=\"progress_indicator\">\n                    <use xlink:href=\"#progress_bar\"></use>\n                </g>\n            </g>\n        `)\n    }\n\n    /// Destroys the component. Removes it from the stage and destroys attached callbacks.\n    destroy() {\n        _html_utils__WEBPACK_IMPORTED_MODULE_1__[\"remove_node\"](this.dom)\n        this.destroyed = true\n    }\n\n    /// Set the value of the loader [0..1].\n    set(value) {\n        let min_angle  = 0\n        let max_angle  = 359\n        let angle_span = max_angle - min_angle\n        let mask_angle = (1-value)*angle_span - min_angle\n        let corner_pos = _math__WEBPACK_IMPORTED_MODULE_2__[\"polar_to_cartesian\"](54, -mask_angle)\n        this.progress_indicator_mask.setAttribute(\"d\", _svg__WEBPACK_IMPORTED_MODULE_3__[\"arc\"](128, -mask_angle))\n        this.progress_indicator_corner.setAttribute(\"cx\", corner_pos.x)\n        this.progress_indicator_corner.setAttribute(\"cy\", corner_pos.y)\n    }\n\n    /// Set the opacity of the loader.\n    set_opacity(val) {\n        this.progress_indicator.setAttribute(\"opacity\",val)\n    }\n\n    /// Set the rotation of the loader (angles).\n    set_rotation(val) {\n        this.progress_indicator.setAttribute(\"transform\",`rotate(${val},0,0)`)\n    }\n\n    /// Start show animation. It is used after the loader is created.\n    animate_show() {\n        let indicator = this\n        return new Promise(function(resolve, reject) {\n            let alpha = 0\n            function show_step() {\n                if (alpha > 1) { alpha = 1 }\n                indicator.set_opacity(_animation__WEBPACK_IMPORTED_MODULE_0__[\"ease_in_out_quad\"](alpha))\n                alpha += 0.02\n                if (alpha < 1) {\n                    window.requestAnimationFrame(show_step)\n                } else {\n                     console.log(\"SHOW END\")\n                     resolve()\n                }\n            }\n            window.requestAnimationFrame(show_step)\n        })\n    }\n\n    /// Start the spinning animation.\n    animate_rotation() {\n        let indicator = this\n        let rotation  = 0\n        function rotate_step(timestamp) {\n            indicator.set_rotation(rotation)\n            rotation += 6\n            if (!indicator.destroyed) {\n                window.requestAnimationFrame(rotate_step)\n            }\n        }\n        window.requestAnimationFrame(rotate_step)\n    }\n}\n\n\n\n// ==============\n// === Loader ===\n// ==============\n\n/// The main loader class. It connects to the provided fetch responses and tracks their status.\nclass Loader {\n    constructor(resources, cfg) {\n        this.indicator         = new ProgressIndicator(cfg)\n        this.total_bytes       = 0\n        this.received_bytes    = 0\n        this.download_speed    = 0\n        this.last_receive_time = performance.now()\n        this.initialized       = this.indicator.initialized\n\n        let self          = this\n        this.done_resolve = null\n        this.done         = new Promise((resolve) => {self.done_resolve = resolve})\n\n        for (let resource of resources) {\n            this.total_bytes += parseInt(resource.headers.get('Content-Length'))\n            resource.clone().body.pipeTo(this.input_stream())\n        }\n\n        if (Number.isNaN(this.total_bytes)) {\n            console.error(\"Loader error. Server is not configured to send the 'Content-Length' metadata.\")\n            this.total_bytes = 0\n        }\n    }\n\n    /// The current loading progress [0..1].\n    value() {\n        if (this.total_bytes == 0) {\n            return 0.3\n        } else {\n            return this.received_bytes / this.total_bytes\n        }\n    }\n\n    /// Returns true if the loader finished.\n    is_done() {\n        return this.received_bytes == this.total_bytes\n    }\n\n    /// Removes the loader with it's dom element.\n    destroy() {\n        this.indicator.destroy()\n    }\n\n    /// Callback run on every new received byte stream.\n    on_receive(new_bytes) {\n        this.received_bytes += new_bytes\n        let time      = performance.now()\n        let time_diff = time - this.last_receive_time\n        this.download_speed = new_bytes / time_diff\n        this.last_receive_time = time\n\n        let percent  = this.show_percentage_value()\n        let speed    = this.show_download_speed()\n        let received = this.show_received_bytes()\n        console.log(`${percent}% (${received}) (${speed}).`)\n\n        this.indicator.set(this.value())\n        if (this.is_done()) { this.done_resolve() }\n    }\n\n    /// Download percentage value.\n    show_percentage_value() {\n        return Math.round(100 * this.value())\n    }\n\n    /// Download total size value.\n    show_total_bytes() {\n        return `${_math__WEBPACK_IMPORTED_MODULE_2__[\"format_mb\"](this.total_bytes)} MB`\n    }\n\n    /// Download received bytes value.\n    show_received_bytes() {\n        return `${_math__WEBPACK_IMPORTED_MODULE_2__[\"format_mb\"](this.received_bytes)} MB`\n    }\n\n    /// Download speed value.\n    show_download_speed() {\n        return `${_math__WEBPACK_IMPORTED_MODULE_2__[\"format_mb\"](1000 * this.download_speed)} MB/s`\n    }\n\n    /// Internal function for attaching new fetch responses.\n    input_stream() {\n        let loader = this\n        return new WritableStream({\n             write(t) {\n                 loader.on_receive(t.length)\n             }\n        })\n    }\n}\n\n\n//# sourceURL=webpack:///./src/loader.js?");

/***/ }),

/***/ "./src/math.js":
/*!*********************!*\
  !*** ./src/math.js ***!
  \*********************/
/*! exports provided: polar_to_cartesian, format_mb */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"polar_to_cartesian\", function() { return polar_to_cartesian; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"format_mb\", function() { return format_mb; });\n/// This module defines a common math operations.\n\n\n\n// ============\n// === Math ===\n// ============\n\n/// Converts the polar coordinates to cartesian ones.\nfunction polar_to_cartesian(radius, angle_degrees) {\n    let angle = (angle_degrees-90) * Math.PI / 180.0\n    return {\n        x : radius * Math.cos(angle),\n        y : radius * Math.sin(angle)\n    }\n}\n\n/// Format bytes as megabytes with a single precision number.\nfunction format_mb(bytes) {\n   return Math.round(10 * bytes / (1024 * 1024)) / 10\n}\n\n\n//# sourceURL=webpack:///./src/math.js?");

/***/ }),

/***/ "./src/svg.js":
/*!********************!*\
  !*** ./src/svg.js ***!
  \********************/
/*! exports provided: new_svg, arc */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"new_svg\", function() { return new_svg; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"arc\", function() { return arc; });\n/* harmony import */ var _math__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./math */ \"./src/math.js\");\n/// This module defines a set of utils for generating and modifying the SVG images.\n\n\n\n\n\n// ===========\n// === SVG ===\n// ===========\n\n/// Defines a new SVG with the provided source.\nfunction new_svg(width, height, str) {\n    return `\n    <svg version=\"1.1\" baseProfile=\"full\" xmlns=\"http://www.w3.org/2000/svg\"\n         xmlns:xlink=\"http://www.w3.org/1999/xlink\"\n         height=\"${height}\" width=\"${width}\" viewBox=\"0 0 ${height} ${width}\">\n    ${str}\n    </svg>`\n}\n\n/// Returns SVG code for an arc with a defined radius and angle.\nfunction arc(radius, end_angle){\n    let start_angle = 0\n    if (end_angle < 0) {\n        start_angle = end_angle\n        end_angle   = 0\n    }\n    let start       = _math__WEBPACK_IMPORTED_MODULE_0__[\"polar_to_cartesian\"](radius, end_angle)\n    let end         = _math__WEBPACK_IMPORTED_MODULE_0__[\"polar_to_cartesian\"](radius, start_angle)\n    let large_arc   = end_angle - start_angle <= 180 ? \"0\" : \"1\"\n    return `M 0 0 L ${start.x} ${start.y} A ${radius} ${radius} 0 ${large_arc} 0 ${end.x} ${end.y}`\n}\n\n\n//# sourceURL=webpack:///./src/svg.js?");

/***/ })

/******/ });
});