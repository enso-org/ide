use crate::prelude::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use js_sys::Uint8Array;

#[wasm_bindgen]
extern "C" {
    type ReadableStreamDefaultReader;

    #[wasm_bindgen(method)]
    fn read(this: &ReadableStreamDefaultReader) -> js_sys::Promise;
}

#[derive(Clone,CloneRef,Default,Derivative)]
#[derivative(Debug)]
pub struct File {
    pub name      : ImString,
    pub mime_type : ImString, // TODO maybe enum?
    pub size      : usize,
    #[derivative(Debug="ignore")]
    reader        : Rc<Option<ReadableStreamDefaultReader>>,
}

impl File {
    pub fn from_js_file(file:&web_sys::File) -> Result<Self, ensogl_system_web::Error> {
        let name      = ImString::new(file.name());
        let size      = file.size() as usize;
        let mime_type = ImString::new(file.type_());
        let blob      = AsRef::<web_sys::Blob>::as_ref(file);
        let js_reader = blob.stream().get_reader();
        let reader    = Rc::new(Some(js_reader.dyn_into::<ReadableStreamDefaultReader>()?));
        Ok(File {name,mime_type,size,reader})
    }

    pub async fn read_chunk(&self) -> Result<Option<Vec<u8>>, ensogl_system_web::Error> {
        if let Some(reader) = &*self.reader {
            let js_result = JsFuture::from(reader.read()).await?;
            let is_done   = js_sys::Reflect::get(&js_result, &"done".into())?.as_bool().unwrap();
            if is_done {
                Ok(None)
            } else {
                let chunk = js_sys::Reflect::get(&js_result, &"value".into())?;
                let data  = chunk.dyn_into::<Uint8Array>()?.to_vec();
                Ok(Some(data))
            }
        } else {
            Ok(None)
        }

    }
}


type DropClosure =  Closure<dyn Fn(web_sys::DragEvent)>;

pub struct DropFileManager {
    logger        : Logger,
    network       : enso_frp::Network,
    file_received : enso_frp::Source<File>,
    callback      : DropClosure,
}

impl DropFileManager {
    pub fn new(target:&web_sys::EventTarget) -> Self {
        let logger        = Logger::new("DropFileManager");
        let network       = enso_frp::Network::new("DropFileManager");
        enso_frp::extend! { network
            file_received <- source();
        }

        let callback:DropClosure = Closure::wrap(Box::new(f!([logger,file_received](event:web_sys::DragEvent) {
            let opt_files = event.data_transfer().and_then(|t| t.files());
            if let Some(js_files) = opt_files {
                let js_files_iter = (0..js_files.length()).filter_map(|i| js_files.get(i));
                let files_iter    = js_files_iter.map(|f| File::from_js_file(&f));
                for file in files_iter {
                    match file {
                        Ok(file) => file_received.emit(file),
                        Err(err) => {
                            error!(logger, "Error when processing dropped file: {err:?}");
                        }
                    }
                }
            }
        })));
        let js_closure = callback.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("drop",js_closure).unwrap();
        Self {logger,network,file_received,callback}
    }

    pub fn file_received(&self) -> &enso_frp::Source<File> { &self.file_received }
}
