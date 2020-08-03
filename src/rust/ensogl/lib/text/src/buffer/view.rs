#![allow(missing_docs)]

use crate::prelude::*;

pub mod movement;
pub mod selection;
pub mod word;

pub use movement::*;
pub use selection::Selection;


use crate::buffer::style::Style;
use crate::buffer::data;
use crate::buffer::data::Data;
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
    pub undo_stack : Vec<(Data,Style,selection::Group)>,
    pub redo_stack : Vec<(Data,Style,selection::Group)>,
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
        self.selection.borrow_mut().add(selection.into())
    }

    pub fn first_selection(&self) -> selection::Group {
        self.selection.borrow().first().cloned().into()
    }

    pub fn last_selection(&self) -> selection::Group {
        self.selection.borrow().last().cloned().into()
    }

    pub fn first_caret(&self) -> selection::Group {
        self.first_selection().to_carets()
    }

    pub fn last_caret(&self) -> selection::Group {
        self.last_selection().to_carets()
    }

    pub fn newest_selection(&self) -> selection::Group {
        self.selection.borrow().newest().cloned().into()
    }

    pub fn oldest_selection(&self) -> selection::Group {
        self.selection.borrow().oldest().cloned().into()
    }

    pub fn newest_caret(&self) -> selection::Group {
        self.newest_selection().to_carets()
    }

    pub fn oldest_caret(&self) -> selection::Group {
        self.oldest_selection().to_carets()
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
        selection.add(new_selection);
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
    pub fn insert(&self, text:impl Into<Data>) -> selection::Group {
        self.modify(Transform::LeftSelectionBorder,text)
    }

    fn delete_left(&self) -> selection::Group {
        self.modify(Transform::Left,"")
    }

//    /// Insert new text in the place of current selections / cursors.
//    pub fn modify(&self, movement:Transform, text:impl Into<Data>) -> selection::Group {
//        self.commit_history();
//        let text       = text.into();
//        let text_size  = text.len();
//        let mut line   = 0.line();
//        let mut offset = 0.column();
//        let mut result = selection::Group::new();
//        for rel_selection in &*self.selection.borrow() {
//            if rel_selection.start.line != line {
//                line   = rel_selection.start.line;
//                offset = 0.column();
//            }
//            let selection     = rel_selection.map(|t|t+offset);
//            let new_selection = self.moved_selection_region(movement,selection,false);
//            let text_size : Column = text.grapheme_count().into();
//            let selection_min = std::cmp::min(selection.min(),new_selection.min());
//            let selection_max = std::cmp::max(selection.max(),new_selection.max());
//            let range    : data::range::Range<Bytes>      = (self.line_col_to_offset(selection_min) .. self.line_col_to_offset(selection_max)).into();
//            let cols_diff     = selection_max.column - selection_min.column;
//            let diff          = text_size - cols_diff;
//            println!("diff {:?} {:?} {:?} {:?}",text,selection_min,selection_max,text_size);
//            println!("range {:?}",range);
//            offset += diff;
//            self.buffer.data.borrow_mut().insert(range,&text);
//            let new_selection = new_selection.map(|t|t+text_size);
//            result.add(new_selection);
//        }
//        result
//    }

    pub fn modify(&self, movement:Transform, text:impl Into<Data>) -> selection::Group {
        println!("MODIFY");
        self.commit_history();
        let text       = text.into();
        let text_size  = text.len();
        let mut result = selection::Group::new();
        let mut offset = 0.bytes();
        for rel_selection in &*self.selection.borrow() {
            println!("----");
            println!("rel_selection {:?}", rel_selection);
            let rel_selection = self.to_bytes_selection(*rel_selection);
            println!("rel_selection {:?}",rel_selection);
            let selection_b     = rel_selection.map(|t|t+offset);
            println!("selection_b {:?}",selection_b);
            let selection     = self.to_location_selection(selection_b);
            println!("selection {:?}",selection);
            let new_selection = self.moved_selection_region(movement,selection,false);
            println!("new_selection {:?}",new_selection);
            let new_selection = self.to_bytes_selection(new_selection);
            println!("new_selection {:?}",new_selection);
            let range         = range_between(selection_b,new_selection);
            println!("range {:?}",range);
            offset            += text_size - range.size();
            println!("offset {:?}",offset);
            self.buffer.data.borrow_mut().insert(range,&text);
            let new_selection = new_selection.map(|t|t+text_size);
            println!("new_selection {:?}",new_selection);
            let new_selection = self.to_location_selection(new_selection);
            println!("new_selection {:?}",new_selection);
            result.add(new_selection);
        }
        result
    }

    fn to_bytes_selection(&self, selection:Selection) -> Selection<Bytes> {
        let start = self.line_col_to_offset(selection.start).unwrap_or_default();
        let end   = self.line_col_to_offset(selection.end).unwrap_or_default();
        let id    = selection.id;
        Selection {start,end,id}
    }

    fn to_location_selection(&self, selection:Selection<Bytes>) -> Selection {
        let start = self.offset_to_location(selection.start);
        let end   = self.offset_to_location(selection.end);
        let id    = selection.id;
        Selection {start,end,id}
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

//            eval_ output.changed (model.commit_history());



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
        let max_line          = self.last_line_number();
        let line_count : Line = self.line_count().into();
        max_line.min(self.first_line_number() + line_count)
    }

    pub fn last_line_number(&self) -> Line {
        self.line_of_offset(self.data().len())
    }

    pub fn line_count(&self) -> usize {
        self.line_count.get()
    }

    pub fn line_range(&self) -> Range<Line> {
        self.first_line_number() .. self.last_view_line_number()
    }

    pub fn first_line_offset(&self) -> Bytes {
        self.offset_of_line(self.first_line_number()).unwrap() // FIXME
    }

    pub fn last_line_offset(&self) -> Bytes {
        println!("last_view_line_number {:?}",self.last_view_line_number());
        self.offset_of_line(self.last_view_line_number()).unwrap()
    }

    pub fn line_offset_range(&self) -> Range<Bytes> {
        println!("line_offset_range {:?} {:?}",self.first_line_offset(),self.last_line_offset());
        self.first_line_offset() .. self.last_line_offset()
    }

    pub fn view_end_offset(&self) -> Bytes {
        let next_line = self.last_view_line_number() + 1.line();
        println!("?? next_line, off {:?} {:?}",next_line, self.offset_of_line(next_line));
        self.offset_of_line(next_line).and_then(|t|self.prev_grapheme_offset(t)).unwrap_or_else(||self.data().len())
    }

    /// Return the offset after the last character of a given line if the line exists.
    pub fn end_offset_of_line(&self, line:Line) -> Option<Bytes> {
        let next_line  = self.last_view_line_number() + 1.line();
        let opt_result = self.offset_of_line(next_line).and_then(|t| self.prev_grapheme_offset(t));
        opt_result.or_else(|| (line <= self.last_line_number()).as_some_from(|| self.data().len()))
    }

    /// Return the offset after the last character of a given view line if the line exists.
    pub fn end_offset_of_view_line(&self, line:Line) -> Option<Bytes> {
        self.end_offset_of_line(line + self.first_line_number.get())
    }

    pub fn view_range(&self) -> Range<Bytes> {
        self.first_line_offset() .. self.view_end_offset()
    }

    pub fn offset_of_view_line(&self, view_line:Line) -> Option<Bytes> {
        let line = self.first_line_number() + view_line;
        self.offset_of_line(line)
    }

//    pub fn offset_of_view_location(&self, location:impl Into<Location>) -> Bytes {
//        let location = location.into();
//        self.offset_of_view_line(location.line) + self.line_offset_of_location_X(location)
//    }

//    pub fn line_byte_size(&self, line:Line) -> Bytes {
//        let start = self.offset_of_view_line(line);
//        let end   = self.offset_of_view_line(line + 1.line());
//        end - start
//    }

    /// Byte range of the given line.
    pub fn line_byte_range(&self, line:Line) -> Range<Bytes> {
        let start = self.offset_of_line(line);
        let end   = self.end_offset_of_line(line);
        start.and_then(|s| end.map(|e| s..e)).unwrap_or_else(|| default()..default())
    }

    /// Byte range of the given view line.
    pub fn view_line_byte_range(&self, view_line:Line) -> Range<Bytes> {
        let line = view_line + self.first_line_number.get();
        self.line_byte_range(line)
    }

//    pub fn lines(&self) -> buffer::Lines {
//        let range = self.line_offset_range();
//        self.buffer.data.borrow().data.rope.lines(range.start.raw .. range.end.raw)
//    }

    /// Return all lines of this buffer view.
    pub fn lines(&self) -> Vec<String> {
        let range        = self.view_range();
        let rope_range   = range.start.as_usize() .. range.end.as_usize();
        let mut lines    = self.buffer.borrow().lines(rope_range).map(|t|t.into()).collect_vec();
        let missing_last = lines.len() == self.last_line_number().as_usize();
        if  missing_last { lines.push("".into()) }
        lines
    }

//    pub fn get(&self, line:Line) -> String {
//        let last_view_line_number = self.line_of_offset(self.data().len());
//        let start   = self.offset_of_line(line);
//        let end     = self.offset_of_line(line+1);
//        let end     = self.buffer.text.prev_grapheme_offset(end).unwrap_or(end);
//        let content = self.buffer.text.rope.subseq(start.raw .. end.raw);
//        println!("buffer line count: {}", last_view_line_number.raw);
//        content.into()
//    }

//    fn scroll_to_cursor(&mut self, text: &Text) {
//        let end = self.sel_regions().last().unwrap().end;
//        let line = self.line_of_offset(text, end);
//        if line < self.first_line_number {
//            self.first_line_number = line;
//        } else if self.first_line_number + self.height <= line {
//            self.first_line_number = line - (self.height - 1);
//        }
//        // We somewhat arbitrarily choose the last region for setting the old-style
//        // selection state, and for scrolling it into view if needed. This choice can
//        // likely be improved.
//        self.scroll_to = Some(end);
//    }
}

