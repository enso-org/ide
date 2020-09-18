//! The text area implementation. It serves the purpose of single and multi-line text labels and
//! text editors.

use crate::prelude::*;

use crate::buffer::data::unit::*;
use crate::buffer::Transform;
use crate::buffer;
use crate::buffer::style;
use crate::buffer::Text;
use crate::typeface::glyph::Glyph;
use crate::typeface::glyph;
use crate::typeface::pen;
use crate::typeface;

use enso_frp as frp;
use enso_frp::io::keyboard::Key;
use ensogl_core::application::Application;
use ensogl_core::application::shortcut;
use ensogl_core::application;
use ensogl_core::data::color;
use ensogl_core::display::scene::Scene;
use ensogl_core::display::shape::*;
use ensogl_core::display;
use ensogl_core::gui::component::Animation;
use ensogl_core::gui::component;
use ensogl_core::gui;
use ensogl_core::system::web::clipboard;
use ensogl_core::system::gpu::shader::glsl::traits::IntoGlsl;
use ensogl_theme;



// =================
// === Constants ===
// =================

/// Record separator ASCII code. Used for separating of copied strings. It is defined as the `\RS`
/// escape code (`x1E`) (https://en.wikipedia.org/wiki/ASCII).
pub const RECORD_SEPARATOR : &str = "\x1E";



// ==================
// === Background ===
// ==================

// FIXME[WD]: Commented for now. We need to consider how to handle background with active state.
// This should be resolved together with the https://github.com/enso-org/ide/issues/670 PR.

// /// Canvas node shape definition.
// pub mod background {
//     use super::*;
//
//     ensogl::define_shape_system! {
//         (style:Style, selection:f32) {
//             let out = Rect((1000.px(),1000.px())).fill(color::Rgba::new(1.0,1.0,1.0,0.05));
//             out.into()
//         }
//     }
// }



// ==============
// === Cursor ===
// ==============

const LINE_VERTICAL_OFFSET     : f32 = 4.0; // Set manually. May depend on font. To be improved.
const CURSOR_PADDING           : f32 = 4.0;
const CURSOR_WIDTH             : f32 = 2.0;
const CURSOR_ALPHA             : f32 = 0.8;
const CURSORS_SPACING          : f32 = 1.0;
const SELECTION_ALPHA          : f32 = 0.3;
const SELECTION_CORNER_RADIUS  : f32 = 2.0;
const BLINK_SLOPE_IN_DURATION  : f32 = 200.0;
const BLINK_SLOPE_OUT_DURATION : f32 = 200.0;
const BLINK_ON_DURATION        : f32 = 300.0;
const BLINK_OFF_DURATION       : f32 = 300.0;
const BLINK_PERIOD             : f32 =
    BLINK_SLOPE_IN_DURATION + BLINK_SLOPE_OUT_DURATION + BLINK_ON_DURATION + BLINK_OFF_DURATION;


/// Text cursor and selection shape definition. If the shape is narrow, it is considered a cursor,
/// and thus, it blinks.
///
/// ## Blinking Implementation
///
/// The blinking alpha is a time-dependent function which starts as a fully opaque value and
/// changes periodically. The `start_time` parameter is set to the current time after each cursor
/// operation, which makes cursor visible during typing and after position change.
///
/// ```compile_fail
/// |
/// |    on         off
/// | <------>   <------->
/// | --------.             .--------.             .-...
/// |          \           /          \           /
/// |           '---------'            '---------'
/// |         <->         <->
/// |      slope_out   slope_in
/// |                                              time
/// |-------------------------------------------------->
/// start time
/// ```
pub mod selection {
    use super::*;
    ensogl_core::define_shape_system! {
        (style:Style, selection:f32, start_time:f32, letter_width:f32) {
            let width_abs      = Var::<f32>::from("abs(input_size.x)");
            let height         = Var::<f32>::from("input_size.y");
            let rect_width     = width_abs - 2.0 * CURSOR_PADDING;
            let rect_height    = height    - 2.0 * CURSOR_PADDING;
            let time           = Var::<f32>::from("input_time");
            let one            = Var::<f32>::from(1.0);
            let time           = time - start_time;
            let on_time        = BLINK_ON_DURATION + BLINK_SLOPE_OUT_DURATION;
            let off_time       = on_time + BLINK_OFF_DURATION;
            let sampler        = time % BLINK_PERIOD;
            let slope_out      = sampler.smoothstep(BLINK_ON_DURATION,on_time);
            let slope_in       = sampler.smoothstep(off_time,BLINK_PERIOD);
            let blinking_alpha = (one - slope_out + slope_in) * CURSOR_ALPHA;
            let sel_width      = &rect_width - CURSOR_WIDTH;
            let alpha_weight   = sel_width.smoothstep(0.0,letter_width);
            let alpha          = alpha_weight.mix(blinking_alpha,SELECTION_ALPHA);
            let shape          = Rect((1.px() * rect_width,1.px() * rect_height));
            let shape          = shape.corners_radius(SELECTION_CORNER_RADIUS.px());
            let color_path     = ensogl_theme::vars::text_editor::text::selection::color;
            let color          = style.get_color(color_path);
            let color          = color::Rgba::from(color);
            let color          = format!("srgba({},{},{},{})",color.red,color.green,color.blue,alpha.glsl());
            let shape          = shape.fill(color);
            shape.into()
        }
    }
}



