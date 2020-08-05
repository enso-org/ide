#![allow(missing_docs)]

use crate::prelude::*;

pub mod movement;
pub mod selection;
pub mod word;

pub use movement::*;
pub use selection::Selection;


use crate::buffer::style::Style;
use crate::buffer::data;
use crate::buffer::data::Text;
use crate::buffer::data::unit::*;
use crate::buffer::Buffer;

use enso_frp as frp;


// ==================
// === Frp Macros ===
// ==================

// FIXME: these are generic FRP utilities. To be refactored out after the API settles down.
// FIXME: They are already copy-pasted in the EnsoGL code. To be unified and refactored.
macro_rules! define_frp {
    (
        Input  { $($in_field  : ident : $in_field_type  : ty),* $(,)? }
        Output { $($out_field : ident : $out_field_type : ty),* $(,)? }
    ) => {
        #[derive(Debug,Clone,CloneRef)]
        pub struct Frp {
            pub network : frp::Network,
            pub input   : FrpInputs,
            pub output  : FrpOutputs,
        }

        impl Frp {
            pub fn new(network:frp::Network, input:FrpInputs, output:FrpOutputs) -> Self {
                Self {network,input,output}
            }
        }

        #[derive(Debug,Clone,CloneRef)]
        pub struct FrpInputs {
            $(pub $in_field : frp::Source<$in_field_type>),*
        }

        impl FrpInputs {
            pub fn new(network:&frp::Network) -> Self {
                frp::extend! { network
                    $($in_field <- source();)*
                }
                Self { $($in_field),* }
            }
        }

        #[derive(Debug,Clone,CloneRef)]
        pub struct FrpOutputsSource {
            $($out_field : frp::Any<$out_field_type>),*
        }

        #[derive(Debug,Clone,CloneRef)]
        pub struct FrpOutputs {
            source : FrpOutputsSource,
            $(pub $out_field : frp::Stream<$out_field_type>),*
        }

        impl FrpOutputsSource {
            pub fn new(network:&frp::Network) -> Self {
                frp::extend! { network
                    $($out_field <- any(...);)*
                }
                Self {$($out_field),*}
            }
        }

        impl FrpOutputs {
            pub fn new(network:&frp::Network) -> Self {
                let source = FrpOutputsSource::new(network);
                $(let $out_field = source.$out_field.clone_ref().into();)*
                Self {source,$($out_field),*}
            }
        }
    };
}



// =================
// === Constants ===
// =================

/// When paging through a file, the number of lines from the previous page that will also be visible
/// in the next.
const SCROLL_OVERLAP : isize = 2;

/// Default visible line count in a new buffer view.
const DEFAULT_LINE_COUNT : usize = 10;


#[derive(Debug,Clone,Default)]
pub struct HistoryData {
    pub undo_stack : Vec<(Text,Style,selection::Group)>,
    pub redo_stack : Vec<(Text,Style,selection::Group)>,
}

#[derive(Debug,Clone,CloneRef,Default)]
pub struct History {
    pub data : Rc<RefCell<HistoryData>>
}



// ==================
// === ViewBuffer ===
// ==================

/// Specialized form of `Buffer` with view-related information, such as selection. This form of
/// buffer is mainly used by `View`, but can also be combined with other `ViewBuffer`s to display
/// cursors, selections, and edits of several users at the same time.
#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct ViewBuffer {
    pub buffer            : Buffer,
    pub selection         : Rc<RefCell<selection::Group>>,
    pub next_selection_id : Rc<Cell<usize>>,
    pub history           : History,
}

impl Deref for ViewBuffer {
    type Target = Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl From<Buffer> for ViewBuffer {
    fn from(buffer:Buffer) -> Self {
        let selection         = default();
        let next_selection_id = default();
        let history           = default();
        Self {buffer,selection,next_selection_id,history}
    }
}

impl From<&Buffer> for ViewBuffer {
    fn from(buffer:&Buffer) -> Self {
        buffer.clone_ref().into()
    }
}

impl Default for ViewBuffer {
    fn default() -> Self {
        Buffer::default().into()
    }
}

// FIXME: Make all these utils private, and use FRP to control the model instead.
impl ViewBuffer {

