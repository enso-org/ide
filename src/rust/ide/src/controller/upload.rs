//! The module with all handlers required by uploading dragged and dropped files on IDE.

use crate::prelude::*;

use crate::controller::graph::LocationHint;
use crate::controller::graph::NewNodeInfo;
use crate::model::module::NodeMetadata;
use crate::model::module::Position;
use crate::model::module::UploadingFile;

use enso_protocol::binary;
use enso_protocol::language_server;
use enso_protocol::language_server::FileSystemObject;
use enso_protocol::language_server::Path;
use json_rpc::error::RpcError;
use sha3::Digest;
use enso_protocol::types::Sha3_224;



// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[derive(Clone,Debug,Fail)]
#[fail(display="Wrong checksum of uploaded file: {}, local checksum is {}",remote,local)]
pub struct ChecksumMismatch {
    pub remote : Sha3_224,
    pub local  : Sha3_224,
}



// =================
// === Constants ===
// =================

const DATA_DIR_NAME               : &str = "data";
const CONTENT_ROOT_NOT_FOUND_CODE : i64  = 1001;
const FILE_NOT_FOUND_CODE         : i64  = 1003;



// ====================
// === DataProvider ===
// ====================

/// Trait allowing reading specific file content chunk by chunk.
pub trait DataProvider {
    /// Return a future providing the next chunk of file data.
    ///
    /// Returns [`None`] if the whole file's content has been read. The upload handlers defined in
    /// this module ([`NodeFromDroppedFileHandler`] and [`FileUploadProcess`]) will not call this
    /// method before fully uploading and freeing the previously read chunk.
    fn next_chunk(&mut self) -> BoxFuture<FallibleResult<Option<Vec<u8>>>>;
}



// =========================
// === FileUploadProcess ===
// =========================

/// Information about file-to-upload.
#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub struct FileToUpload<DataProvider> {
    pub name : String,
    pub size : usize,
    pub data : DataProvider
}

/// The handler of uploading a given file to the specific location using the Language Server's file
/// API.
#[derive(Clone,Debug)]
pub struct FileUploadProcess<DataProvider> {
    logger          : Logger,
    bin_connection  : Rc<binary::Connection>,
    json_connection : Rc<language_server::Connection>,
    file            : FileToUpload<DataProvider>,
    remote_path     : Path,
    bytes_uploaded  : usize,
    checksum        : sha3::Sha3_224,
}

impl<DP:DataProvider> FileUploadProcess<DP> {
    /// Constructor.
    pub fn new
    ( parent          : impl AnyLogger
    , file            : FileToUpload<DP>
    , bin_connection  : Rc<binary::Connection>
    , json_connection : Rc<language_server::Connection>
    , remote_path : Path
    ) -> Self {
        let logger         = Logger::sub(parent,"FileUploadProcess");
        let bytes_uploaded = 0;
        let checksum       = sha3::Sha3_224::new();
        Self {logger,bin_connection,json_connection,file,remote_path,bytes_uploaded,checksum}
    }

    /// Upload next chunk. Returns true if all data has been uploaded.
    ///
    /// After uploading, the checksum of the uploaded file is compared with the file content digest,
    /// and an error is returned if they do not match.
    pub async fn upload_chunk(&mut self) -> FallibleResult<bool> {
        match self.file.data.next_chunk().await {
            Ok(Some(data)) => {
                info!(self.logger, "Received chunk of {self.file.name} of size {data.len()}");
                trace!(self.logger, "Data: {data:?}");
                self.bin_connection.write_file(&self.remote_path,&data).await?; // TODO replace with write bytes.
                self.checksum.input(&data);
                self.bytes_uploaded += data.len();
                Ok(false)
            },
            Ok(None) => {
                if self.bytes_uploaded != self.file.size {
                    error!(self.logger, "The promised file size ({self.file.size}) and uploaded \
                        data length ({self.bytes_uploaded}) do not match. Leaving as much data as \
                        received.");
                    self.bytes_uploaded = self.file.size;
                }
                self.check_checksum().await?;
                Ok(true)
            }
            Err(err) => Err(err),
        }
    }

    async fn check_checksum(&mut self) -> FallibleResult {
        let remote = self.json_connection.file_checksum(&self.remote_path).await?.checksum;
        let local  = std::mem::take(&mut self.checksum).into();
        if remote != local {
            Err(ChecksumMismatch {remote,local}.into())
        } else {
            Ok(())
        }
    }
}



// ==================================
// === NodeFromDroppedFileHandler ===
// ==================================