// =================
// === Selection ===
// =================

/// Visual representation of text cursor and text selection.
///
/// ## Implementation Notes
/// Selection contains a `right_side` display object which is always placed on its right side. It is
/// used for smooth glyph animation. For example, after several glyphs were selected and removed,
/// the selection will gradually shrink. Making all following glyphs children of the `right_side`
/// object will make the following glyphs  animate while the selection is shrinking.
#[derive(Clone,CloneRef,Debug)]
pub struct Selection {
    logger         : Logger,
    display_object : display::object::Instance,
    right_side     : display::object::Instance,
    shape_view     : component::ShapeView<selection::Shape>,
    network        : frp::Network,
    position       : Animation<Vector2>,
    width          : Animation<f32>,
    edit_mode      : Rc<Cell<bool>>,
}

impl Deref for Selection {
    type Target = component::ShapeView<selection::Shape>;
    fn deref(&self) -> &Self::Target {
        &self.shape_view
    }
}

impl Selection {
    /// Constructor.
    pub fn new(logger:impl AnyLogger, scene:&Scene, edit_mode:bool) -> Self {
        let logger         = Logger::sub(logger,"selection");
        let display_object = display::object::Instance::new(&logger);
        let right_side     = display::object::Instance::new(&logger);
        let network        = frp::Network::new();
        let shape_view     = component::ShapeView::<selection::Shape>::new(&logger,scene);
        let position       = Animation::new(&network);
        let width          = Animation::new(&network);
        let edit_mode      = Rc::new(Cell::new(edit_mode));
        let debug          = false; // Change to true to slow-down movement for debug purposes.
        let spring_factor  = if debug { 0.1 } else { 1.5 };
        position . update_spring (|spring| spring * spring_factor);
        width    . update_spring (|spring| spring * spring_factor);

        Self {logger,display_object,right_side,shape_view,network,position,width,edit_mode} . init()
    }

    fn init(self) -> Self {
        let network    = &self.network;
        let view       = &self.shape_view;
        let object     = &self.display_object;
        let right_side = &self.right_side;
        self.add_child(view);
        view.add_child(right_side);
        frp::extend! { network
            _eval <- all_with(&self.position.value,&self.width.value,
                f!([view,object,right_side](p,width){
                    let side        = width.signum();
                    let abs_width   = width.abs();
                    let width       = max(CURSOR_WIDTH, abs_width - CURSORS_SPACING);
                    let view_width  = CURSOR_PADDING * 2.0 + width;
                    let view_height = CURSOR_PADDING * 2.0 + LINE_HEIGHT;
                    let view_x      = (abs_width/2.0) * side;
                    let view_y      = 0.0;
                    object.set_position_xy(*p);
                    right_side.set_position_x(abs_width/2.0);
                    view.shape.sprite.size.set(Vector2(view_width,view_height));
                    view.set_position_xy(Vector2(view_x,view_y));
                })
            );
        }
        self
    }

    fn flip_sides(&self) {
        let width    = self.width.target_value();
        self.position.set_value(self.position.value() + Vector2(width,0.0));
        self.position.set_target_value(self.position.target_value() + Vector2(width,0.0));

        self.width.set_value(-self.width.value());
        self.width.set_target_value(-self.width.target_value());
    }
}

impl display::Object for Selection {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// ====================
// === SelectionMap ===
// ====================

/// Mapping between selection id, `Selection`, and text location.
#[derive(Clone,Debug,Default)]
pub struct SelectionMap {
    id_map       : HashMap<usize,Selection>,
    location_map : HashMap<usize,HashMap<Column,usize>>
}



// ============
// === Line ===
// ============

/// Visual line representation.
///
/// **Design Notes**
/// The `divs` and `centers` are kept as vectors for performance reasons. Especially, when clicking
/// inside of the text area, it allows us to binary search the place of the mouse pointer.
#[derive(Debug)]
pub struct Line {
    display_object : display::object::Instance,
    glyphs         : Vec<Glyph>,
    divs           : Vec<f32>,
    centers        : Vec<f32>,
}

impl Line {
    fn new(logger:impl AnyLogger) -> Self {
        let logger         = Logger::sub(logger,"line");
        let display_object = display::object::Instance::new(logger);
        let glyphs         = default();
        let divs           = default();
        let centers        = default();
        Self {display_object,glyphs,divs,centers}
    }

    /// Set the division points (offsets between letters). Also updates center points.
    fn set_divs(&mut self, divs:Vec<f32>) {
        let div_iter         = divs.iter();
        let div_iter_skipped = divs.iter().skip(1);
        self.centers         = div_iter.zip(div_iter_skipped).map(|(t,s)|(t+s)/2.0).collect();
        self.divs = divs;
    }

    fn div_index_close_to(&self, offset:f32) -> usize {
        self.centers.binary_search_by(|t|t.partial_cmp(&offset).unwrap()).unwrap_both()
    }

