use crate::prelude::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen;
use web_sys::*;
use std::cmp;
use std::f64;
use js_sys::WebAssembly::Memory;
use js_sys::ArrayBuffer;



// ===============
// === Helpers ===
// ===============

pub fn window() -> web_sys::Window {
    web_sys::window().unwrap_or_else(|| panic!("Cannot access window."))
}

pub fn document() -> web_sys::Document {
    window().document().unwrap_or_else(|| panic!("Cannot access window.document."))
}

pub fn body() -> web_sys::HtmlElement {
    document().body().unwrap_or_else(|| panic!("Cannot access window.document.body."))
}

pub fn performance() -> Performance {
    window().performance().unwrap_or_else(|| panic!("Cannot access window.performance."))
}



// ==============
// === Config ===
// ==============

#[derive(Clone,Debug)]
pub struct Config {
    pub background_color      : String,
    pub label_color_ok        : String,
    pub label_color_warn      : String,
    pub label_color_err       : String,
    pub plot_color_ok         : String,
    pub plot_color_warn       : String,
    pub plot_color_err        : String,
    pub plot_background_color : String,
    pub plot_step_size        : u32,
    pub margin                : u32,
    pub panel_height          : u32,
    pub labels_width          : u32,
    pub results_width         : u32,
    pub plots_width           : u32,
    pub font_size             : u32,
}

#[derive(Clone,Debug)]
pub struct PlotConfig {
    pub background_color      : JsValue,
    pub label_color_ok        : JsValue,
    pub label_color_warn      : JsValue,
    pub label_color_err       : JsValue,
    pub plot_color_ok         : JsValue,
    pub plot_color_warn       : JsValue,
    pub plot_color_err        : JsValue,
    pub plot_background_color : JsValue,
    pub plot_step_size        : f64,
    pub margin                : f64,
    pub panel_height          : f64,
    pub labels_width          : f64,
    pub results_width         : f64,
    pub plots_width           : f64,
    pub font_size             : f64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            background_color      : "#222222".into(),
            label_color_ok        : "#8e939a".into(),
            label_color_warn      : "#ffba18".into(),
            label_color_err       : "#eb3941".into(),
            plot_color_ok         : "#8e939a".into(),
            plot_color_warn       : "#ffba18".into(),
            plot_color_err        : "#eb3941".into(),
            plot_background_color : "#333333".into(),
            plot_step_size        : 1,
            margin                : 4,
            panel_height          : 15,
            labels_width          : 120,
            results_width         : 30,
            plots_width           : 100,
            font_size             : 9,
        }
    }
}

impl Config {
    pub fn to_plot_config(&self) -> PlotConfig {
        let ratio      = window().device_pixel_ratio();
        PlotConfig {
            background_color      : (&self.background_color     ) . into(),
            label_color_ok        : (&self.label_color_ok       ) . into(),
            label_color_warn      : (&self.label_color_warn     ) . into(),
            label_color_err       : (&self.label_color_err      ) . into(),
            plot_color_ok         : (&self.plot_color_ok        ) . into(),
            plot_color_warn       : (&self.plot_color_warn      ) . into(),
            plot_color_err        : (&self.plot_color_err       ) . into(),
            plot_background_color : (&self.plot_background_color) . into(),
            plot_step_size        : self.plot_step_size as f64 * ratio,
            margin                : self.margin         as f64 * ratio,
            panel_height          : self.panel_height   as f64 * ratio,
            labels_width          : self.labels_width   as f64 * ratio,
            results_width         : self.results_width  as f64 * ratio,
            plots_width           : self.plots_width    as f64 * ratio,
            font_size             : self.font_size      as f64 * ratio,
        }
    }
}



// =============
// === Stats ===
// =============

#[derive(Debug)]
pub struct Stats {
    user_config   : Config,
    config        : PlotConfig,
    width         : f64,
    height        : f64,
    dom           : Element,
    panels        : Vec<Panel>,
    canvas        : HtmlCanvasElement,
    context       : CanvasRenderingContext2d,
    is_first_draw : bool,
}

