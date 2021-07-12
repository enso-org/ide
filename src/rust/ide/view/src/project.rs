//! The main view of single project opened in IDE.

use crate::prelude::*;

use crate::code_editor;
use crate::graph_editor::component::node;
use crate::graph_editor::component::node::Expression;
use crate::graph_editor::component::visualization;
use crate::graph_editor::GraphEditor;
use crate::graph_editor::NodeId;
use crate::open_dialog::OpenDialog;
use crate::searcher;
use crate::status_bar;

use enso_args::ARGS;
use enso_frp as frp;
use ensogl::Animation;
use ensogl::application;
use ensogl::application::Application;
use ensogl::application::shortcut;
use ensogl::display;
use ensogl::display::shape::*;
use ensogl::DEPRECATED_Animation;
use ensogl::system::web;
use ensogl::system::web::dom;
use ensogl_theme::Theme as Theme;



// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! {
    Input {
        /// Open the searcher.
        open_searcher(),
        /// Open the Open File or Project Dialog.
        show_open_dialog(),
        /// Close the searcher without taking any actions
        close_searcher(),
        /// Close the Open File or Project Dialog without further action
        close_open_dialog(),
        /// Simulates a style toggle press event.
        toggle_style(),
        /// Saves the currently opened module to file.
        save_module(),
        /// Undo the last user's action.
        undo(),
        /// Redo the last undone action.
        redo(),
        /// Show the prompt informing about tab key.
        show_prompt(),
        /// Hide the prompt
        hide_prompt(),
    }

    Output {
        searcher_opened                     (NodeId),
        adding_new_node                     (bool),
        searcher_input                      (Option<NodeId>),
        is_searcher_opened                  (bool),
        old_expression_of_edited_node       (Expression),
        editing_aborted                     (NodeId),
        editing_committed                   (NodeId, Option<searcher::entry::Id>),
        open_dialog_shown                   (bool),
        code_editor_shown                   (bool),
        style                               (Theme),
        fullscreen_visualization_shown      (bool),
        default_gap_between_nodes           (f32),
    }
}



// ==============
// === Shapes ===
// ==============

mod prompt_background {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style, color_rgba:Vector4<f32>) {
            let width         = Var::<Pixels>::from("input_size.x");
            let height        = Var::<Pixels>::from("input_size.y");

            let corner_radius = style.get_number(ensogl_theme::graph_editor::prompt::background::corner_radius);
            let shape         = Rect((&width,&height));
            let shape         = shape.corners_radius(corner_radius.px());
            let bg            = shape.fill(color_rgba);

            bg.into()
        }
    }
}



// =============
// === Model ===
// =============

#[derive(Clone,CloneRef,Debug)]
struct Model {
    app                    : Application,
    logger                 : Logger,
    display_object         : display::object::Instance,
    /// These buttons are present only in a cloud environment.
    window_control_buttons : Immutable<Option<crate::window_control_buttons::View>>,
    graph_editor           : Rc<GraphEditor>,
    searcher               : searcher::View,
    code_editor            : code_editor::View,
    status_bar             : status_bar::View,
    fullscreen_vis         : Rc<RefCell<Option<visualization::fullscreen::Panel>>>,
    prompt_background      : prompt_background::View,
    prompt                 : ensogl_text::Area,
    open_dialog            : Rc<OpenDialog>,
}

impl Model {
    fn new(app:&Application) -> Self {
        let logger                 = Logger::new("project::View");
        let scene                  = app.display.scene();
        let display_object         = display::object::Instance::new(&logger);
        let searcher               = app.new_view::<searcher::View>();
        let graph_editor           = app.new_view::<GraphEditor>();
        let code_editor            = app.new_view::<code_editor::View>();
        let status_bar             = status_bar::View::new(app);
        let fullscreen_vis         = default();
        let prompt_background      = prompt_background::View::new(&logger);
        let prompt                 = ensogl_text::Area::new(app);
        let window_control_buttons = ARGS.is_in_cloud.unwrap_or_default().as_some_from(|| {
            let window_control_buttons = app.new_view::<crate::window_control_buttons::View>();
            display_object.add_child(&window_control_buttons);
            scene.layers.panel.add_exclusive(&window_control_buttons);
            window_control_buttons
        });
        let window_control_buttons = Immutable(window_control_buttons);
        let open_dialog            = Rc::new(OpenDialog::new(app));
        prompt_background.add_child(&prompt);
        prompt.set_content("Press the tab key to search for components.");
        scene.layers.panel.add_exclusive(&prompt_background);
        prompt.remove_from_scene_layer(&scene.layers.main);
        prompt.add_to_scene_layer(&scene.layers.panel_text);

        display_object.add_child(&graph_editor);
        display_object.add_child(&code_editor);
        display_object.add_child(&searcher);
        display_object.add_child(&status_bar);
        display_object.add_child(&prompt_background);
        display_object.remove_child(&searcher);

        let app          = app.clone_ref();
        let graph_editor = Rc::new(graph_editor);
        Self{app,logger,display_object,window_control_buttons,graph_editor,searcher,code_editor,
            status_bar,fullscreen_vis,prompt_background,prompt,open_dialog}
    }