    fn commit_history(&self) {
        let data      = self.buffer.data();
        let style     = self.buffer.style();
        let selection = self.selection.borrow().clone();
        self.history.data.borrow_mut().undo_stack.push((data,style,selection));
    }

    fn undo(&self) -> Option<selection::Group> {
        let item      = self.history.data.borrow_mut().undo_stack.pop();
        item.map(|(data,style,selection)| {
            println!("SETTING DATA: {:?}", data);
            self.buffer.set_data(data);
            self.buffer.set_style(style);
            selection
        })
    }

    /// Add a new selection to the current view.
    pub fn add_selection(&self, selection:impl Into<Selection>) {
        self.selection.borrow_mut().merge(selection.into())
    }

    pub fn first_selection(&self) -> selection::Group {
        self.selection.borrow().first().cloned().into()
    }

    pub fn last_selection(&self) -> selection::Group {
        self.selection.borrow().last().cloned().into()
    }

    pub fn first_caret(&self) -> selection::Group {
        self.first_selection().snap_selections_to_start()
    }

    pub fn last_caret(&self) -> selection::Group {
        self.last_selection().snap_selections_to_start()
    }

    pub fn newest_selection(&self) -> selection::Group {
        self.selection.borrow().newest().cloned().into()
    }

    pub fn oldest_selection(&self) -> selection::Group {
        self.selection.borrow().oldest().cloned().into()
    }

    pub fn newest_caret(&self) -> selection::Group {
        self.newest_selection().snap_selections_to_start()
    }

    pub fn oldest_caret(&self) -> selection::Group {
        self.oldest_selection().snap_selections_to_start()
    }

    /// Add a new cursor for the given byte offset.
    pub fn add_cursor_old(&self, location:Location) {
        let id = self.next_selection_id.get();
        self.next_selection_id.set(id+1);
        self.add_selection(Selection::new_cursor(location,id))
    }

    pub fn new_cursor(&self, location:Location) -> Selection {
        let id = self.next_selection_id.get();
        self.next_selection_id.set(id+1);
        Selection::new_cursor(location,id)
    }

    pub fn add_cursor(&self, location:Location) -> selection::Group {
        let mut selection = self.selection.borrow().clone();
        let new_selection = self.new_cursor(location);
        selection.merge(new_selection);
        selection
    }

    pub fn set_newest_selection_end(&self, location:Location) -> selection::Group {
        let mut group = self.selection.borrow().clone();
        group.newest_mut().for_each(|s| s.end=location);
        group
    }

    pub fn set_oldest_selection_end(&self, location:Location) -> selection::Group {
        let mut group = self.selection.borrow().clone();
        group.oldest_mut().for_each(|s| s.end=location);
        group
    }

    /// Insert new text in the place of current selections / cursors.
    pub fn insert(&self, text:impl Into<Text>) -> selection::Group {
        self.modify(Transform::LeftSelectionBorder,text)
    }

    fn delete_left(&self) -> selection::Group {
        self.modify(Transform::Left,"")
    }

    /// Generic buffer modify utility. First, it transforms all selections with the provided
    /// `transform`, and then it replaces the resulting selection diff with the provided `text`.
    /// See its usages across the file to learn more.
    ///
    /// ## Implementation details.
    /// This function converts all selections to byte-based ones first, and then applies all
    /// modification rules. This way, it can work in an 1D byte-based space (as opposed to 2D
    /// location-based space), which makes handling multiple cursors much easier.
    pub fn modify(&self, transform:Transform, text:impl Into<Text>) -> selection::Group {
        self.commit_history();
        let text                    = text.into();
        let text_byte_size          = text.byte_size();
        let mut new_selection_group = selection::Group::new();
        let mut byte_offset         = 0.bytes();
        for rel_byte_selection in self.byte_selections() {
            let byte_selection     = rel_byte_selection.map(|t|t+byte_offset);
            let selection          = self.to_location_selection(byte_selection);
            let new_selection      = self.moved_selection_region(transform,selection,false);
            let new_byte_selection = self.to_bytes_selection(new_selection);
            let byte_range         = range_between(byte_selection,new_byte_selection);
            byte_offset           += text_byte_size - byte_range.size();
            self.buffer.data.borrow_mut().insert(byte_range,&text);
            let new_byte_selection = new_byte_selection.map(|t|t+text_byte_size);
            let new_selection      = self.to_location_selection(new_byte_selection);
            new_selection_group.merge(new_selection);
        }
        new_selection_group
    }

