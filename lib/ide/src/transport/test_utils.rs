use crate::prelude::*;

use crate::executor::test_utils::TestWithLocalPoolExecutor;

use json_rpc::test_util::transport::mock::MockTransport;



#[derive(Debug)]
pub struct TestWithMockedTransport {
    with_executor_fixture : TestWithLocalPoolExecutor,
    transport             : MockTransport,
    next_response_id      : usize,
}

impl TestWithMockedTransport {
    pub fn set_up(transport:&MockTransport) -> Self {
        Self {
            with_executor_fixture : TestWithLocalPoolExecutor::set_up(),
            transport             : transport.clone_ref(),
            next_response_id      : 0,
        }
    }

    pub fn run_test<TestBody>(&mut self, test:TestBody) -> &mut Self
    where TestBody : Future<Output=()> + 'static {
        self.with_executor_fixture.run_test(test);
        self
    }

    pub fn when_stalled_send_response(&mut self, result:Option<&str>) -> &mut Self {
        let mut transport = self.transport.clone_ref();
        let id            = self.next_response_id;
        let response_val  = result.map(|s| format!("\"{}\"", s)).unwrap_or("null".to_string());
        self.with_executor_fixture.when_stalled(move ||
            transport.mock_peer_message_text(format!(r#"{{
                "jsonrpc" : "2.0",
                "id"      : {},
                "result"  : {}
            }}"#,id,response_val))
        );
        self.next_response_id += 1;
        self
    }
}
