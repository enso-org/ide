use super::text_editor::TextEditor;

use basegl::display::world::World;

//TODO: Implement resizeable panels and panel containers?
pub struct ViewLayout {
    text_editor : TextEditor
}

impl ViewLayout {
    pub fn new(world:&World) -> Self {
        let text_editor = TextEditor::new(&world);
        Self {text_editor}
    }
}