    fn byte_selections(&self) -> Vec<Selection<Bytes>> {
        self.selection.borrow().iter().map(|s|self.to_bytes_selection(*s)).collect()
    }

    fn to_bytes_selection(&self, selection:Selection) -> Selection<Bytes> {
        let start = self.byte_offset_from_location_snapped(selection.start);
        let end   = self.byte_offset_from_location_snapped(selection.end);
        let id    = selection.id;
        Selection::new(start,end,id)
    }

    fn to_location_selection(&self, selection:Selection<Bytes>) -> Selection {
        let start = self.offset_to_location(selection.start);
        let end   = self.offset_to_location(selection.end);
        let id    = selection.id;
        Selection::new(start,end,id)
    }

    fn data(&self) -> Text {
        self.buffer.data.borrow().text.clone() // FIXME
    }

//    fn line_col_to_offset(&self, location:Location) -> Option<Bytes> {
//        self.line_offset_of_location_X2(location)
//    }



    fn offset_to_location(&self, offset:Bytes) -> Location {
        let line         = self.line_index_from_byte_offset_snapped(offset);
        let line_offset  = (offset - self.byte_offset_from_line_index(line).unwrap());
        let column       = self.column_from_line_index_and_in_line_byte_offset_snapped(line,line_offset);
        Location(line,column)
    }
}

fn range_between(a:Selection<Bytes>, b:Selection<Bytes>) -> data::range::Range<Bytes> {
    let min = std::cmp::min(a.min(),b.min());
    let max = std::cmp::max(a.max(),b.max());
    (min .. max).into()
}




// ===========
// === FRP ===
// ===========

define_frp! {
    Input {
        cursors_move               : Option<Transform>,
        cursors_select             : Option<Transform>,
        set_cursor                 : Location,
        add_cursor                 : Location,
        set_newest_selection_end   : Location,
        set_oldest_selection_end   : Location,
        insert                     : String,
        remove_all_cursors         : (),
        delete_left                : (),
        delete_word_left           : (),
        clear_selection            : (),
        keep_first_selection_only  : (),
        keep_last_selection_only   : (),
        keep_first_caret_only      : (),
        keep_last_caret_only       : (),
        keep_oldest_selection_only : (),
        keep_newest_selection_only : (),
        keep_oldest_caret_only     : (),
        keep_newest_caret_only     : (),
        undo                       : (),
        redo                       : (),
    }

    Output {
        edit_selection     : selection::Group,
        non_edit_selection : selection::Group,
        changed            : (),
    }
}



// ============
// === View ===
// ============

/// View for a region of a buffer. There are several cases where multiple views share the same
/// buffer, including displaying the buffer in separate tabs or displaying multiple users in the
/// same file (keeping a view per user and merging them visually).
#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct View {
    model   : ViewModel,
    pub frp : Frp,
}