/// The handler for nodes created by dragging and dropping files into IDE.
///
/// It is responsible for creating node, uploading file and updating the node's metadata.
#[derive(Clone,CloneRef,Debug)]
pub struct NodeFromDroppedFileHandler {
    logger   : Logger,
    project  : model::Project,
    graph    : controller::Graph
}

impl NodeFromDroppedFileHandler {
    /// Constructor
    pub fn new(parent:impl AnyLogger, project:model::Project, graph:controller::Graph) -> Self {
        let logger = Logger::sub(parent,"NodeFromDroppedFileHandler");
        Self{logger,project,graph}
    }

    /// Create a node from dropped file and start uploading file.
    ///
    /// The function returns once the node is created; the uploading process is scheduled in the
    /// global executor. The node's metadata will be updated with the uploading progress and
    /// error messages if any.
    pub fn create_node_and_start_uploading
    (self, file:FileToUpload<impl DataProvider + 'static>, position:Position) -> FallibleResult {
        let node = self.graph.add_node(Self::new_node_info(&file,position))?;
        executor::global::spawn(async move {
            if let Err(err) = self.upload_file(node,file).await {
                error!(self.logger, "Error while uploading file: {err}");
                self.update_metadata(node, |md| md.error = Some(err.to_string()));
            }
        });
        Ok(())
    }

    fn new_node_info<DP>(file:&FileToUpload<DP>, position:Position) -> NewNodeInfo {
        NewNodeInfo {
            expression        : Self::uploading_node_expression(&file.name),
            metadata          : Some(Self::metadata_of_new_node(file,position)),
            id                : None,
            location_hint     : LocationHint::End,
            introduce_pattern : true
        }
    }

    fn metadata_of_new_node<DP>(file:&FileToUpload<DP>, position:Position) -> NodeMetadata {
        let uploading_metadata = UploadingFile {
            name           : file.name.clone(),
            remote_name    : None,
            size           : file.size,
            bytes_uploaded : 0,
            error          : None,
        };
        NodeMetadata {
            position        : Some(position),
            intended_method : None,
            uploading_file  : Some(uploading_metadata)
        }
    }

    async fn upload_file
    (&self, node:ast::Id, file:FileToUpload<impl DataProvider>) -> FallibleResult {
        self.ensure_data_directory_exists().await?;
        let remote_name = self.establish_remote_file_name(&file.name).await?;
        self.update_metadata(node, |md| md.remote_name = Some(remote_name.clone()));
        self.graph.set_expression(node,Self::uploading_node_expression(&remote_name))?;
        let file_size       = file.size;
        let remote_path     = self.data_path().append_im(&remote_name);
        let bin_connection  = self.project.binary_rpc();
        let json_connection = self.project.json_rpc();
        let mut process     = FileUploadProcess::new
            (&self.logger,file,bin_connection,json_connection,remote_path);

        while process.bytes_uploaded < file_size {
            process.upload_chunk().await?;
            self.update_metadata(node, |md| md.bytes_uploaded = process.bytes_uploaded);
        }
        self.graph.set_expression(node,Self::uploaded_node_expression(&remote_name))?;
        if let Err(err) = self.graph.module.with_node_metadata(node, Box::new(|md| md.uploading_file = None)) {
            warning!(self.logger, "Cannot remove uploading metadata: {err}");
        }
        Ok(())
    }

    fn update_metadata(&self, node:ast::Id, f:impl FnOnce(&mut UploadingFile)) {
        let update_md = Box::new(|md:&mut NodeMetadata| {
            if let Some(uploading_md) = &mut md.uploading_file {
                f(uploading_md)
            } else {
                warning!(self.logger, "Cannot update upload progress: Metadata are missing");
            }
        });
        if let Err(err) = self.graph.module.with_node_metadata(node,update_md) {
            warning!(self.logger, "Cannot update upload progress: {err}");
        }
    }

    async fn establish_remote_file_name(&self, original_name:&str) -> FallibleResult<String> {
        let list_response     = self.project.json_rpc().client.file_list(&self.data_path()).await?;
        let files_list        = list_response.paths.into_iter().map(|f| f.take_name());
        let files_in_data_dir = files_list.collect::<HashSet<String>>();
        let extension_sep     = original_name.rfind('.');
        let name_core         = extension_sep.map_or(original_name, |i| &original_name[0..i]);
        let name_ext          = extension_sep.map_or("", |i| &original_name[i..]);
        let first_candidate   = std::iter::once(original_name.to_owned());
        let next_candidates   = (1..).map(|num| iformat!("{name_core}_{num}{name_ext}"));
        let mut candidates    = first_candidate.chain(next_candidates);
        Ok(candidates.find(|name| !files_in_data_dir.contains(name)).unwrap())
    }

