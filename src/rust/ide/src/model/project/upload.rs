use crate::prelude::*;

use crate::notification::Publisher;

use enso_protocol::binary::Connection;
use enso_protocol::language_server;
use enso_protocol::language_server::Path;
use futures::SinkExt;
use ensogl::application::shortcut::Condition::Not;
use crate::controller::graph::{NewNodeInfo, LocationHint};
use crate::model::module::{NodeMetadata, UploadingFile}; //TODO maybe from the binary protocol?

#[derive(Clone,Debug,Fail)]
#[fail(display="Error while uploading file \"{}\": {}",file_name,msg)]
pub struct Error {
    file_name : String,
    msg       : String,
}

#[derive(Clone,Debug)]
pub enum Notification {
    UploadProgress {
        file_size      : usize,
        bytes_uploaded : usize,
    },
    Error(Error)
}

pub trait DataProvider {
    fn next_chunk(&mut self) -> BoxFuture<FallibleResult<Option<Vec<u8>>>>;
}

impl<T:DataProvider + ?Sized> DataProvider for Box<T> {
    fn next_chunk(&mut self) -> BoxFuture<FallibleResult<Option<Vec<u8>>>> {
        // We reassign to type-constrained deref to be sure we won't fall the infinite recursion.
        let deref:&mut T = &mut **self;
        deref.next_chunk()
    }
}

#[derive(Clone,Debug)]
pub struct FileUploadProcess {
    logger         : Logger,
    connection     : Rc<Connection>,
    path           : Path,
    size           : usize,
    bytes_uploaded : usize,
    // TODO checksums
}

impl FileUploadProcess {
    pub fn new(parent:impl AnyLogger, connection:Rc<Connection>, path:Path, size:usize) -> Self {
        let logger         = Logger::sub(parent,"FileUploadProcess");
        let bytes_uploaded = 0;
        Self{logger,connection,path,size,bytes_uploaded}
    }

    pub fn start
    (mut self, mut data_provider:impl DataProvider + 'static) -> impl Stream<Item=Notification> {
        let (mut notification_in,notification_out) = futures::channel::mpsc::channel(5); //TODO ugly constant
        executor::global::spawn(async move {
            loop {
                match data_provider.next_chunk().await {
                    Ok(Some(data)) => {
                        info!(self.logger, "Received chunk of {self.path} of size {data.len()}");
                        self.connection.write_file(&self.path,&data); // TODO replace with write bytes.
                        self.bytes_uploaded += data.len();
                        let notification = Notification::UploadProgress {
                            file_size      : self.size,
                            bytes_uploaded : self.bytes_uploaded
                        };
                        notification_in.send(notification).await;
                    },
                    Ok(None) => {
                        if self.bytes_uploaded != self.size {
                            error!(self.logger, "The promised file size ({self.size}) and uploaded \
                                data length ({self.bytes_uploaded}) do not match. Leaving as much \
                                data as received.");
                        }
                        break;
                    }
                    Err(err) => {
                        error!(self.logger, "Error while retrieving next chunk for {self.path}: \
                            {err:?}");
                        let msg       = err.to_string();
                        let file_name = self.path.file_name().map(AsRef::<str>::as_ref).unwrap_or("").to_owned();
                        let error     = Error{msg,file_name};
                        notification_in.send(Notification::Error(error)).await;
                        break;
                    }
                }
            }
        });
        notification_out
    }
}

pub async fn create_node_from_dropped_file
( project  : &model::Project
, graph    : &controller::Graph
, position : model::module::Position
, name     : &str
, size     : usize
, data     : impl DataProvider + 'static
) -> FallibleResult {
    let logger   = Logger::new("create_node_from_dropped_file");
    let path     = create_remote_path(project,name).await?;
    let uploading_metadata = UploadingFile {
        name : name.to_owned(),
        remote_path : path.clone(),
        size,
        bytes_uploaded: 0
    };
    let metadata = NodeMetadata {
        position: Some(position),
        intended_method: None,
        uploading_file: Some(uploading_metadata.clone())
    };
    let node_info = NewNodeInfo {
        expression: format!("File.read Enso_Project.data/\"{}\"", name),
        metadata: Some(metadata.clone()),
        id: None,
        location_hint: LocationHint::End,
        introduce_pattern: true
    };
    let node       = graph.add_node(node_info)?;
    let process    = FileUploadProcess::new(&logger,project.binary_rpc(),path,size);
    let mut stream = process.start(data);
    let graph      = graph.clone_ref();
    while let Some(notification) = stream.next().await  {
        match notification {
            Notification::UploadProgress {bytes_uploaded,..} => {
                graph.module.with_node_metadata(node,Box::new(|metadata| {
                    let field = metadata.uploading_file.get_or_insert_with(|| uploading_metadata.clone());
                    field.bytes_uploaded = bytes_uploaded;
                }))?;
            }
            Notification::Error(error) => return Err(error.into())
        }
    };
    Ok(())
}

async fn create_remote_path(project:&model::Project, original_name:&str) -> FallibleResult<Path> {
    let data_path       = Path::new(project.content_root_id(),&["data"]);
    let list_response   = project.json_rpc().client.file_list(&data_path).await?;
    let files_in_data_dir:HashSet<String> = list_response.paths.into_iter().map(|f| f.take_name()).collect();
    let extension_sep   = original_name.rfind(".");
    let name_core       = extension_sep.map_or(original_name, |i| &original_name[0..i]);
    let name_ext        = extension_sep.map_or("", |i| &original_name[i..]);
    let first_candidate = std::iter::once(original_name.to_owned());
    let next_candidates = (1..).map(|num| iformat!("{name_core}_{num}{name_ext}"));
    let mut candidates  = first_candidate.chain(next_candidates);
    let picked          = candidates.find(|name| !files_in_data_dir.contains(name)).unwrap();
    Ok(data_path.append_im(picked))
}


// ============
// === Test ===
// ============

#[cfg(test)]
mod test {

}