impl Stats {
    pub fn new() -> Self {
        let user_config: Config = default();
        let panels              = default();
        let width               = default();
        let height              = default();
        let is_first_draw       = true;
        let config              = user_config.to_plot_config();

        let dom = document().create_element("div").unwrap();
        dom.set_attribute("style", "position:absolute;");
        body().prepend_with_node_1(&dom);

        let canvas = document().create_element("canvas").unwrap();
        let mut canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();

        let context = canvas.get_context("2d").unwrap().unwrap();
        let context: CanvasRenderingContext2d = context.dyn_into().unwrap();
        dom.append_child(&canvas);



        let mut out = Self {user_config,config,width,height,dom,panels,canvas,context,is_first_draw};
        out.update_config();
        out
    }

    pub fn mod_config<F:FnOnce(&mut Config)>(&mut self, f:F) {
        f(&mut self.user_config);
        self.update_config();
    }

    fn update_config(&mut self) {
        self.config = self.user_config.to_plot_config()
    }

    fn resize(&mut self) {
        let width = self.config.labels_width
                  + self.config.results_width
                  + self.config.plots_width
                  + 4.0 * self.config.margin;
        let mut height = 0.0;
        for panel in &self.panels {
            height += self.config.margin + self.config.panel_height;
        }
        height += self.config.margin;

        let u_width  = width  as u32;
        let u_height = height as u32;
        self.width   = width;
        self.height  = height;
        self.canvas.set_width  (u_width);
        self.canvas.set_height (u_height);
        self.canvas.set_attribute("style",&format!("width:{}px; height:{}px",u_width/2,u_height/2));

    }

    pub fn add_panel<M:Monitor+'static>(&mut self, monitor:M) -> Panel {
        let panel = Panel::new(self.context.clone(),self.config.clone(),monitor);
        self.panels.push(panel.clone());
        self.resize();
        panel
    }

    pub fn draw(&mut self) {
        if self.is_first_draw {
            self.is_first_draw = false;
            self.first_draw();
        }
        self.shift_plot_area_left();
        self.clear_labels_area();
        self.draw_plots();
    }

    fn shift_plot_area_left(&mut self) {
        let width  = self.width as f64;
        let height = self.height as f64;
        let shift  = -(self.config.plot_step_size as f64);
        self.context.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh
        (&self.canvas,0.0,0.0,width,height,shift,0.0,self.width,self.height);
    }

    fn clear_labels_area(&mut self) {
        let step  = self.config.plot_step_size;
        let width = self.config.labels_width + self.config.results_width + 3.0 * self.config.margin;
        self.context.set_fill_style(&self.config.background_color);
        self.context.fill_rect(0.0,0.0,width,self.height);
        self.context.fill_rect(self.width-step,0.0,step,self.height);
    }

    fn draw_plots(&mut self) {
        self.with_all_panels(|panel| panel.draw());
    }

    pub fn first_draw(&self) {
        self.context.set_fill_style(&self.config.background_color);
        self.context.fill_rect(0.0,0.0,self.width,self.height);
        self.with_all_panels(|panel| panel.first_draw());
    }

    fn with_all_panels<F:Fn(&Panel)>(&self,f:F) {
        let mut total_off = 0.0;
        for panel in &self.panels {
            let off = self.config.margin;
            self.context.translate(0.0,off);
            total_off += off;
            f(panel);
            let off = self.config.panel_height;
            self.context.translate(0.0,off);
            total_off += off;
        }
        self.context.translate(0.0,-total_off);
    }
}



// =============
// === Panel ===
// =============

#[derive(Clone,Debug)]
pub struct Panel {
    rc: Rc<RefCell<PanelData>>
}

impl Panel {
    pub fn new<M:Monitor+'static>
    (context:CanvasRenderingContext2d, config:PlotConfig, monitor:M) -> Self {
        let rc = Rc::new(RefCell::new(PanelData::new(context,config,monitor)));
        Self {rc}
    }

    pub fn draw(&self) {
        self.rc.borrow_mut().draw()
    }

    pub fn first_draw(&self) {
        self.rc.borrow_mut().first_draw()
    }

    pub fn begin(&self) {
        self.rc.borrow_mut().begin()
    }

    pub fn end(&self) {
        self.rc.borrow_mut().end()
    }
}



// ==================
// === ValueCheck ===
// ==================

#[derive(Copy,Clone,Debug)]
pub enum ValueCheck {Correct,Warning,Error}