    /// Sets style of IDE to the one defined by parameter `theme`.
    pub fn set_style(&self, theme:Theme) {
        match theme {
            Theme::Light => { self.set_light_style() },
            _            => { self.set_dark_style()  },
        }
    }

    fn set_light_style(&self) {
        ensogl_theme::builtin::light::enable(&self.app);
        self.set_html_style("light-theme");
    }

    fn set_dark_style(&self) {
        ensogl_theme::builtin::dark::enable(&self.app);
        self.set_html_style("dark-theme");
    }

    fn set_html_style(&self, style:&'static str) {
        web::with_element_by_id_or_warn(&self.logger,"root",|root| root.set_class_name(style));
    }

    fn searcher_left_top_position_when_under_node_at(position:Vector2<f32>) -> Vector2<f32> {
        let x = position.x;
        let y = position.y - node::HEIGHT/2.0;
        Vector2(x,y)
    }

    fn searcher_left_top_position_when_under_node(&self, node_id:NodeId) -> Vector2<f32> {
        if let Some(node) = self.graph_editor.model.nodes.get_cloned_ref(&node_id) {
            Self::searcher_left_top_position_when_under_node_at(node.position().xy())
        } else {
            error!(self.logger, "Trying to show searcher under nonexisting node");
            default()
        }
    }

    /// Update Searcher View - its visibility and position - when edited node changed.
    fn update_searcher_view
    ( &self
    , edited_node                : Option<NodeId>
    , is_searcher_empty          : bool
    , searcher_left_top_position : &DEPRECATED_Animation<Vector2<f32>>
    ) {
        match edited_node {
            Some(id) if !is_searcher_empty => {
                self.searcher.show();
                let new_position = self.searcher_left_top_position_when_under_node(id);
                searcher_left_top_position.set_target_value(new_position);
            }
            _ => {
                self.searcher.hide();
                self.searcher.clear_actions();
            }
        }
    }

    fn add_node_and_edit(&self) -> NodeId {
        let graph_editor_inputs = &self.graph_editor.frp.input;
        let node_id = if let Some(selected) = self.graph_editor.model.nodes.selected.first_cloned() {
            let selected_pos = self.graph_editor.model.get_node_position(selected).unwrap_or_default();
            let styles       = StyleWatch::new(&self.app.display.scene().style_sheet);
            let offset_y     = styles.get_number(ensogl_theme::project::default_gap_between_nodes);
            let y            = selected_pos.y - offset_y;
            let pos          = Vector2(selected_pos.x,y);
            graph_editor_inputs.add_node.emit(());
            let node_id = self.graph_editor.frp.output.node_added.value();
            self.graph_editor.set_node_position((node_id,pos));
            node_id
        } else {
            graph_editor_inputs.add_node_at_cursor.emit(());
            self.graph_editor.frp.output.node_added.value()
        };
        graph_editor_inputs.set_node_expression.emit(&(node_id,Expression::default()));
        graph_editor_inputs.edit_node.emit(&node_id);
        node_id
    }

    fn show_fullscreen_visualization(&self, node_id:NodeId) {
        let node = self.graph_editor.model.model.nodes.all.get_cloned_ref(&node_id);
        if let Some(node) = node {
            let visualization = node.view.model.visualization.fullscreen_visualization().clone_ref();
            self.display_object.remove_child(&*self.graph_editor);
            self.display_object.add_child(&visualization);
            *self.fullscreen_vis.borrow_mut() = Some(visualization);
        }
    }

    fn hide_fullscreen_visualization(&self) {
        if let Some(visualization) = std::mem::take(&mut *self.fullscreen_vis.borrow_mut()) {
            self.display_object.remove_child(&visualization);
            self.display_object.add_child(&*self.graph_editor);
        }
    }

