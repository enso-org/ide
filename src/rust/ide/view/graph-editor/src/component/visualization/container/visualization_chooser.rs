use crate::component::visualization;
use ensogl_gui_list_view::entry::Model;
use ensogl_gui_list_view as list_view;


// ======================
// VisualisationPathList
// ======================


#[derive(Clone,Debug,Default)]
pub struct VisualisationPathList {
    pub content: Vec<visualization::Path>
}

impl From<Vec<visualization::Path>> for VisualisationPathList {
    fn from(content:Vec<visualization::Path>) -> Self {
        Self{content}
    }
}

impl list_view::entry::ModelProvider for VisualisationPathList {
    fn entry_count(&self) -> usize {
        self.content.len()
    }

    fn get(&self, id:usize) -> Option<Model> {
        let path  = self.content.get(id)?;
        let label = format!("{}", path);
        println!("{}", label);
        Some(list_view::entry::Model::new(label))
    }
}