impl Default for ValueCheck {
    fn default() -> Self {
        Self::Correct
    }
}



// ===============
// === Monitor ===
// ===============

pub trait Monitor: Debug {
    fn label     (&self) -> &str;
    fn begin     (&mut self, time:f64);
    fn end       (&mut self, time:f64);
    fn value     (&self) -> f64;
    fn check     (&self) -> ValueCheck;
    fn max_value (&self) -> Option<f64> { None }
    fn min_value (&self) -> Option<f64> { None }
    fn min_size  (&self) -> Option<f64> { None }
}



// =================
// === PanelData ===
// =================

#[derive(Debug)]
pub struct PanelData {
    label         : String,
    context       : CanvasRenderingContext2d,
    performance   : Performance,
    config        : PlotConfig,
    min_value     : f64,
    max_value     : f64,
    begin_value   : f64,
    value         : f64,
    norm_value    : f64,
    draw_offset   : f64,
    value_check   : ValueCheck,
    monitor       : Box<dyn Monitor>
}

impl PanelData {
    pub fn new<M:Monitor+'static>
    (context:CanvasRenderingContext2d, config:PlotConfig, monitor:M) -> Self {
        let label         = monitor.label().into();
        let performance   = performance();
        let min_value     = f64::INFINITY;
        let max_value     = f64::NEG_INFINITY;
        let begin_value   = default();
        let value         = default();
        let norm_value    = default();
        let draw_offset   = 0.0;
        let value_check   = default();
        let monitor       = Box::new(monitor);
        Self {label,context,performance,config,min_value,max_value,begin_value,value,norm_value,draw_offset,value_check,monitor}
    }

    pub fn begin(&mut self) {
        let time = self.performance.now();
        self.monitor.begin(time);
    }

    pub fn end(&mut self) {
        let time = self.performance.now();
        self.monitor.end(time);
        self.value_check = self.monitor.check();
        self.value       = self.monitor.value();
        if let Some(max_value) = self.monitor.max_value() {
            if self.value > max_value { self.value = max_value; }
        }
        if let Some(min_value) = self.monitor.min_value() {
            if self.value > min_value { self.value = min_value; }
        }
        if self.value > self.max_value { self.max_value = self.value; }
        if self.value < self.min_value { self.min_value = self.value; }

        let mut size = (self.max_value - self.min_value);
        if let Some(min_size) = self.monitor.min_size() {
            if size < min_size { size = min_size; }
        }
        self.norm_value  = (self.value - self.min_value) / size;
    }

    fn move_to_next_element(&mut self, offset:f64) {
        self.context.translate(offset,0.0);
        self.draw_offset += offset;
    }

    fn finish_draw(&mut self) {
        self.context.translate(-self.draw_offset,0.0);
        self.draw_offset = 0.0;
    }

    pub fn draw(&mut self) {
        self.init_draw();
        self.draw_plots();
        self.finish_draw();
    }

    pub fn first_draw(&mut self) {
        self.init_draw();
        self.context.set_fill_style(&self.config.plot_background_color);
        self.context.fill_rect(0.0,0.0,self.config.plots_width,self.config.panel_height);
        self.finish_draw();
    }

    fn init_draw(&mut self) {
        self.move_to_next_element(self.config.margin);
        self.draw_labels();
        self.draw_results();
    }

    fn draw_labels(&mut self) {
        self.context.set_font (&format!("bold {}px Helvetica,Arial,sans-serif",self.config.font_size));
        self.context.set_text_align("right");
        self.context.set_fill_style(&self.config.label_color_ok);
        self.context.fill_text(&self.label,self.config.labels_width,self.config.panel_height - 4.0);
        self.move_to_next_element(self.config.labels_width + self.config.margin);
    }

    fn draw_results(&mut self) {
        let display_value = (self.value * 100.0).round() / 100.0;
        let display_value = format!("{:.*}",2,display_value);
        let color = match self.value_check {
            ValueCheck::Correct => &self.config.label_color_ok,
            ValueCheck::Warning => &self.config.label_color_warn,
            ValueCheck::Error   => &self.config.label_color_err
        };
        self.context.set_fill_style(color);
        self.context.fill_text(&display_value,self.config.results_width,self.config.panel_height - 4.0);
        self.move_to_next_element(self.config.results_width + self.config.margin);
    }