    async fn ensure_data_directory_exists(&self) -> FallibleResult {
        if !self.data_directory_exists().await? {
            let to_create = FileSystemObject::Directory {
                name : DATA_DIR_NAME.to_owned(),
                path : Path::new_root(self.project.content_root_id())
            };
            self.project.json_rpc().create_file(&to_create).await?
        }
        Ok(())
    }

    async fn data_directory_exists(&self) -> FallibleResult<bool> {
        let path     = self.data_path();
        let dir_info = self.project.json_rpc().file_info(&path).await;
        match dir_info {
            Ok(info) => Ok(matches!(info.attributes.kind, FileSystemObject::Directory {..})),
            Err(RpcError::RemoteError(err))
                if err.code == FILE_NOT_FOUND_CODE || err.code == CONTENT_ROOT_NOT_FOUND_CODE =>
                Ok(false),
            Err(other_error) => Err(other_error.into())
        }
    }

    fn uploading_node_expression(name:&str) -> String {
        format!("File_Uploading.file_uploading Enso_Project.data/\"{}\"",name)
    }

    fn uploaded_node_expression(name:&str) -> String {
        format!("File.read Enso_Project.data/\"{}\"",name)
    }

    fn data_path(&self) -> Path {
        Path::new(self.project.content_root_id(),&[DATA_DIR_NAME])
    }
}



// ============
// === Test ===
// ============

#[cfg(test)]
mod test {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::test::mock;

    use enso_protocol::language_server::{response, FileAttributes};
    use futures::SinkExt;
    use mockall::Sequence;
    use utils::test::traits::*;
    use enso_protocol::types::UTCDateTime;


    // === Test Providers ===

    type TestProvider          = Box<dyn Iterator<Item=Vec<u8>>>;
    type TestAsyncProvider     = futures::channel::mpsc::Receiver<FallibleResult<Vec<u8>>>;
    type TestAsyncProviderSink = futures::channel::mpsc::Sender<FallibleResult<Vec<u8>>>;

    impl DataProvider for TestProvider {
        fn next_chunk(&mut self) -> BoxFuture<FallibleResult<Option<Vec<u8>>>> {
            futures::future::ready(Ok(self.next())).boxed_local()
        }
    }

    impl DataProvider for TestAsyncProvider {
        fn next_chunk(&mut self) -> BoxFuture<FallibleResult<Option<Vec<u8>>>> {
            self.next().map(|chunk| match chunk {
                Some(Ok(chunk)) => Ok(Some(chunk)),
                Some(Err(err))  => Err(err),
                None            => Ok(None),
            }).boxed_local()
        }
    }


    // === Data ===

    const TEST_FILE:&str = "file";

    struct TestData {
        chunks    : Vec<Vec<u8>>,
        file_size : usize,
        checksum  : Sha3_224,
        path      : Path,
    }

    impl TestData {
        fn new(chunks:Vec<Vec<u8>>) -> Self {
            let entire_file = chunks.iter().flatten().copied().collect_vec();
            let file_size   = entire_file.len();
            let checksum    = Sha3_224::new(&entire_file);
            let path        = Path::new(mock::data::ROOT_ID, &[DATA_DIR_NAME,TEST_FILE]);
            Self{chunks,file_size,checksum,path}
        }

        fn setup_uploading_expectations
        (&self, json_client:&language_server::MockClient, binary_client:&mut binary::MockClient) {
            let mut write_seq = Sequence::new();
            for chunk in self.chunks.iter().cloned() {
                let path = self.path.clone();
                binary_client.expect_write_file()
                    .withf(move |p,ch| *p == path && ch == chunk)
                    .times(1)
                    .in_sequence(&mut write_seq)
                    .returning(|_,_| futures::future::ready(Ok(())).boxed_local());
            };
            let checksum = self.checksum.clone();
            json_client.expect.file_checksum(enclose!((self.path => path) move |p| {
                assert_eq!(*p,path);
                Ok(response::FileChecksum{checksum})
            }));
        }

        fn file_to_upload(&self) -> FileToUpload<TestProvider> {
            FileToUpload {
                name: TEST_FILE.to_owned(),
                size: self.file_size,
                data: Box::new(self.chunks.clone().into_iter())
            }
        }

        fn file_to_upload_async(&self) -> (FileToUpload<TestAsyncProvider>,TestAsyncProviderSink) {
            let (sender,receiver) = futures::channel::mpsc::channel(5);
            let file_to_upload    = FileToUpload {
                name: TEST_FILE.to_owned(),
                size: self.file_size,
                data: receiver
            };
            (file_to_upload,sender)
        }
    }


