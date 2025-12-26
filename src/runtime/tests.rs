#[cfg(test)]
mod tests {
    use crate::api::{EventHandler, LambdaContext, LambdaAPIResponse, LambdaRuntime, Transport};
    use crate::error::Error;
    use crate::runtime::DefaultRuntime;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use serde::Serialize;
    use serial_test::serial;

    // --- Mock Transport ---

    #[derive(Clone)]
    struct MockResponse {
        body: String,
        status: u16,
        headers: Vec<(String, String)>,
    }

    impl LambdaAPIResponse for MockResponse {
        fn get_body(self) -> Result<String, Error> {
            Ok(self.body)
        }
        fn get_status_code(&self) -> u16 {
            self.status
        }
        fn get_aws_request_id(&self) -> Option<&str> {
            self.headers.iter().find(|(k, _)| k == "Lambda-Runtime-Aws-Request-Id").map(|(_, v)| v.as_str())
        }
        fn get_deadline(&self) -> Option<u64> {
            None
        }
        fn get_invoked_function_arn(&self) -> Option<&str> {
            None
        }
        fn get_x_ray_tracing_id(&self) -> Option<&str> {
            None
        }
        fn get_client_context(&self) -> Option<&str> {
            None
        }
        fn get_cognito_identity(&self) -> Option<&str> {
            None
        }
    }

    #[derive(Default)]
    struct MockTransport;

    thread_local! {
        static MOCK_QUEUE: RefCell<VecDeque<MockResponse>> = RefCell::new(VecDeque::new());
        static REQUEST_LOG: RefCell<Vec<(String, String, Option<String>)>> = RefCell::new(Vec::new()); // (method, url, body)
    }

    impl MockTransport {
        fn push_response(body: &str, status: u16, request_id: Option<&str>) {
            let mut headers = Vec::new();
            if let Some(rid) = request_id {
                headers.push(("Lambda-Runtime-Aws-Request-Id".to_string(), rid.to_string()));
            }
            MOCK_QUEUE.with(|q| {
                q.borrow_mut().push_back(MockResponse {
                    body: body.to_string(),
                    status,
                    headers,
                })
            });
        }

        fn pop_request() -> Option<(String, String, Option<String>)> {
            REQUEST_LOG.with(|l| l.borrow_mut().pop())
        }

        fn clear() {
             MOCK_QUEUE.with(|q| q.borrow_mut().clear());
             REQUEST_LOG.with(|l| l.borrow_mut().clear());
        }
    }

    impl Transport for MockTransport {
        type Response = MockResponse;

        fn get(
            &self,
            url: &str,
            body: Option<&str>,
            _headers: Option<&[(&str, &str)]>,
        ) -> Result<Self::Response, Error> {
             REQUEST_LOG.with(|l| l.borrow_mut().push(("GET".to_string(), url.to_string(), body.map(|s| s.to_string()))));
             MOCK_QUEUE.with(|q| {
                 q.borrow_mut().pop_front().ok_or(Error::new("No mock response".to_string()))
             })
        }

        fn post(
            &self,
            url: &str,
            body: Option<&str>,
            headers: Option<&[(&str, &str)]>,
        ) -> Result<Self::Response, Error> {
             let mut logged_headers = Vec::new();
             if let Some(h) = headers {
                 for (k, v) in h.iter() {
                     logged_headers.push((k.to_string(), v.to_string()));
                 }
             }
             // For now we just log the body/url, but if we wanted to check headers we could add another field to REQUEST_LOG
             // Or append to body for verification in tests.
             // Let's modify REQUEST_LOG to include headers?
             // Or simpler: just log if we find the error header.
             let mut body_str = body.map(|s| s.to_string()).unwrap_or_default();
             if let Some(h) = headers {
                for (k, v) in h.iter() {
                    if *k == "Lambda-Runtime-Function-Error-Type" {
                        body_str.push_str(&format!("|Header:{}:{}", k, v));
                    }
                }
             }

             REQUEST_LOG.with(|l| l.borrow_mut().push(("POST".to_string(), url.to_string(), Some(body_str))));
             MOCK_QUEUE.with(|q| {
                 q.borrow_mut().pop_front().ok_or(Error::new("No mock response".to_string()))
             })
        }
    }

    // --- Mock Handler ---

    struct TestEventHandler;

    #[derive(Serialize)]
    struct TestOutput {
        msg: String,
    }

    impl EventHandler for TestEventHandler {
        type EventOutput = TestOutput;
        type EventError = String;
        type InitError = String;

        fn initialize() -> Result<Self, Self::InitError> {
            Ok(TestEventHandler)
        }

        fn on_event<Ctx: LambdaContext>(
            &mut self,
            event: String,
            _context: &Ctx,
        ) -> Result<Self::EventOutput, Self::EventError> {
            if event == "\"fail\"" {
                Err("Failed".to_string())
            } else {
                Ok(TestOutput { msg: format!("Echo: {}", event) })
            }
        }
    }

    // --- Tests ---

    #[test]
    #[serial]
    fn test_runtime_next_invocation() {
        MockTransport::clear();
        std::env::set_var("AWS_LAMBDA_RUNTIME_API", "localhost:8080");

        MockTransport::push_response("\"hello\"", 200, Some("req-1"));

        let mut runtime = DefaultRuntime::<MockTransport, TestEventHandler>::new("2018-06-01");

        // This fails if MockTransport doesn't work or logic is wrong
        let resp = runtime.next_invocation().expect("Failed to get next invocation");

        assert_eq!(resp.get_aws_request_id(), Some("req-1"));
        assert_eq!(resp.get_body().unwrap(), "\"hello\"");
    }

    #[test]
    #[serial]
    fn test_runtime_invocation_response() {
        MockTransport::clear();
        std::env::set_var("AWS_LAMBDA_RUNTIME_API", "localhost:8080");

        MockTransport::push_response("", 202, None);

        let runtime = DefaultRuntime::<MockTransport, TestEventHandler>::new("2018-06-01");

        let output = TestOutput { msg: "success".to_string() };
        let resp = runtime.invocation_response("req-1", &output).expect("Failed to send response");

        assert_eq!(resp.get_status_code(), 202);

        let req = MockTransport::pop_request().unwrap();
        assert_eq!(req.0, "POST");
        assert!(req.1.contains("/invocation/req-1/response"));
        assert_eq!(req.2.unwrap(), "{\"msg\":\"success\"}");
    }

    #[test]
    #[serial]
    fn test_runtime_invocation_error() {
        MockTransport::clear();
        std::env::set_var("AWS_LAMBDA_RUNTIME_API", "localhost:8080");

        MockTransport::push_response("", 202, None);

        let runtime = DefaultRuntime::<MockTransport, TestEventHandler>::new("2018-06-01");

        let resp = runtime.invocation_error("req-1", Some("ErrorType"), Some("ErrorMsg")).expect("Failed to report error");

        assert_eq!(resp.get_status_code(), 202);

        let req = MockTransport::pop_request().unwrap();
        assert_eq!(req.0, "POST");
        assert!(req.1.contains("/invocation/req-1/error"));
        // Check for the error header which we appended to the body string in MockTransport::post
        assert!(req.2.unwrap().contains("|Header:Lambda-Runtime-Function-Error-Type:ErrorType"));
    }
}