impl LineOffset for ViewModel {
    fn data(&self) -> Data {
        self.buffer.data.borrow().data.clone() // FIXME
    }

    fn column_of_location(&self, line:Line, line_offset:Bytes) -> Column {
        self.column_of_location_X(line,line_offset)
    }

    fn line_col_to_offset(&self, location:Location) -> Option<Bytes> {
        self.line_offset_of_location_X2(location)
    }

    fn offset_of_line(&self,line:Line) -> Option<Bytes> {
        let max_line = (self.data().measure::<data::metric::Lines>()).into();
        if line > max_line { None } else {
            Some(self.data().offset_of_line(line))
        }
    }

    fn line_of_offset(&self,offset:Bytes) -> Line {
        self.data().line_of_offset(offset)
    }
}

impl LineOffset for ViewBuffer {
    fn data(&self) -> Data {
        self.buffer.data.borrow().data.clone() // FIXME
    }

    fn column_of_location(&self, line:Line, line_offset:Bytes) -> Column {
        self.column_of_location_X(line,line_offset)
    }

    fn line_col_to_offset(&self, location:Location) -> Option<Bytes> {
        self.line_offset_of_location_X2(location)
    }

    fn offset_of_line(&self,line:Line) -> Option<Bytes> {
        println!("offset_of_line {:?}",line);
        let max_line = (self.data().measure::<data::metric::Lines>()).into();
        println!("max_line {:?}",max_line);
        if line > max_line { None } else {
            Some(self.data().offset_of_line(line))
        }
    }