    fn div_by_column(&self, column:Column) -> f32 {
        let ix = column.as_usize().min(self.divs.len() - 1);
        self.divs[ix]
    }

    fn resize_with(&mut self, size:usize, cons:impl Fn()->Glyph) {
        let display_object = self.display_object().clone_ref();
        self.glyphs.resize_with(size,move || {
            let glyph = cons();
            display_object.add_child(&glyph);
            glyph
        });
    }
}

impl display::Object for Line {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// =============
// === Lines ===
// =============

/// Set of all visible lines.
#[derive(Clone,CloneRef,Debug,Default)]
struct Lines {
    rc : Rc<RefCell<Vec<Line>>>
}

impl Lines {
    /// The number of visible lines.
    pub fn count(&self) -> usize {
        self.rc.borrow().len()
    }

    /// Resize the line container and use the provided function to construct missing elements.
    pub fn resize_with(&self, size:usize, cons:impl Fn(usize)->Line) {
        let vec    = &mut self.rc.borrow_mut();
        let mut ix = vec.len();
        vec.resize_with(size,|| {
            let line = cons(ix);
            ix += 1;
            line
        })
    }
}



// ===========
// === FRP ===
// ===========

ensogl_core::def_command_api! { Commands
    /// Insert character of the last pressed key at every cursor.
    insert_char_of_last_pressed_key,
    /// Increase the indentation of all lines containing cursors.
    increase_indentation,
    /// Decrease the indentation of all lines containing cursors.
    decrease_indentation,
    /// Removes the character on the left of every cursor.
    delete_left,
    /// Removes the word on the left of every cursor.
    delete_word_left,
    /// Set the text cursor at the mouse cursor position.
    set_cursor_at_mouse_position,
    /// Set the text cursor at the end of text.
    set_cursor_at_end,
    /// Add a new cursor at the mouse cursor position.
    add_cursor_at_mouse_position,
    /// Remove all cursors.
    remove_all_cursors,
    /// Start changing the shape of the newest selection with the mouse position.
    start_newest_selection_end_follow_mouse,
    /// Stop changing the shape of the newest selection with the mouse position.
    stop_newest_selection_end_follow_mouse,
    /// Move the cursor to the left by one character.
    cursor_move_left,
    /// Move the cursor to the right by one character.
    cursor_move_right,
    /// Move the cursor to the left by one word.
    cursor_move_left_word,
    /// Move the cursor to the right by one word.
    cursor_move_right_word,
    /// Move the cursor down one line.
    cursor_move_down,
    /// Move the cursor up one line.
    cursor_move_up,
    /// Extend the cursor selection to the left by one character.
    cursor_select_left,
    /// Extend the cursor selection to the right by one character.
    cursor_select_right,
    /// Extend the cursor selection down one line.
    cursor_select_down,
    /// Extend the cursor selection up one line.
    cursor_select_up,
    /// Extend the cursor selection to the left by one word.
    cursor_select_left_word,
    /// Extend the cursor selection to the right by one word.
    cursor_select_right_word,
    /// Select all characters.
    select_all,
    /// Select the word at cursor position.
    select_word_at_cursor,
    /// Discard all but the first selection.
    keep_first_selection_only,
    /// Discard all but the last selection.
    keep_last_selection_only,
    /// Discard all but the first selection and convert it to cursor.
    keep_first_cursor_only,
    /// Discard all but the last selection and convert it to cursor.
    keep_last_cursor_only,
    /// Discard all but the newest selection.
    keep_newest_selection_only,
    /// Discard all but the oldest selection.
    keep_oldest_selection_only,
    /// Discard all but the newest selection and convert it to cursor.
    keep_newest_cursor_only,
    /// Discard all but the oldest selection and convert it to cursor.
    keep_oldest_cursor_only,
    /// Set the oldest selection end to mouse position.
    set_newest_selection_end_to_mouse_position,
    /// Set the newest selection end to mouse position.
    set_oldest_selection_end_to_mouse_position,
    /// Undo the last operation.
    undo,
    /// Redo the last operation.
    redo,
    /// Copy selected text to clipboard.
    copy,
    /// Paste selected text from clipboard.
    paste,
    /// Set the text area in active state. You should rather not need to control it manually, as it is automatically managed by the active state manager.
    set_active_on,
    /// Set the text area in non-active state. You should rather not need to control it manually, as it is automatically managed by the active state manager.
    set_active_off,
}

impl application::command::CommandApi for Area {
    fn command_api_docs() -> Vec<application::command::EndpointDocs> {
        Commands::command_api_docs()
    }

    fn command_api(&self) -> Vec<application::command::CommandEndpoint> {
        self.frp.input.command.command_api()
    }
}

crate::define_endpoints! {
    Commands { Commands }
    Input {
        set_cursor            (Location),
        add_cursor            (Location),
        paste_string          (String),
        insert                (String),
        set_color_bytes       (buffer::Range<Bytes>,color::Rgba),
        set_default_color     (color::Rgba),
        set_default_text_size (style::Size),
        set_content           (String),
    }
    Output {
        mouse_cursor_style (gui::cursor::Style),
        active             (bool),
        width              (f32),
        changed            (Text),
    }
}



// ============
// === Area ===
// ============

/// Hardcoded line height. To be generalized in the future.
pub const LINE_HEIGHT : f32 = 14.0;

/// The visual text area implementation.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Area {
    data    : AreaData,
    pub frp : Frp,
}

impl Deref for Area {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl Area {
    /// Consturctor.
    pub fn new(app:&Application) -> Self {
        let network = frp::Network::new();
        let data    = AreaData::new(app,&network);
        let output  = data.frp_endpoints.clone_ref();
        let frp     = Frp::new(network,output);
        Self {data,frp} . init()
    }

