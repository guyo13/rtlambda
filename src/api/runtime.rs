// Copyright 2022-2023 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::api::{EventHandler, Transport};
use crate::error::*;

/// Defines the interface for the Lambda runtime.
pub trait LambdaRuntime {
    /// Defines the type of the event handler executed by the runtime in each invocation.
    type Handler: EventHandler;
    /// Defines the Transport type. See `[crate::transport::Transport]`.
    type Transport: crate::api::Transport;
    /// Used to fetch the next event from the Lambda service.
    fn next_invocation(&mut self) -> Result<<Self::Transport as Transport>::Response, Error>;
    /// Sends back a JSON formatted response to the Lambda service, after processing an event.
    fn invocation_response(
        &self,
        request_id: &str,
        response: &<Self::Handler as EventHandler>::EventOutput,
    ) -> Result<<Self::Transport as Transport>::Response, Error>;
    /// Used to report an error during initialization to the Lambda service.
    fn initialization_error(
        &self,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<<Self::Transport as Transport>::Response, Error>;
    /// Used to report an error during function invocation to the Lambda service.
    fn invocation_error(
        &self,
        request_id: &str,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<<Self::Transport as Transport>::Response, Error>;
    /// Implements the runtime loop logic.
    fn run(&mut self);
}
