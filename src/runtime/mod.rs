// Copyright 2022-2023 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use std::time::Duration;

use crate::api::{
    EventHandler, LambdaAPIResponse, LambdaContext, LambdaContextSetter, LambdaEnvSetter,
    LambdaEnvVars, LambdaRuntime, Transport, AWS_FUNC_ERR_TYPE,
};
use crate::data::context::EventContext;
use crate::error::{Error, CONTAINER_ERR};

// Already handles any panic inducing errors
macro_rules! handle_response {
    ($resp:expr) => {
        let status_code = $resp.get_status_code();
        match status_code {
            400..=499 => {
                let err = $resp
                    .get_body()
                    .unwrap_or_else(|_| String::with_capacity(0));
                return Err(Error::new(format!(
                    "Client error ({}). ErrorResponse: {}",
                    status_code, err
                )));
            }
            500 => panic!("{}", CONTAINER_ERR),
            _ => (),
        };
    };
}

macro_rules! format_version_string {
    ($version:expr) => {
        if let Some(v) = $version.strip_prefix("/") {
            v.to_string()
        } else {
            $version.to_string()
        }
    };
}

/// The default generic implementation of the [`LambdaRuntime`] interface.
/// Works by accepting an owned [`EventHandler`] object which is first initialized by the runtime by calling [`EventHandler::initialize`].
pub struct DefaultRuntime<T: Transport, H: EventHandler> {
    /// An owned container that holds a copy of the env vars and the current invocation data.
    context: EventContext,
    /// The Lambda API version string.
    version: String,
    /// URI of the Lambda API.
    api_base: String,
    /// An owned instance of the HTTP Backend implementing [`crate::transport::Transport`].
    transport: T,
    /// The event handler instance.
    handler: H,
}

impl<T: Transport, H: EventHandler> DefaultRuntime<T, H> {
    pub fn new(version: &str, handler: H) -> Self {
        // Initialize the context object
        let context = EventContext::default();
        // Check for the host and port of the runtime API.
        let api_base = match context.get_lambda_runtime_api() {
            Some(v) => v.to_string(),
            None => panic!("Failed getting API base URL from env vars"),
        };

        // Format the version string, later used in API calls
        let formatted_version: String = format_version_string!(version);

        // Start the transport layer object
        let transport = T::default();

        Self {
            context,
            version: formatted_version,
            api_base,
            transport,
            handler,
        }
    }
}

impl<T, H> LambdaRuntime for DefaultRuntime<T, H>
where
    T: Transport,
    H: EventHandler,
{
    type Handler = H;
    type Transport = T;

    fn run(&mut self) {
        // Run the app's initializer and check for errors
        let init_result = self.handler.initialize();
        if let Err(init_err) = init_result {
            // Report any initialization error to the Lambda service
            // TODO: Serialize the init_err and the error type into JSON as specified in
            // https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html#runtimes-api-initerror
            // If an error occurs during reporting the init error, panic.
            if let Err(err) = self.initialization_error(Some("Runtime.InitError"), None) {
                panic!(
                    "Failed to report initialization error. Error: {}, AWS Error: {}",
                    &init_err, err
                );
            };

            // After reporting an init error just panic.
            panic!("Initialization Error: {}", &init_err);
        }

        // Start event processing loop as specified in [https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html]
        loop {
            // Get the next event in the queue and update the context if successful.
            // Failing to get the next event will either panic (on server error) or continue with an error (on client-error codes).
            let next_invo = match self.next_invocation() {
                // TODO - perhaps log the error
                Err(_e) => continue,
                Ok(resp) => resp,
            };

            // Retrieve the event JSON
            // The response body is safe to unwrap at this point.
            let event = next_invo.get_body().unwrap();

            // Execute the event handler
            // TODO - pass the event an an owned String
            let lambda_output = self.handler.on_event(&event, &self.context);
            let request_id = self.context.get_aws_request_id().unwrap();

            // TODO - figure out what we'd like to do with the result returned from success/client-err api responses (e.g: log, run a user defined callback...)
            let _ = match lambda_output {
                Ok(out) => self.invocation_response(request_id, &out),
                // TODO - pass an ErrorRequest json - https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html#runtimes-api-invokeerror
                Err(err) => {
                    let _err = format!("{}", &err);
                    self.invocation_error(request_id, Some(&_err), Some(&_err))
                }
            };
        }
    }

    fn next_invocation(&mut self) -> Result<<Self::Transport as Transport>::Response, Error> {
        // TODO - cache this string
        let url = format!(
            "http://{}/{}/runtime/invocation/next",
            self.api_base, self.version
        );
        let resp = self.transport.get(&url, None, None)?;
        handle_response!(resp);

        // Update the request context
        self.context.set_aws_request_id(resp.get_aws_request_id());
        self.context.set_client_context(resp.get_client_context());
        self.context
            .set_cognito_identity(resp.get_cognito_identity());
        self.context
            .set_deadline(resp.get_deadline().map(Duration::from_millis));
        self.context
            .set_invoked_function_arn(resp.get_invoked_function_arn());
        self.context
            .set_x_ray_tracing_id(resp.get_x_ray_tracing_id());

        // Vaidate that request id is present in the response. If not report to Lambda.
        if self.context.get_aws_request_id().is_none() {
            // TODO - figure out what we'd like to do with the result returned from success/client-err api responses
            let _ = self.initialization_error(Some("Runtime.MissingRequestId"), None);
            // TODO - return None - requires modifying the function signature
            return Err(Error::empty());
        }
        Ok(resp)
    }

    fn invocation_response(
        &self,
        request_id: &str,
        response: &<Self::Handler as EventHandler>::Output,
    ) -> Result<<Self::Transport as Transport>::Response, Error> {
        let url = format!(
            "http://{}/{}/runtime/invocation/{}/response",
            self.api_base, self.version, request_id
        );
        // TODO - Utilize a user-defined JSON serializer?
        let serialized = match serde_json::to_string(response) {
            Ok(ser) => ser,
            Err(err) => {
                return Err(Error::new(format!(
                    "Failed serializing output to JSON. {}",
                    err
                )))
            }
        };
        let resp = self.transport.post(&url, Some(&serialized), None)?;
        handle_response!(resp);

        Ok(resp)
    }

    fn initialization_error(
        &self,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<<Self::Transport as Transport>::Response, Error> {
        let url = format!(
            "http://{}/{}/runtime/init/error",
            self.api_base, self.version
        );
        let headers = error_type.map(|et| (vec![AWS_FUNC_ERR_TYPE], vec![et]));
        let resp = self.transport.post(&url, error_req, headers)?;
        handle_response!(resp);

        Ok(resp)
    }

    fn invocation_error(
        &self,
        request_id: &str,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<<Self::Transport as Transport>::Response, Error> {
        let url = format!(
            "http://{}/{}/runtime/invocation/{}/error",
            self.api_base, self.version, request_id
        );
        let headers = error_type.map(|et| (vec![AWS_FUNC_ERR_TYPE], vec![et]));
        let resp = self.transport.post(&url, error_req, headers)?;
        handle_response!(resp);

        Ok(resp)
    }
}