    fn init(self) -> Self {
        let network  = &self.frp.network;
        let model    = &self.data;
        let mouse    = &model.scene.mouse.frp;
        let input    = &model.frp_endpoints.input;
        let cmd      = &input.command;
        let pos      = Animation :: <Vector2> :: new(&network);
        let keyboard = &model.scene.keyboard;
        let m        = &model;
        pos.update_spring(|spring| spring*2.0);

        frp::extend! { network

            // FIXME[WD]: Commented for now. We need to consider how to handle background with active state.
            // This should be resolved together with the https://github.com/enso-org/ide/issues/670 PR.

            // cursor_over  <- bg.events.mouse_over.constant(gui::cursor::Style::new_text_cursor());
            // cursor_out   <- bg.events.mouse_out.constant(gui::cursor::Style::default());
            // mouse_cursor <- any(cursor_over,cursor_out);
            // self.frp.source.mouse_cursor_style <+ mouse_cursor;


            // === Active State Management ===

            // FIXME[WD]: Connect the active state to active state management when its ready.
            // Should be removed as part of https://github.com/enso-org/ide/issues/670
            active <- bool(&cmd.set_active_off,&cmd.set_active_on);
            self.frp.source.active <+ active;

            // === Set / Add cursor ===

            // FIXME[WD]: These frp nodes are simulating active state management. To be removed
            // as part of https://github.com/enso-org/ide/issues/670
            set_cursor_at_mouse_position <- cmd.set_cursor_at_mouse_position.gate(&active);
            set_cursor_at_end            <- cmd.set_cursor_at_end           .gate(&active);
            add_cursor_at_mouse_position <- cmd.add_cursor_at_mouse_position.gate(&active);

            mouse_on_set_cursor      <- mouse.position.sample(&set_cursor_at_mouse_position);
            mouse_on_add_cursor      <- mouse.position.sample(&add_cursor_at_mouse_position);
            loc_on_set_cursor_mouse  <- mouse_on_set_cursor.map(f!((p) m.get_in_text_location(*p)));
            loc_on_add_cursor_mouse  <- mouse_on_add_cursor.map(f!((p) m.get_in_text_location(*p)));
            loc_on_set_cursor_at_end <- set_cursor_at_end.map(f_!(model.buffer.text().location_of_text_end()));
            loc_on_set_cursor        <- any(input.set_cursor,loc_on_set_cursor_mouse,loc_on_set_cursor_at_end);
            loc_on_add_cursor        <- any(&input.add_cursor,&loc_on_add_cursor_mouse);


            eval loc_on_set_cursor ((loc) m.buffer.frp.set_cursor(loc));
            eval loc_on_add_cursor ((loc) m.buffer.frp.add_cursor(loc));

            _eval <- m.buffer.frp.selection_edit_mode.map2
                (&m.scene.frp.frame_time,f!([m](selections,time) {
                        m.redraw(); // FIXME: added for undo redo. Should not be needed.
                        m.on_modified_selection(selections,*time,true)
                    }
            ));

            _eval <- m.buffer.frp.selection_non_edit_mode.map2
                (&m.scene.frp.frame_time,f!([m](selections,time) {
                    m.redraw(); // FIXME: added for undo redo. Should not be needed.
                    m.on_modified_selection(selections,*time,false)
                }
            ));

            selecting <- bool
                ( &cmd.stop_newest_selection_end_follow_mouse
                , &cmd.start_newest_selection_end_follow_mouse
                );

            set_sel_end_1 <- mouse.position.gate(&selecting);
            set_sel_end_2 <- mouse.position.sample(&cmd.set_newest_selection_end_to_mouse_position);
            set_newest_selection_end <- any(&set_sel_end_1,&set_sel_end_2);

            eval set_newest_selection_end([m](screen_pos) {
                let location = m.get_in_text_location(*screen_pos);
                m.buffer.frp.set_newest_selection_end(location);
            });


            // === Copy / Paste ===

            copy_sels      <- cmd.copy.map(f_!(m.buffer.selections_contents()));
            all_empty_sels <- copy_sels.map(|s|s.iter().all(|t|t.is_empty()));
            line_sel_mode  <- copy_sels.gate(&all_empty_sels);

            eval_ line_sel_mode (m.buffer.frp.cursors_select(Some(Transform::Line)));
            non_line_sel_mode_sels <- copy_sels.gate_not(&all_empty_sels);
            line_sel_mode_sels     <- line_sel_mode.map(f_!(m.buffer.selections_contents()));
            sels                   <- any(&line_sel_mode_sels,&non_line_sel_mode_sels);
            eval sels ((s) m.copy(s));
            eval_ cmd.paste (m.paste());
            eval input.paste_string ((s) m.buffer.frp.paste(m.decode_paste(s)));


            eval_ m.buffer.frp.text_changed (m.redraw());

            eval_ cmd.remove_all_cursors (m.buffer.frp.remove_all_cursors());

            eval_ cmd.keep_first_selection_only (m.buffer.frp.keep_first_selection_only());
            eval_ cmd.keep_last_selection_only  (m.buffer.frp.keep_last_selection_only());
            eval_ cmd.keep_first_cursor_only    (m.buffer.frp.keep_first_cursor_only());
            eval_ cmd.keep_last_cursor_only     (m.buffer.frp.keep_last_cursor_only());

            eval_ cmd.keep_newest_selection_only (m.buffer.frp.keep_newest_selection_only());
            eval_ cmd.keep_oldest_selection_only (m.buffer.frp.keep_oldest_selection_only());
            eval_ cmd.keep_newest_cursor_only    (m.buffer.frp.keep_newest_cursor_only());
            eval_ cmd.keep_oldest_cursor_only    (m.buffer.frp.keep_oldest_cursor_only());

            eval_ cmd.cursor_move_left  (m.buffer.frp.cursors_move(Transform::Left));
            eval_ cmd.cursor_move_right (m.buffer.frp.cursors_move(Transform::Right));
            eval_ cmd.cursor_move_up    (m.buffer.frp.cursors_move(Transform::Up));
            eval_ cmd.cursor_move_down  (m.buffer.frp.cursors_move(Transform::Down));

            eval_ cmd.cursor_move_left_word  (m.buffer.frp.cursors_move(Transform::LeftWord));
            eval_ cmd.cursor_move_right_word (m.buffer.frp.cursors_move(Transform::RightWord));

            eval_ cmd.cursor_select_left  (m.buffer.frp.cursors_select(Transform::Left));
            eval_ cmd.cursor_select_right (m.buffer.frp.cursors_select(Transform::Right));
            eval_ cmd.cursor_select_up    (m.buffer.frp.cursors_select(Transform::Up));
            eval_ cmd.cursor_select_down  (m.buffer.frp.cursors_select(Transform::Down));

            eval_ cmd.cursor_select_left_word  (m.buffer.frp.cursors_select(Transform::LeftWord));
            eval_ cmd.cursor_select_right_word (m.buffer.frp.cursors_select(Transform::RightWord));

            eval_ cmd.select_all            (m.buffer.frp.cursors_select(Transform::All));
            eval_ cmd.select_word_at_cursor (m.buffer.frp.cursors_select(Transform::Word));

            eval_ cmd.delete_left      (m.buffer.frp.delete_left());
            eval_ cmd.delete_word_left (m.buffer.frp.delete_word_left());

            eval_ cmd.undo (m.buffer.frp.undo());
            eval_ cmd.redo (m.buffer.frp.redo());


            // === Insert ===

            key_to_insert <- keyboard.frp.on_pressed.sample(&cmd.insert_char_of_last_pressed_key);
            key_to_insert <= key_to_insert.map(f!((key) m.key_to_string(key)));
            str_to_insert <- any(&input.insert,&key_to_insert);
            eval str_to_insert ((s) m.buffer.frp.insert(s));
            eval input.set_content ([input,cmd](s) {
                input.set_cursor(&default());
                cmd.select_all();
                input.insert(s);
                cmd.remove_all_cursors();
            });

            // === Colors ===

            eval input.set_default_color     ((t) m.buffer.frp.set_default_color(*t));
            eval input.set_default_text_size ((t) m.buffer.frp.set_default_text_size(*t));
            eval input.set_color_bytes       ((t) {
                m.buffer.frp.set_color_bytes.emit(*t);
                m.redraw(); // FIXME: Should not be needed.
            });

            // === Changes ===

            self.frp.source.changed <+ m.buffer.frp.text_changed.map(f_!(m.buffer.text()));
        }
        self
    }
}



// ================
// === AreaData ===
// ================

/// Internal representation of `Area`.
#[derive(Clone,CloneRef,Debug)]
pub struct AreaData {
    scene          : Scene,
    logger         : Logger,
    frp_endpoints  : FrpEndpoints,
    buffer         : buffer::View,
    display_object : display::object::Instance,
    glyph_system   : glyph::System,
    lines          : Lines,
    selection_map  : Rc<RefCell<SelectionMap>>,
    // FIXME[WD]: Commented for now. We need to consider how to handle background with active state.
    // This should be resolved together with the https://github.com/enso-org/ide/issues/670 PR.
    //background     : component::ShapeView<background::Shape>,
}

impl AreaData {
    /// Constructor.
    pub fn new(app:&Application, network:&frp::Network) -> Self {
        let scene          = app.display.scene().clone_ref();
        let logger         = Logger::new("text_area");
        let _bg_logger     = Logger::sub(&logger,"background");
        let selection_map  = default();

        // FIXME[WD]: Commented for now. We need to consider how to handle background with active state.
        // This should be resolved together with the https://github.com/enso-org/ide/issues/670 PR.
        // let background     = component::ShapeView::<background::Shape>::new(&bg_logger,&scene);
        // display_object.add_child(&background);
        // background.shape.sprite.size.set(Vector2(150.0,100.0));
        // background.mod_position(|p| p.x += 50.0);

        let fonts          = scene.extension::<typeface::font::Registry>();
        let font           = fonts.load("DejaVuSansMono");
        let glyph_system   = typeface::glyph::System::new(&scene,font);
        let display_object = display::object::Instance::new(&logger);
        let glyph_system   = glyph_system.clone_ref();
        let buffer         = default();
        let lines          = default();
        let frp_inputs     = FrpInputs::new(network);
        let frp_endpoints  = FrpEndpoints::new(&network,frp_inputs.clone_ref());

        // FIXME[WD]: These settings should be managed wiser. They should be set up during
        // initialization of the shape system, not for every area creation. To be improved during
        // refactoring of the architecture some day.
        let shape_system = scene.shapes.shape_system(PhantomData::<selection::Shape>);
        let symbol       = &shape_system.shape_system.sprite_system.symbol;
        shape_system.shape_system.set_pointer_events(false);

        // FIXME[WD]: This is temporary sorting utility, which places the cursor in front of mouse
        // pointer. Should be refactored when proper sorting mechanisms are in place.
        scene.views.main.remove(symbol);
        scene.views.label.add(symbol);
        Self {scene,logger,frp_endpoints,display_object,glyph_system,buffer,lines,selection_map}.init()
    }

