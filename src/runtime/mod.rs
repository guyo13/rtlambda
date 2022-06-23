// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

/// Defines the interface an event handler should implement.
pub mod event_handler;

use crate::data::context::{EventContext, LambdaContext};
use crate::data::env::RuntimeEnvVars;
use crate::data::response::{LambdaAPIResponse, AWS_FUNC_ERR_TYPE};
use crate::error::{Error, CONTAINER_ERR};
use crate::runtime::event_handler::EventHandler;
use crate::transport::Transport;

use std::env::set_var;
use std::ffi::OsStr;
use std::fmt::Display;
use std::marker::PhantomData;

use serde::Serialize;

// Already handles any panic inducing errors
macro_rules! handle_response {
    ($resp:expr) => {
        let status_code = $resp.get_status_code();
        match status_code {
            400..=499 => {
                let err = $resp.error_response().or(Some("")).unwrap();
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

/// A generic trait defining an interface for a Lambda runtime.
/// The HTTP Backend in use is defined by the input types `T` that implements [`Transport`] and `R` implementing [`LambdaAPIResponse`].
/// The `OUT` type parameter is the user-defined response type which represents the success result of the event handler.
///
/// The combination of type parameters enables the compiled program to avoid dynamic dispatch when calling the runtime methods.
pub trait LambdaRuntime<R, T, OUT>
where
    R: LambdaAPIResponse,
    T: Transport<R>,
    OUT: Serialize,
{
    /// Used to fetch the next event from the Lambda service.
    fn next_invocation(&mut self) -> Result<R, Error>;
    /// Sends back a JSON formatted response to the Lambda service, after processing an event.
    fn invocation_response(&self, request_id: &str, response: &OUT) -> Result<R, Error>;
    /// Used to report an error during initialization to the Lambda service.
    fn initialization_error(
        &self,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<R, Error>;
    /// Used to report an error during function invocation to the Lambda service.
    fn invocation_error(
        &self,
        request_id: &str,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<R, Error>;
    /// Implements the runtime loop logic.
    fn run(&mut self);
}

/// The default generic implementation of the [`LambdaRuntime`] interface.
/// Works by accepting a pointer to an initialization function or a closure `initializer` -
/// that is run once and initializes "global" variables that are created once
/// and persist across the runtime's life (DB connections, heap allocated static data etc...).
///
/// The initialization function returns a user-defined closure object that acts as the event handler and can
/// take ownership over those variables by move.
/// The Ok output type of the closure - `OUT` - should implement [`serde::Serialize`].
///
/// The `R`, `T` and `OUT` type parameters correspond to the ones defined in [`LambdaRuntime`].
///
/// The `ENV` type parameter defines the implementation of [`crate::data::env::RuntimeEnvVars`] for reading the env-vars set for the runtime.
///
/// The `ERR` type parameter is a user-defined type representing any error that may occur during initialization or invocation of the event handler.
pub struct DefaultRuntime<R, T, H, ENV, OUT, ERR>
where
    R: LambdaAPIResponse,
    T: Transport<R>,
    H: EventHandler<OUT, ERR, EventContext<ENV, R>>,
    ENV: RuntimeEnvVars,
    ERR: Display,
    OUT: Serialize,
{
    /// An owned container that holds a copy of the env vars and the current invocation data.
    context: EventContext<ENV, R>,
    /// The Lambda API version string.
    version: String,
    /// URI of the Lambda API.
    api_base: String,
    /// An owned instance of the HTTP Backend implementing [`crate::transport::Transport`].
    transport: T,
    /// The event handler instance.
    handler: H,
    output_type: PhantomData<OUT>,
    err_type: PhantomData<ERR>,
}

impl<R, T, H, ENV, OUT, ERR> DefaultRuntime<R, T, H, ENV, OUT, ERR>
where
    R: LambdaAPIResponse,
    T: Transport<R>,
    H: EventHandler<OUT, ERR, EventContext<ENV, R>>,
    ENV: RuntimeEnvVars,
    ERR: Display,
    OUT: Serialize,
{
    pub fn new(version: &str, handler: H) -> Self {
        // Initialize default env vars and check for the host and port of the runtime API.
        let env_vars = ENV::default();
        let api_base = match env_vars.get_runtime_api() {
            Some(v) => v.to_string(),
            None => panic!("Failed getting API base URL from env vars"),
        };

        // Format the version string, later used in API calls
        let formatted_version: String = format_version_string!(version);

        // Start the transport layer object
        let transport = T::default();

        // Initialize the context object
        let context = EventContext::<ENV, R> {
            env_vars,
            invo_resp: None,
        };

        Self {
            context,
            version: formatted_version,
            api_base,
            transport,
            handler,
            output_type: PhantomData,
            err_type: PhantomData,
        }
    }
}

impl<R, T, H, ENV, OUT, ERR> LambdaRuntime<R, T, OUT> for DefaultRuntime<R, T, H, ENV, OUT, ERR>
where
    R: LambdaAPIResponse,
    T: Transport<R>,
    H: EventHandler<OUT, ERR, EventContext<ENV, R>>,
    ENV: RuntimeEnvVars,
    ERR: Display,
    OUT: Serialize,
{
    fn run(&mut self) {
        // Run the app's initializer and check for errors
        let init_result = self.handler.initialize();
        if let Err(init_err) = init_result {
            // Try reporting to the Lambda service if there is an error during initialization
            // TODO: Take error type and request from ERR
            match self.initialization_error(Some("Runtime.InitError"), None) {
                Ok(r) => r,
                // If an error occurs during reporting the previous error, panic.
                Err(err) => panic!(
                    "Failed to report initialization error. Error: {}, AWS Error: {}",
                    &init_err, err
                ),
            };
            // After reporting an init error just panic.
            panic!("Initialization Error: {}", &init_err);
        }

        // Start event processing loop as specified in [https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html]
        loop {
            // Get the next event in the queue.
            // Failing to get the next event will either panic (on server error) or continue with an error (on client-error codes).
            self.context.invo_resp = match self.next_invocation() {
                // TODO - perhaps log the error
                Err(_e) => continue,
                // TODO - check if RVO optimization kicks in?
                Ok(resp) => Some(resp),
            };
            // Vaidate that request id is present in the response.
            let request_id = match self.context.aws_request_id() {
                Some(rid) => rid,
                None => {
                    // TODO - figure out what we'd like to do with the result returned from success/client-err api responses
                    let _ = self.initialization_error(Some("Runtime.MissingRequestId"), None);
                    continue;
                }
            };

            // Retrieve the event JSON
            // TODO - deserialize? Currently user code should deserialize inside their handler
            // Both the invocation response and event response are safe to unwrap at this point.
            let event = self
                .context
                .invo_resp
                .as_ref()
                .unwrap()
                .event_response()
                .unwrap();

            // Execute the event handler
            let lambda_output = self.handler.on_event(event, &self.context);

            // TODO - figure out what we'd like to do with the result returned from success/client-err api responses (e.g: log, run a user defined callback...)
            let _ = match lambda_output {
                Ok(out) => self.invocation_response(request_id, &out),
                // TODO - pass an ErrorRequest json
                Err(err) => {
                    let _err = format!("{}", &err);
                    self.invocation_error(request_id, Some(&_err), Some(&_err))
                }
            };
        }
    }

    fn next_invocation(&mut self) -> Result<R, Error> {
        let url = format!(
            "http://{}/{}/runtime/invocation/next",
            self.api_base, self.version
        );
        let resp = self.transport.get(&url, None, None)?;

        handle_response!(resp);

        // If AWS returns the "Lambda-Runtime-Trace-Id" header, set its value to the -
        // "_X_AMZN_TRACE_ID" env var
        if let Some(req_id) = resp.trace_id() {
            set_var(OsStr::new("_X_AMZN_TRACE_ID"), OsStr::new(req_id));
            self.context.env_vars.set_trace_id(Some(req_id));
        };

        Ok(resp)
    }

    fn invocation_response(&self, request_id: &str, response: &OUT) -> Result<R, Error> {
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
    ) -> Result<R, Error> {
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
    ) -> Result<R, Error> {
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
