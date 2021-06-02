use crate::prelude::*;

use crate::notification::Publisher;

use enso_protocol::binary::Connection;
use enso_protocol::language_server::Path;
use futures::SinkExt;
use ensogl::application::shortcut::Condition::Not; //TODO maybe from the binary protocol?

#[derive(Clone,Debug)]
pub enum Notification {
    UploadProgress {
        file_size      : usize,
        bytes_uploaded : usize,
    },
    Error {msg:ImString}
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
    pub fn new(parent:impl AnyLogger, connection:Rc<Connection>, path:Path) -> Self {
        let logger         = Logger::sub(parent,"FileUploadProcess");
        let size           = 0;
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
                        let msg = err.to_string().into();
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