    fn line_of_offset(&self,offset:Bytes) -> Line {
        self.data().line_of_offset(offset)
    }
}



// ==================
// === LineOffset ===
// ==================

/// A trait from which lines and columns in a document can be calculated
/// into offsets inside a text an vice versa.
pub trait LineOffset {
    // use own breaks if present, or text if not (no line wrapping)

    fn data(&self) -> Data;

    fn offset_of_line(&self,line:Line) -> Option<Bytes>;


        /// Returns the visible line number containing the given offset.
    fn line_of_offset(&self, offset:Bytes) -> Line {
        self.data().line_of_offset(offset)
    }

    fn column_of_location(&self, line:Line, line_offset:Bytes) -> Column;

    // How should we count "column"? Valid choices include:
    // * Unicode codepoints
    // * grapheme clusters
    // * Unicode width (so CJK counts as 2)
    // * Actual measurement in text layout
    // * Code units in some encoding
    //
    // Of course, all these are identical for ASCII. For now we use UTF-8 code units
    // for simplicity.

    fn offset_to_location(&self, offset:Bytes) -> Location {
        let line         = self.line_of_offset(offset);
        let line_offset  = (offset - self.offset_of_line(line).unwrap());
        let column       = self.column_of_location(line,line_offset);
        Location(line,column)
    }

    fn line_col_to_offset(&self, location:Location) -> Option<Bytes>;// {
//        let mut offset = self.offset_of_line(line).saturating_add(col.value.bytes()); // fixme: raw.bytes seems wrong
//        let len = self.data().len();
//        if offset >= len {
//            offset = len;
//            if self.line_of_offset(offset) <= line {
//                return offset;
//            }
//        } else {
//            // Snap to grapheme cluster boundary
//            offset = self.data().prev_grapheme_offset(offset + 1.bytes()).unwrap_or_default();
//        }
//
//        // clamp to end of line
//        let next_line_offset = self.offset_of_line(line + 1.line());
//        if offset >= next_line_offset {
//            if let Some(prev) = self.data().prev_grapheme_offset(next_line_offset) {
//                offset = prev;
//            }
//        }
//        offset
//    }

//    /// Get the line range of a selected region.
//    fn get_line_range(&self, text: &Text, region: &Selection) -> std::ops::Range<usize> {
//        let (first_line_number, _) = self.offset_to_location(text, region.min());
//        let (mut last_line, last_col) = self.offset_to_location(text, region.max());
//        if last_col == 0 && last_line > first_line_number {
//            last_line -= 1;
//        }
//
//        first_line_number..(last_line + 1)
//    }
}