    fn on_dom_shape_changed(&self, shape:&dom::shape::Shape) {
        // Top buttons must always stay in top-left corner.
        if let Some(window_control_buttons) = &*self.window_control_buttons {
            let pos = Vector2(-shape.width, shape.height) / 2.0;
            window_control_buttons.set_position_xy(pos);
        }
    }

    fn on_close_clicked(&self) {
        js::close(enso_config::CONFIG.window_app_scope_name);
    }

    fn on_fullscreen_clicked(&self) {
        js::fullscreen();
    }

    fn show_open_dialog(&self) {
        self.display_object.add_child(&*self.open_dialog);
    }

    fn hide_open_dialog(&self) {
        self.display_object.remove_child(&*self.open_dialog);
    }
}



mod js {
    // use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(inline_js="
    export function close(windowAppScopeConfigName) {
        try { window[windowAppScopeConfigName].close(); }
        catch(e) {
            console.error(`Exception thrown from window.${windowAppScopeConfigName}.close:`,e)
        }
    }")]
    extern "C" {
        #[allow(unsafe_code)]
        pub fn close(window_app_scope_name:&str);
    }


    #[wasm_bindgen(inline_js="
    export function fullscreen() {
        try {
            if(document.fullscreenElement === null)
                document.documentElement.requestFullscreen()
            else
                document.exitFullscreen()
        } catch (e) {
            console.error('Exception thrown when toggling fullscreen display mode:',e)
        }
    }
    ")]
    extern "C" {
        #[allow(unsafe_code)]
        pub fn fullscreen();
    }
}



// ============
// === View ===
// ============

/// The main view of single project opened in IDE.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct View {
    model   : Model,
    pub frp : Frp,
}

