use super::*;

use crate::transport::test_utils::TestWithMockedTransport;

use json_rpc::test_util::transport::mock::MockTransport;
use wasm_bindgen_test::wasm_bindgen_test_configure;
use wasm_bindgen_test::wasm_bindgen_test;
use serde_json::json;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn failure_to_open_project_is_reported() {
    let transport   = MockTransport::new();
    let mut fixture = TestWithMockedTransport::set_up(&transport);
    fixture.run_test(async move {
        let client  = setup_project_manager(transport);
        let project = open_most_recent_project_or_create_new(&default(),&client).await;
        project.expect_err("error should have been reported");
    });
    fixture.when_stalled_send_response(json!({
            "projects": [{
                "name"       : "Project",
                "id"         : "4b871393-eef2-4970-8765-4f3c1ea83d09",
                "lastOpened" : "2020-05-08T11:04:07.28738Z"
            }]
        }));
    fixture.when_stalled_send_error(1,"Service error");
}