impl Deref for View {
    type Target = ViewModel;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl View {
    /// Constructor.
    pub fn new(view_buffer:impl Into<ViewBuffer>) -> Self {
        let network = frp::Network::new();
        let model   = ViewModel::new(&network,view_buffer);
        let input   = model.frp.clone_ref();
        let output  = FrpOutputs::new(&network);

        frp::extend! { network

            selection_on_insert <- input.insert.map(f!((s) model.insert(s)));
            output.source.changed <+ selection_on_insert.constant(());

            selection_on_delete_left <- input.delete_left.map(f_!(model.delete_left()));
            output.source.changed <+ selection_on_delete_left.constant(());

            selection_on_move  <- input.cursors_move.map(f!((t) model.moved_selection2(*t,false)));
            selection_on_mod   <- input.cursors_select.map(f!((t) model.moved_selection2(*t,true)));
            selection_on_clear <- input.clear_selection.constant(default());
            selection_on_keep_last <- input.keep_last_selection_only.map(f_!(model.last_selection()));
            selection_on_keep_first <- input.keep_first_selection_only.map(f_!(model.first_selection()));
            selection_on_keep_last_caret <- input.keep_last_caret_only.map(f_!(model.last_caret()));
            selection_on_keep_first_caret <- input.keep_first_caret_only.map(f_!(model.first_caret()));

            selection_on_keep_last <- input.keep_newest_selection_only.map(f_!(model.newest_selection()));
            selection_on_keep_first <- input.keep_oldest_selection_only.map(f_!(model.oldest_selection()));
            selection_on_keep_last_caret <- input.keep_newest_caret_only.map(f_!(model.newest_caret()));
            selection_on_keep_first_caret <- input.keep_oldest_caret_only.map(f_!(model.oldest_caret()));

            selection_on_set_cursor <- input.set_cursor.map(f!([model](t) model.new_cursor(*t).into()));
            selection_on_add_cursor <- input.add_cursor.map(f!([model](t) model.add_cursor(*t)));
            selection_on_set_newest_end <- input.set_newest_selection_end.map(f!([model](t) model.set_newest_selection_end(*t)));
            selection_on_set_oldest_end <- input.set_oldest_selection_end.map(f!([model](t) model.set_oldest_selection_end(*t)));

            selection_on_remove_all <- input.remove_all_cursors.map(|_| default());

            selection_on_undo <= input.undo.map(f_!(model.undo()));
//            output.source.changed <+ selection_on_undo.constant(());
            output.source.edit_selection     <+ selection_on_undo;

            output.source.non_edit_selection <+ selection_on_move;
            output.source.non_edit_selection <+ selection_on_mod;
            output.source.edit_selection     <+ selection_on_clear;
            output.source.non_edit_selection <+ selection_on_keep_last;
            output.source.non_edit_selection <+ selection_on_keep_first;
            output.source.non_edit_selection <+ selection_on_keep_last_caret;
            output.source.non_edit_selection <+ selection_on_keep_first_caret;
            output.source.non_edit_selection <+ selection_on_keep_last_caret;
            output.source.non_edit_selection <+ selection_on_keep_first_caret;
            output.source.non_edit_selection <+ selection_on_set_cursor;
            output.source.non_edit_selection <+ selection_on_add_cursor;
            output.source.non_edit_selection <+ selection_on_set_newest_end;
            output.source.non_edit_selection <+ selection_on_set_oldest_end;
            output.source.edit_selection     <+ selection_on_insert;
            output.source.edit_selection     <+ selection_on_delete_left;
            output.source.non_edit_selection <+ selection_on_remove_all;


            eval output.source.edit_selection ((t) model.set_selection(t));
            eval output.source.non_edit_selection ((t) model.set_selection(t));

        }
        let frp = Frp::new(network,input,output);
        Self {frp,model}
    }
}

impl Default for View {
    fn default() -> Self {
        Self::new(ViewBuffer::default())
    }
}



// =================
// === ViewModel ===
// =================

/// Internal model for the `View`.
#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct ViewModel {
    pub frp           : FrpInputs,
    pub view_buffer   : ViewBuffer,
    first_line_number : Rc<Cell<Line>>,
    line_count        : Rc<Cell<usize>>,
}

impl Deref for ViewModel {
    type Target = ViewBuffer;
    fn deref(&self) -> &Self::Target {
        &self.view_buffer
    }
}

impl ViewModel {
    /// Constructor.
    pub fn new(network:&frp::Network, view_buffer:impl Into<ViewBuffer>) -> Self {
        let frp               = FrpInputs::new(network);
        let view_buffer       = view_buffer.into();
        let first_line_number = default();
        let line_count        = Rc::new(Cell::new(DEFAULT_LINE_COUNT));
        Self {frp,view_buffer,first_line_number,line_count}
    }
}

impl ViewModel {
    /// Set the selection to a new value.
    pub fn set_selection(&self, selection:&selection::Group) {
        *self.selection.borrow_mut() = selection.clone();
    }

