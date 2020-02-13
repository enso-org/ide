/// A single set of edits. All edits use indices relative to the document's
    /// state from before any edits being applied.
pub type Edits = Vec<Edit>;

/// External context for this controller (underlying controller).
#[derive(Clone,Debug)]
pub enum Context {
    /// Controller for the Luna module that we are displaying.
    TextFromModule(module::StrongHandle),
    /// Controller for the project with the non-Luna file we are displaying.
    PlainTextFile(project::StrongHandle),
}

/// Events sent from text controller to its view.
#[derive(Clone,Debug)]
pub enum EventToView {
    /// File contents needs to be set to the following due to
    /// synchronization with external state.
    SetNewContent(String),
}

/// Edit action on the text document that replaces text on given span with
/// a new one.
#[derive(Clone,Debug)]
pub struct Edit {
    /// Replaced range begin.
    pub from     : usize,
    /// Replaced range end (after last replaced character).
    /// If same value as `from` this is insert operation.
    pub to       : usize,
    /// Text to be placed. May be empty to erase portion of text.
    pub new_text : String,
}

/// Data stored by the text controller.
#[derive(Clone,Debug)]
pub struct Data {
    /// Context, i.e. entity that we can query for externally-synchronized
    /// text content.
    pub context    : Context,
    /// Sink where we put events to be consumed by the view.
    pub tx_to_view : futures::channel::mpsc::UnboundedSender<EventToView>
}

impl Data {
    /// Method called by the context when the file was externally modified.
    /// (externally, as in not by the view we are connected with)
    pub async fn file_externally_modified(&mut self) -> FallibleResult<()> {
        let new_text = match &self.context {
            Context::TextFromModule(module) =>
                module.fetch_text().await?,
            Context::PlainTextFile(_project) =>
            // TODO [mwu] fetch the text directly through project
            //      manager or the file manager (whatever is deemed to
            //      be more appropriate as a context provider here)
                todo!(),
        };
        let event = EventToView::SetNewContent(new_text);
        self.tx_to_view.unbounded_send(event)?;
        Ok(())
    }

    /// View can at any point request setting up the channel, in such case
    /// any previous channel is abandoned and subsequent event will be
    /// obtainable through the returned receiver.
    pub fn setup_stream_to_view(&mut self) -> futures::channel::mpsc::UnboundedReceiver<EventToView> {
        let (tx,rx) = futures::channel::mpsc::unbounded();
        self.tx_to_view = tx;
        rx
    }
}

make_handles!(Data);