impl Deref for View {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl View {
    /// Constructor.
    pub fn new(app:&Application) -> Self {
        ensogl_theme::builtin::dark::register(app);
        ensogl_theme::builtin::light::register(app);
        let theme = match ARGS.theme.as_deref() {
            Some("dark") => {
                ensogl_theme::builtin::dark::enable(app);
                Theme::Dark
            }
            _ => {
                ensogl_theme::builtin::light::enable(app);
                Theme::Light
            }
        };

        display::style::javascript::expose_to_window(&app.themes);

        let model                      = Model::new(app);
        let frp                        = Frp::new();
        let searcher                   = &model.searcher.frp;
        let graph                      = &model.graph_editor.frp;
        let project_list               = &model.open_dialog.project_list;
        let file_browser               = &model.open_dialog.file_browser;
        let network                    = &frp.network;
        let searcher_left_top_position = DEPRECATED_Animation::<Vector2<f32>>::new(network);
        let prompt_visibility          = Animation::new(network);

        // FIXME[WD]: Think how to refactor it, as it needs to be done before model, as we do not
        //   want shader recompilation. Model uses styles already.
        model.set_style(theme);
        // TODO[WD]: This should not be needed after the theme switching issue is implemented.
        //   See: https://github.com/enso-org/ide/issues/795
        app.themes.update();

        let style_sheet                    = &model.app.display.scene().style_sheet;
        let styles                         = StyleWatchFrp::new(style_sheet);
        let default_gap_between_nodes_path = ensogl_theme::project::default_gap_between_nodes;

        let default_gap_between_nodes = styles.get_number_or(default_gap_between_nodes_path, 0.0);
        frp::extend! { network
            frp.source.default_gap_between_nodes <+ default_gap_between_nodes;
        }
        frp.source.default_gap_between_nodes.emit(default_gap_between_nodes.value());

        if let Some(window_control_buttons) = &*model.window_control_buttons {
            let initial_size = &window_control_buttons.size.value();
            model.graph_editor.input.space_for_window_buttons(initial_size);
            frp::extend! { network
                graph.space_for_window_buttons <+ window_control_buttons.size;
                eval_ window_control_buttons.close      (model.on_close_clicked());
                eval_ window_control_buttons.fullscreen (model.on_fullscreen_clicked());
            }
        }

        let shape = app.display.scene().shape().clone_ref();
        frp::extend!{ network
            eval shape ((shape) model.on_dom_shape_changed(shape));

            // === Searcher Position and Size ===

            _eval <- all_with(&searcher_left_top_position.value,&searcher.size,f!([model](lt,size) {
                let x = lt.x + size.x / 2.0;
                let y = lt.y - size.y / 2.0;
                model.searcher.set_position_xy(Vector2(x,y));
            }));

            eval searcher.is_visible ([model](is_visible) {
                let is_attached = model.searcher.has_parent();
                if !is_attached && *is_visible {
                    model.display_object.add_child(&model.searcher);
                } else if is_attached && !is_visible {
                    model.display_object.remove_child(&model.searcher);
                }
            });

            // === Searcher Selection ===

            eval searcher.is_selected ((is_selected) graph.set_navigator_disabled(is_selected));


            // === Editing ===

            // The order of instructions below is important to properly distinguish between
            // committing and aborting node editing.

            frp.source.editing_committed <+ searcher.editing_committed
                .map2(&graph.output.node_being_edited, |entry,id| (*id,*entry))
                .filter_map(|(id,entry)| Some(((*id)?, *entry)));

            // This node is true when received "abort_node_editing" signal, and should get false
            // once processing of "node_being_edited" event from graph is performed.
            editing_aborted              <- any(...);
            editing_aborted              <+ frp.close_searcher.constant(true);
            editing_commited_in_searcher <- searcher.editing_committed.constant(());
            should_finish_editing_if_any <- any(frp.close_searcher,editing_commited_in_searcher
                ,frp.open_searcher,frp.show_open_dialog);
            should_finish_editing <- should_finish_editing_if_any.gate(&graph.output.node_editing);
            eval should_finish_editing ((()) graph.input.stop_editing.emit(()));

            visibility_conditions <- all(&graph.output.node_being_edited,&searcher.is_empty);
            _eval                 <- visibility_conditions.map2(&searcher.is_visible,
                f!([model,searcher_left_top_position]((node_id,is_searcher_empty),is_visible) {
                    model.update_searcher_view(*node_id,*is_searcher_empty,&searcher_left_top_position);
                    if !is_visible {
                        // Do not animate
                        searcher_left_top_position.skip();
                    }
                })
            );

            _eval <- graph.output.node_position_set.map2(&graph.output.node_being_edited,
                f!([searcher_left_top_position]((node_id,position),edited_node_id) {
                    if edited_node_id.contains(node_id) {
                        let new = Model::searcher_left_top_position_when_under_node_at(*position);
                        searcher_left_top_position.set_target_value(new);
                    }
                })
            );
            let editing_finished         =  graph.output.node_editing_finished.clone_ref();
            editing_finished_no_entry    <- editing_finished.gate_not(&editing_aborted);
            frp.source.editing_committed <+ editing_finished_no_entry.map(|id| (*id,None));
            frp.source.editing_aborted   <+ editing_finished.gate(&editing_aborted);
            editing_aborted              <+ graph.output.node_editing_finished.constant(false);

            frp.source.searcher_input     <+ graph.output.node_being_edited;
            frp.source.is_searcher_opened <+ frp.searcher_input.map(|n| n.is_some());


            // === Adding Node ===

            frp.source.adding_new_node <+ frp.open_searcher.constant(true);
            frp.source.searcher_opened <+ frp.open_searcher.map(f!((_) model.add_node_and_edit()));

            adding_committed           <- frp.editing_committed.gate(&frp.adding_new_node).map(|(id,_)| *id);
            adding_aborted             <- frp.editing_aborted.gate(&frp.adding_new_node);
            frp.source.adding_new_node <+ any(&adding_committed,&adding_aborted).constant(false);

            eval adding_committed ([graph](node) {
                graph.deselect_all_nodes();
                graph.select_node(node);
            });
            eval adding_aborted  ((node) graph.remove_node(node));


            // === Opening Open File or Project Dialog ===

            eval_ frp.show_open_dialog  (model.show_open_dialog());
            project_chosen   <- project_list.chosen_entry.constant(());
            file_chosen      <- file_browser.entry_chosen.constant(());
            should_be_closed <- any(frp.close_open_dialog,project_chosen,file_chosen);
            eval_ should_be_closed (model.hide_open_dialog());

            frp.source.open_dialog_shown <+ bool(&should_be_closed,&frp.show_open_dialog);


            // === Style toggle ===

            let style_toggle_ev   = frp.toggle_style.clone_ref();
            style_pressed        <- style_toggle_ev.toggle() ;
            style_was_pressed    <- style_pressed.previous();
            style_press          <- style_toggle_ev.gate_not(&style_was_pressed);
            style_press_on_off   <- style_press.map2(&frp.style, |_,s| match s {
                Theme::Light => Theme::Dark ,
                _            => Theme::Light,
            });
            frp.source.style     <+ style_press_on_off;
            eval frp.style ((style) model.set_style(style.clone()));


            // === Fullscreen Visualization ===

            // TODO[ao]: All DOM elements in visualizations ale displayed below canvas, because
            //     The mouse cursor must be displayed over them. But fullscreen visualization should
            //     be displayed "above" nodes. The workaround is to hide whole graph editor except
            //     fullscreen visualization, and bring it back when fullscreen is closed.
            //
            //     The workaround should be replaced with proper solution being a part of
            //     https://github.com/enso-org/ide/issues/526
            eval  graph.visualization_fullscreen ([model](node_id) {
                if let Some(node_id) = node_id {
                    model.show_fullscreen_visualization(*node_id)
                } else {
                    model.hide_fullscreen_visualization()
                }
            });


            // === Prompt ===
            init <- source::<()>();
            let prompt_bg_color_path   = ensogl_theme::graph_editor::prompt::background;
            let prompt_bg_padding_path = ensogl_theme::graph_editor::prompt::background::padding;
            let prompt_color_path      = ensogl_theme::graph_editor::prompt::text;
            let prompt_size_path       = ensogl_theme::graph_editor::prompt::text::size;
            let prompt_bg_color        = styles.get_color(prompt_bg_color_path);
            prompt_bg_color            <- all(&prompt_bg_color,&init)._0();
            let prompt_bg_padding      = styles.get_number(prompt_bg_padding_path);
            prompt_bg_padding          <- all(&prompt_bg_padding,&init)._0();
            let prompt_color           = styles.get_color(prompt_color_path);
            prompt_color               <- all(&prompt_color,&init)._0();
            let prompt_size            = styles.get_number(prompt_size_path);
            prompt_size                <- all(&prompt_size,&init)._0();

            prompt_visibility.target <+ frp.show_prompt.constant(1.0);
            prompt_visibility.target <+ frp.hide_prompt.constant(0.0);
            prompt_visibility.target <+ frp.is_searcher_opened.filter(|v| *v).constant(0.0);
            _eval <- all_with4(&prompt_visibility.value,&prompt_bg_color,&prompt_color,&prompt_size,
                f!([model](weight,bg_color,color,size) {
                    let mut bg_color = *bg_color;
                    bg_color.alpha  *= weight;
                    let mut color    = *color;
                    color.alpha     *= weight;
                    model.prompt_background.color_rgba.set(bg_color.into());
                    model.prompt.set_color_all(color);
                    model.prompt.set_default_text_size(ensogl_text::Size(*size));
                })
            );
            _eval <- all_with3(&model.prompt.width,&prompt_size,&prompt_bg_padding,
                f!([model](width,size,padding) {
                    model.prompt.set_position_x(- *width / 2.0);
                    model.prompt_background.size.set(Vector2(*width + padding, *size + padding));
                })
            );
        }
        init.emit(());
        std::mem::forget(prompt_visibility);

        Self{model,frp}
    }