    fn on_modified_selection(&self, selections:&buffer::selection::Group, time:f32, do_edit:bool) {
        {
        let mut selection_map     = self.selection_map.borrow_mut();
        let mut new_selection_map = SelectionMap::default();
        for sel in selections {
            let sel        = self.buffer.snap_selection(*sel);
            let id         = sel.id;
            let start_line = sel.start.line.as_usize();
            let end_line   = sel.end.line.as_usize();
            let min_pos_x  = self.lines.rc.borrow()[start_line].div_by_column(sel.start.column);
            let max_pos_x  = self.lines.rc.borrow()[end_line].div_by_column(sel.end.column);
            let logger     = Logger::sub(&self.logger,"cursor");
            let min_pos_y  = -LINE_HEIGHT/2.0 - LINE_HEIGHT * start_line as f32;
            let pos        = Vector2(min_pos_x,min_pos_y);
            let width      = max_pos_x - min_pos_x;
            let selection  = match selection_map.id_map.remove(&id) {
                Some(selection) => {
                    let select_left  = selection.width.simulator.target_value() < 0.0;
                    let select_right = selection.width.simulator.target_value() > 0.0;
                    let tgt_pos_x    = selection.position.simulator.target_value().x;
                    let tgt_width    = selection.width.simulator.target_value();
                    let mid_point    = tgt_pos_x + tgt_width / 2.0;
                    let go_left      = pos.x < mid_point;
                    let go_right     = pos.x > mid_point;
                    let need_flip    = (select_left && go_left) || (select_right && go_right);
                    if width == 0.0 && need_flip { selection.flip_sides() }
                    selection.position.set_target_value(pos);
                    selection
                }
                None => {
                    let selection = Selection::new(&logger,&self.scene,do_edit);
                    selection.shape.letter_width.set(7.0); // FIXME hardcoded values
                    self.add_child(&selection);
                    selection.position.set_target_value(pos);
                    selection.position.skip();
                    let selection_network = &selection.network;
                    let model = self.clone_ref(); // FIXME: memory leak. To be fixed with the below note.
                    frp::extend! { selection_network
                        // FIXME[WD]: This is ultra-slow. Redrawing all glyphs on each
                        //            animation frame. Multiple times, once per cursor.
                        eval_ selection.position.value (model.redraw());
                    }
                    selection
                }
            };

            selection.width.set_target_value(width);
            selection.edit_mode.set(do_edit);
            selection.shape.start_time.set(time);
            new_selection_map.id_map.insert(id,selection);
            new_selection_map.location_map.entry(start_line).or_default().insert(sel.start.column,id);
        }
        *selection_map = new_selection_map;
        }
        self.redraw()
    }

