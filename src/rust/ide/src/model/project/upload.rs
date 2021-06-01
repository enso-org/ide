use crate::prelude::*;

use crate::notification::Publisher;

use enso_protocol::binary::Connection;
use enso_protocol::language_server::Path;
use futures::SinkExt;
use ensogl::application::shortcut::Condition::Not; //TODO maybe from the binary protocol?

#[derive(Clone,Debug,Default)]
pub enum Notification {
    UploadProgress {
        file_size      : usize,
        bytes_uploaded : usize,
    },
    Error {msg:ImString}
}


pub struct FileUploadProcess {
    logger         : Logger,
    connection     : Rc<Connection>,
    path           : Path,
    size           : usize,
    bytes_uploaded : usize,
    // TODO checksums
}


impl FileUploadProcess {
    pub fn new(parent:impl AnyLogger, connection:Rc<Connection>, path:Path) -> Self {
        let logger         = Logger::sub(parent,"FileUploadProcess");
        let size           = 0;
        let bytes_uploaded = 0;
        Self{logger,connection,path,size,bytes_uploaded}
    }

    pub fn start<ChunkFuture>
    (mut self, data_provider:impl FnMut() -> ChunkFuture) -> impl Stream<Item=Notification>
    where ChunkFuture : Future<Output=FallibleResult<Some<Vec<u8>>>> {
        let (mut notification_in,notification_out) = futures::channel::mpsc::channel(5); //TODO ugly constant
        executor::global::spawn(async move {
            loop {
                match data_provider().await {
                    Ok(Some(data)) => {
                        info!(self.logger, "Received chunk of {self.path} of size {data.len()}");
                        self.connection.write_file(&path,&data); // TODO replace with write bytes.
                        self.bytes_uploaded += data.len();
                        let notification = Notification::UploadProgress {file_size,bytes_uploaded};
                        notification_in.send(notification).await;
                    },
                    Ok(None) => {
                        if self.bytes_uploaded != self.file_size {
                            error!(self.logger, "The promised file size ({self.file_size}) \
                                and uploaded data length ({self.bytes_uploaded}) do not match. \
                                Leaving as much data as received");
                        }
                        break;
                    }
                    Err(err) => {
                        error!(self.logger, "Error while retrieving next chunk for {self.path}: \
                            {err:?}");
                        let msg = err.msg.into();
                        notification_in.send(Notification::Error {msg}).await;
                        break;
                    }
                }
            }
        });
        notification_out
    }
}


// ============
// === Test ===
// ============

#[cfg(test)]
mod test {

}
