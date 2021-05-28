use crate::prelude::*;



pub struct FileMetadata {
    pub name      : ImString,
    pub mime_type : ImString, // TODO maybe enum?
    pub size      : usize,
}

impl FileMetadata {
    pub fn from_js_file(file:&web_sys::File) -> Self {
        let name      = ImString::new(file.name());
        let size      = file.size() as usize;
        let mime_type = ImString::new(file.type_());
        FileMetadata {name,mime_type,size}
    }
}

pub struct DoppedFile {
    reader : web_sys::ReadableStreamDefaultReader,
}



pub struct DropFileManager {

}

impl DropFileManager {

}