    /// Transforms screen position to the object (display object) coordinate system.
    fn to_object_space(&self, screen_pos:Vector2) -> Vector2 {
        let origin_world_space = Vector4(0.0,0.0,0.0,1.0);
        let origin_clip_space  = self.scene.camera().view_projection_matrix() * origin_world_space;
        let inv_object_matrix  = self.transform_matrix().try_inverse().unwrap();

        let shape        = self.scene.frp.shape.value();
        let clip_space_z = origin_clip_space.z;
        let clip_space_x = origin_clip_space.w * 2.0 * screen_pos.x / shape.width;
        let clip_space_y = origin_clip_space.w * 2.0 * screen_pos.y / shape.height;
        let clip_space   = Vector4(clip_space_x,clip_space_y,clip_space_z,origin_clip_space.w);
        let world_space  = self.scene.camera().inversed_view_projection_matrix() * clip_space;
        (inv_object_matrix * world_space).xy()
    }

    fn get_in_text_location(&self, screen_pos:Vector2) -> Location {
        let object_space = self.to_object_space(screen_pos);
        let line_index   = (-object_space.y / LINE_HEIGHT) as usize;
        let line_index   = std::cmp::min(line_index,self.lines.count() - 1);
        let div_index    = self.lines.rc.borrow()[line_index].div_index_close_to(object_space.x);
        let line         = line_index.into();
        let column       = div_index.into();
        Location(line,column)
    }