    /// Graph Editor View.
    pub fn graph(&self) -> &GraphEditor { &self.model.graph_editor }

    /// Searcher View.
    pub fn searcher(&self) -> &searcher::View { &self.model.searcher }

    /// Code Editor View.
    pub fn code_editor(&self) -> &code_editor::View { &self.model.code_editor }

    /// Status Bar View.
    pub fn status_bar(&self) -> &status_bar::View { &self.model.status_bar }

    /// Open File or Project Dialog
    pub fn open_dialog(&self) -> &OpenDialog { &self.model.open_dialog }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object }
}

impl application::command::FrpNetworkProvider for View {
    fn network(&self) -> &frp::Network { &self.frp.network }
}

impl application::View for View {
    fn label() -> &'static str { "ProjectView" }

    fn new(app:&Application) -> Self { View::new(app) }

    fn app(&self) -> &Application {
        &self.model.app
    }

    fn default_shortcuts() -> Vec<application::shortcut::Shortcut> {
        use shortcut::ActionType::*;
        (&[ (Press   , "!is_searcher_opened", "tab"             , "open_searcher")
          , (Press   , "!is_searcher_opened", "cmd o"           , "show_open_dialog")
          , (Press   , "is_searcher_opened" , "escape"          , "close_searcher")
          , (Press   , "open_dialog_shown"  , "escape"          , "close_open_dialog")
          , (Press   , ""                   , "tab"             , "hide_prompt")
          , (Press   , ""                   , "cmd o"           , "hide_prompt")
          , (Press   , ""                   , "cmd alt shift t" , "toggle_style")
          , (Press   , ""                   , "cmd s"           , "save_module")
          , (Press   , ""                   , "cmd z"           , "undo")
          , (Press   , ""                   , "cmd y"           , "redo")
          ]).iter().map(|(a,b,c,d)|Self::self_shortcut_when(*a,*c,*d,*b)).collect()
    }
}