    /// Return all active selections.
    pub fn selections(&self) -> selection::Group {
        self.selection.borrow().clone()
    }

    // FIXME: rename
    fn moved_selection2(&self, movement:Option<Transform>, modify:bool) -> selection::Group {
        movement.map(|t| self.moved_selection(t,modify)).unwrap_or_default()
    }

    /// Computes the actual desired amount of scrolling (generally slightly less than the height of
    /// the viewport, to allow overlap).
    fn page_scroll_height(&self) -> isize {
        std::cmp::max(self.line_count.get() as isize - SCROLL_OVERLAP, 1)
    }

    pub fn first_line_number(&self) -> Line {
        self.first_line_number.get()
    }

    pub fn last_view_line_number(&self) -> Line {
        let max_line          = self.last_line_index();
        let line_count : Line = self.line_count().into();
        max_line.min(self.first_line_number() + line_count)
    }

    pub fn line_count(&self) -> usize {
        self.line_count.get()
    }

    pub fn line_range(&self) -> Range<Line> {
        self.first_line_number() .. self.last_view_line_number()
    }

    pub fn first_line_offset(&self) -> Bytes {
        self.byte_offset_from_line_index(self.first_line_number()).unwrap() // FIXME
    }

    pub fn last_line_offset(&self) -> Bytes {
        self.byte_offset_from_line_index(self.last_view_line_number()).unwrap()
    }

    pub fn line_offset_range(&self) -> Range<Bytes> {
        self.first_line_offset() .. self.last_line_offset()
    }

    pub fn view_end_offset(&self) -> Bytes {
        let next_line = self.last_view_line_number() + 1.line();
        self.byte_offset_from_line_index(next_line).ok().and_then(|t|self.prev_grapheme_offset(t)).unwrap_or_else(||self.data().byte_size())
    }

    pub fn end_offset(&self) -> Bytes {
        self.data().byte_size()
    }

    pub fn end_location(&self) -> Location {
        self.offset_to_location(self.end_offset())
    }



    /// Return the offset after the last character of a given view line if the line exists.
    pub fn end_offset_of_view_line(&self, line:Line) -> Option<Bytes> {
        self.end_byte_offset_from_line_index(line + self.first_line_number.get()).ok()
    }

    pub fn view_range(&self) -> Range<Bytes> {
        self.first_line_offset() .. self.view_end_offset()
    }

    pub fn offset_of_view_line(&self, view_line:Line) -> Option<Bytes> {
        let line = self.first_line_number() + view_line;
        self.byte_offset_from_line_index(line).ok()
    }

    pub fn clamp_selection(&self, selection:Selection) -> Selection {
        let min_line = 0.line();
        let max_line = self.last_line_index();
        let max_loc  = self.end_location();
        let start    = selection.start;
        let start    = if selection.start.line < min_line { default() } else { start };
        let start    = if selection.start.line > max_line { max_loc   } else { start };
        let end      = selection.end;
        let end      = if selection.end.line   < min_line { default() } else { end };
        let end      = if selection.end.line   > max_line { max_loc   } else { end };
        selection.with_start(start).with_end(end)
    }

    /// Byte range of the given line.
    pub fn line_byte_range(&self, line:Line) -> Range<Bytes> {
        let start = self.byte_offset_from_line_index(line);
        let end   = self.end_byte_offset_from_line_index(line);
        start.and_then(|s| end.map(|e| s..e)).unwrap_or_else(|_| default()..default())
    }

    /// Byte range of the given view line.
    pub fn view_line_byte_range(&self, view_line:Line) -> Range<Bytes> {
        let line = view_line + self.first_line_number.get();
        self.line_byte_range(line)
    }

    /// Return all lines of this buffer view.
    pub fn lines(&self) -> Vec<String> {
        let range        = self.view_range();
        let rope_range   = range.start.as_usize() .. range.end.as_usize();
        let mut lines    = self.buffer.borrow().lines(rope_range).map(|t|t.into()).collect_vec();
        let missing_last = lines.len() == self.last_line_index().as_usize();
        if  missing_last { lines.push("".into()) }
        lines
    }

}