    fn init(self) -> Self {
        self.redraw();
        self
    }

    /// Redraw the text.
    fn redraw(&self) {
        let lines      = self.buffer.view_lines();
        let line_count = lines.len();
        self.lines.resize_with(line_count,|ix| self.new_line(ix));
        let lengths = lines.into_iter().enumerate().map(|(view_line_index,content)|{
            self.redraw_line(view_line_index,content)
        }).collect_vec();
        let length = lengths.into_iter().max_by(|x,y|x.partial_cmp(y).unwrap()).unwrap_or_default();
        self.frp_endpoints.source.width.emit(length);
    }

    fn redraw_line(&self, view_line_index:usize, content:String) -> f32 {
        let cursor_map = self.selection_map.borrow()
            .location_map.get(&view_line_index).cloned().unwrap_or_default();
        let line            = &mut self.lines.rc.borrow_mut()[view_line_index];
        let line_object     = line.display_object().clone_ref();
        let line_range      = self.buffer.byte_range_of_view_line_index_snapped(view_line_index.into());
        let mut line_style  = self.buffer.sub_style(line_range.start .. line_range.end).iter();
        let mut pen         = pen::Pen::new(&self.glyph_system.font);
        let mut divs        = vec![];
        let mut column      = 0.column();
        let mut last_cursor = None;
        let mut last_cursor_target = default();
        line.resize_with(content.chars().count(),||self.glyph_system.new_glyph());
        let mut iter = line.glyphs.iter_mut().zip(content.chars());
        loop {
            let next      = iter.next();
            let style     = line_style.next().unwrap_or_default();
            let chr_size  = style.size.raw;
            let char_info = next.as_ref().map(|t|pen::CharInfo::new(t.1,chr_size));
            let info      = pen.advance(char_info);

            cursor_map.get(&column).for_each(|id| {
                self.selection_map.borrow().id_map.get(id).for_each(|cursor| {
                    if cursor.edit_mode.get() {
                        let pos_y          = LINE_HEIGHT/2.0 - LINE_VERTICAL_OFFSET;
                        last_cursor        = Some(cursor.clone_ref());
                        last_cursor_target = Vector2(info.offset,pos_y);
                    }
                });
            });

            divs.push(info.offset);
            let opt_glyph = next.map(|t|t.0);
            match opt_glyph.zip(info.char) {
                Some((glyph,chr)) => {
                    let chr_bytes : Bytes = chr.len_utf8().into();
                    line_style.drop(chr_bytes - 1.bytes());
                    let glyph_info   = self.glyph_system.font.glyph_info(chr);
                    let size         = glyph_info.scale.scale(chr_size);
                    let glyph_offset = glyph_info.offset.scale(chr_size);
                    let glyph_x      = info.offset + glyph_offset.x;
                    let glyph_y      = glyph_offset.y;
                    glyph.set_position_xy(Vector2(glyph_x,glyph_y));
                    glyph.set_char(chr);
                    glyph.set_color(style.color);
                    glyph.size.set(size);
                    match &last_cursor {
                        None         => line_object.add_child(glyph),
                        Some(cursor) => {
                            cursor.right_side.add_child(glyph);
                            glyph.mod_position_xy(|p| p - last_cursor_target);
                        },
                    }
                }
                None => break
            }
            column += 1.column();
        }

        let last_offset   = divs.last().cloned().unwrap_or_default();
        let cursor_offset = last_cursor.map(|cursor| last_cursor_target.x - cursor.position().x);
        let cursor_offset = cursor_offset.unwrap_or_default();
        line.set_divs(divs);
        last_offset - cursor_offset
    }

    fn new_line(&self, index:usize) -> Line {
        let line     = Line::new(&self.logger);
        let y_offset = - ((index + 1) as f32) * LINE_HEIGHT + LINE_VERTICAL_OFFSET;
        line.set_position_y(y_offset);
        self.add_child(&line);
        line
    }

    fn copy(&self, selections:&[String]) {
        let encoded = match selections {
            []  => "".to_string(),
            [s] => s.clone(),
            lst => lst.join(RECORD_SEPARATOR),
        };
        clipboard::write_text(encoded);
    }

    fn paste(&self) {
        let paste_string = self.frp_endpoints.input.paste_string.clone_ref();
        clipboard::read_text(move |t| paste_string.emit(t));
    }

    fn decode_paste(&self, encoded:&str) -> Vec<String> {
        encoded.split(RECORD_SEPARATOR).map(|s|s.into()).collect()
    }