    // === FileUploadProcess Tests ===

    struct UploadingFixture {
        test          : TestWithLocalPoolExecutor,
        chunks        : <Vec<Vec<u8>> as IntoIterator>::IntoIter,
        process       : FileUploadProcess<TestAsyncProvider>,
        provider_sink : Option<TestAsyncProviderSink>,
    }

    impl UploadingFixture {
        fn new(logger:impl AnyLogger, data:TestData) -> Self {
            let mut binary_cli = binary::MockClient::new();
            let json_cli       = language_server::MockClient::default();
            data.setup_uploading_expectations(&json_cli,&mut binary_cli);
            let (file,provider_sink) = data.file_to_upload_async();
            let bin_connection       = Rc::new(binary::Connection::new_mock(binary_cli));
            let json_connection      = Rc::new(language_server::Connection::new_mock(json_cli));

            Self {
                test          : TestWithLocalPoolExecutor::set_up(),
                chunks        : data.chunks.into_iter(),
                process       : FileUploadProcess::new(logger,file,bin_connection,json_connection,data.path),
                provider_sink : Some(provider_sink),
            }
        }

        fn next_chunk_result(&mut self) -> FallibleResult<bool> {
            let mut future = self.process.upload_chunk().boxed_local();

            if let Some(mut sink) = std::mem::take(&mut self.provider_sink) {
                // If the stream is still open, we shall wait for a new value
                self.test.run_until_stalled();
                future.expect_pending();

                if let Some(chunk) = self.chunks.next() {
                    sink.send(Ok(chunk)).boxed_local().expect_ok();
                    self.provider_sink = Some(sink);
                }
            }

            self.test.run_until_stalled();
            future.expect_ready()
        }
    }

    #[test]
    fn uploading_file() {
        let logger   = Logger::new("test::uploading_file");
        let data     = TestData::new(vec![vec![1,2,3,4,5],vec![3,4,5,6,7,8]]);
        let mut test = UploadingFixture::new(logger,data);

        assert!(!test.next_chunk_result().unwrap());
        assert!(!test.next_chunk_result().unwrap());
        assert!(test.next_chunk_result().unwrap());
    }

    #[test]
    fn checksum_mismatch_should_cause_an_error() {
        let logger    = Logger::new("test::uploading_file");
        let mut data  = TestData::new(vec![vec![1,2,3,4,5]]);
        data.checksum = Sha3_224::new(&[3,4,5,6,7,8]);
        let mut test  = UploadingFixture::new(logger,data);

        assert!(!test.next_chunk_result().unwrap());
        assert!(test.next_chunk_result().is_err());
    }


    // === NodeFromDroppedFileHandler Tests ===

    #[test]
    fn creating_node_from_dropped_file() {
        let logger = Logger::new("test::creating_node_from_dropped_file");
        let data   = TestData::new(vec![vec![1,2,3,4],vec![5,6,7,8]]);
        let mut fixture = mock::Unified::new().fixture_customize(|_,json_rpc,binary_rpc| {
            json_rpc.expect.file_info(|path| {
                assert_eq!(*path, Path::new(mock::data::ROOT_ID,&[DATA_DIR_NAME]));
                let dummy_time = UTCDateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap();
                Ok(response::FileInfo {
                    attributes : FileAttributes {
                        creation_time: dummy_time.clone(),
                        last_access_time: dummy_time.clone(),
                        last_modified_time: dummy_time,
                        kind: FileSystemObject::Directory {
                            name : DATA_DIR_NAME.to_owned(),
                            path : Path::new_root(mock::data::ROOT_ID)
                        },
                        byte_size: 0
                    } 
                })
            });
            json_rpc.expect.file_list(|path| {
                assert_eq!(*path, Path::new(mock::data::ROOT_ID,&[DATA_DIR_NAME]));
                Ok(response::FileList { paths: vec![] })
            });
            data.setup_uploading_expectations(json_rpc,binary_rpc);
        });

        let handler  = NodeFromDroppedFileHandler::new(logger,fixture.project,fixture.graph);
        let position = model::module::Position::new(45.0,70.0);
        let file     = data.file_to_upload();
        handler.create_node_and_start_uploading(file,position).unwrap();
        assert_eq!(fixture.module.ast().repr(), format!("{}\n    operator1 = File_Uploading.file_uploading Enso_Project.data/\"file\"", mock::data::CODE));
        fixture.executor.run_until_stalled();
        assert_eq!(fixture.module.ast().repr(), format!("{}\n    operator1 = File.read Enso_Project.data/\"file\"", mock::data::CODE));

    }
}