    fn draw_plots(&mut self) {
        self.move_to_next_element(self.config.plots_width - self.config.plot_step_size);

        self.context.set_fill_style(&self.config.plot_background_color);
        self.context.fill_rect(0.0,0.0,self.config.plot_step_size,self.config.panel_height);

        let value_height  = self.norm_value * self.config.panel_height;
        let color = match self.value_check {
            ValueCheck::Correct => &self.config.plot_color_ok,
            ValueCheck::Warning => &self.config.plot_color_warn,
            ValueCheck::Error   => &self.config.plot_color_err
        };
        self.context.set_fill_style(color);
        self.context.fill_rect(0.0,self.config.panel_height-value_height,self.config.plot_step_size,value_height);

    }
}



// ========================
// === FrameTimeMonitor ===
// ========================

#[derive(Debug)]
pub struct FrameTimeMonitor {
    begin_time  : f64,
    value       : f64,
    value_check : ValueCheck,
}

impl FrameTimeMonitor {
    pub fn new() -> Self {
        let begin_time  = default();
        let value       = default();
        let value_check = default();
        Self {begin_time,value,value_check}
    }
}

impl Monitor for FrameTimeMonitor {
    fn label(&self) -> &str {
        "Frame time (ms)"
    }

    fn begin(&mut self, time:f64) {
        self.begin_time = time;
    }

    fn end(&mut self, time:f64) {
        let end_time     = time;
        self.value       = (end_time - self.begin_time);
        self.value_check =
            if      self.value < 1000.0 / 55.0 { ValueCheck::Correct }
            else if self.value < 1000.0 / 25.0 { ValueCheck::Warning }
            else                               { ValueCheck::Error   };
    }

    fn value(&self) -> f64 {
        self.value
    }

    fn check(&self) -> ValueCheck {
        self.value_check
    }
}



// ==================
// === FpsMonitor ===
// ==================

#[derive(Debug)]
pub struct FpsMonitor {
    begin_time  : f64,
    value       : f64,
    value_check : ValueCheck,
}

impl FpsMonitor {
    pub fn new() -> Self {
        let begin_time  = default();
        let value       = default();
        let value_check = default();
        Self {begin_time,value,value_check}
    }
}

impl Monitor for FpsMonitor {
    fn label(&self) -> &str {
        "Frames per second"
    }

    fn begin(&mut self, time:f64) {
        if self.begin_time > 0.0 {
            let end_time     = time;
            self.value       = 1000.0 / (end_time - self.begin_time);
            self.value_check =
                if      self.value >= 55.0 { ValueCheck::Correct }
                else if self.value >= 25.0 { ValueCheck::Warning }
                else                       { ValueCheck::Error   };
        }
        self.begin_time = time;
    }

    fn end(&mut self, time:f64) {}

    fn value(&self) -> f64 {
        self.value
    }

    fn check(&self) -> ValueCheck {
        self.value_check
    }

    fn max_value(&self) -> Option<f64> {
        Some(60.0)
    }
}



// =========================
// === WasmMemoryMonitor ===
// =========================

#[derive(Debug)]
pub struct WasmMemoryMonitor {
    value       : f64,
    value_check : ValueCheck,
}

impl WasmMemoryMonitor {
    pub fn new() -> Self {
        let value       = default();
        let value_check = default();
        Self {value,value_check}
    }
}

impl Monitor for WasmMemoryMonitor {
    fn label(&self) -> &str {
        "WASM memory usage (Mb)"
    }

    fn begin(&mut self, time:f64) {}

    fn end(&mut self, time:f64) {
        let memory: Memory = wasm_bindgen::memory().dyn_into().unwrap();
        let buffer: ArrayBuffer = memory.buffer().dyn_into().unwrap();
        self.value = (buffer.byte_length() as f64) / (1024.0 * 1024.0);
        self.value_check =
            if      self.value <=  50.0 { ValueCheck::Correct }
            else if self.value <= 100.0 { ValueCheck::Warning }
            else                        { ValueCheck::Error   };
    }

    fn value(&self) -> f64 {
        self.value
    }

    fn check(&self) -> ValueCheck {
        self.value_check
    }

    fn min_size(&self) -> Option<f64> {
        Some(100.0)
    }

}