    fn key_to_string(&self, key:&Key) -> Option<String> {
        match key {
            Key::Character(s) => Some(s.clone()),
            Key::Enter        => Some("\n".into()),
            _                 => None
        }
    }
}

impl display::Object for AreaData {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}

impl display::Object for Area {
    fn display_object(&self) -> &display::object::Instance {
        self.data.display_object()
    }
}

impl application::command::FrpNetworkProvider for Area {
    fn network(&self) -> &frp::Network {
        &self.frp.network
    }
}

impl application::command::Provider for Area {
    fn label() -> &'static str {
        "TextArea"
    }
}

impl application::View for Area {
    fn new(app:&Application) -> Self {
        Area::new(app)
    }
}

// FIXME[WD]: Some of the shortcuts are commented out and some are assigned to strange key bindings
//            because the shortcut manager is broken. To be fixed after fixing the shortcut manager.
//            Moreover, the lines should fit 100 chars after introducing the new cmd engine.
//            Should be resolved as part of https://github.com/enso-org/ide/issues/713
impl application::shortcut::DefaultShortcutProvider for Area {
    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        use enso_frp::io::mouse;
//        vec! [ Self::self_shortcut(shortcut::Action::press (&[],&[mouse::PrimaryButton]), "set_cursor_at_mouse_position")
//        ]
        vec! [ Self::self_shortcut(shortcut::Action::press   (&[Key::ArrowLeft]                       , shortcut::Pattern::Any) , "cursor_move_left"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::ArrowRight]                      , shortcut::Pattern::Any) , "cursor_move_right"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::ArrowUp]                         , shortcut::Pattern::Any) , "cursor_move_up"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::ArrowDown]                       , shortcut::Pattern::Any) , "cursor_move_down"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta,Key::ArrowLeft]             , shortcut::Pattern::Any) , "cursor_move_left_word"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta,Key::ArrowRight]            , shortcut::Pattern::Any) , "cursor_move_right_word"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Shift,Key::ArrowLeft]            , shortcut::Pattern::Any) , "cursor_select_left"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Shift,Key::ArrowRight]           , shortcut::Pattern::Any) , "cursor_select_right"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta,Key::Shift,Key::ArrowLeft]  , shortcut::Pattern::Any) , "cursor_select_left_word"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta,Key::Shift,Key::ArrowRight] , shortcut::Pattern::Any) , "cursor_select_right_word"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Shift,Key::ArrowUp]              , shortcut::Pattern::Any) , "cursor_select_up"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Shift,Key::ArrowDown]            , shortcut::Pattern::Any) , "cursor_select_down"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Backspace]                       , shortcut::Pattern::Any) , "delete_left"),
//               Self::self_shortcut(shortcut::Action::press   (&[Key::Tab]                             , shortcut::Pattern::Any) , "increase_indentation"),
//               Self::self_shortcut(shortcut::Action::press   (&[Key::Shift,Key::Tab]                  , shortcut::Pattern::Any) , "decrease_indentation"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta,Key::Backspace]             , shortcut::Pattern::Any) , "delete_word_left"),
//               Self::self_shortcut(shortcut::Action::press   (&[Key::Escape]                          , shortcut::Pattern::Any) , "keep_oldest_cursor_only"),
               Self::self_shortcut(shortcut::Action::press   (shortcut::Pattern::Any,&[])                                       , "insert_char_of_last_pressed_key"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Shift],&[mouse::PrimaryButton])                            , "set_newest_selection_end_to_mouse_position"),
               Self::self_shortcut(shortcut::Action::double_press (&[],&[mouse::PrimaryButton])                                 , "select_word_at_cursor"),
               Self::self_shortcut(shortcut::Action::press   (&[],&[mouse::PrimaryButton])                                      , "set_cursor_at_mouse_position"),
               Self::self_shortcut(shortcut::Action::press   (&[],&[mouse::PrimaryButton])                                      , "start_newest_selection_end_follow_mouse"),
               Self::self_shortcut(shortcut::Action::release (&[],&[mouse::PrimaryButton])                                      , "stop_newest_selection_end_follow_mouse"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta],&[mouse::PrimaryButton])                             , "add_cursor_at_mouse_position"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta],&[mouse::PrimaryButton])                             , "start_newest_selection_end_follow_mouse"),
               Self::self_shortcut(shortcut::Action::release (&[Key::Meta],&[mouse::PrimaryButton])                             , "stop_newest_selection_end_follow_mouse"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta,Key::Character("a".into())],&[])                      , "select_all"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta,Key::Character("c".into())],&[])                      , "copy"),
               Self::self_shortcut(shortcut::Action::press   (&[Key::Meta,Key::Character("v".into())],&[])                      , "paste"),
//               Self::self_shortcut(shortcut::Action::release (&[Key::Meta,Key::Character("z".into())],&[])                      , "undo"),
//               Self::self_shortcut(shortcut::Action::release (&[Key::Meta,Key::Character("y".into())],&[])                      , "redo"),
//               Self::self_shortcut(shortcut::Action::release (&[Key::Meta,Key::Shift,Key::Character("z".into())],&[])           , "redo"),
//                Self::self_shortcut(shortcut::Action::press   (&[Key::Escape]                          , shortcut::Pattern::Any) , "undo"),
        ]
    }
}

// TODO[WD]: The following shortcuts are missing and its worth adding them in the future:
// undo, redo, multicursor up/down, check with different views, scrolling, parens, lines indent
