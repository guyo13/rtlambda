// Copyright 2022-2023 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::api::LambdaContext;

use serde::Serialize;
use std::fmt::Display;

/// Defines an event handler plus any initilization logic required for implementing the lambda.
/// This is the main interface users of the library should implement.
pub trait EventHandler: Sized {
    /// Defines the lambda's output type which must implement or derive [`serde::Serialize`] in order to be sent as a JSON to the RuntimeAPI.
    type EventOutput: Serialize;
    /// Defines the lambda's error type which must implement or derive [`Display`].
    type EventError: Display;
    /// Defines the lambda's initialization error type which must implement or derive [`Display`].
    type InitError: Display;
    /// Constructs the event handler object and sets up any resources that are reusable across the lifetime of the lambda instance.
    /// Returns a [`Result`] with the event handler object or an error object if failed.
    fn initialize() -> Result<Self, Self::InitError>;
    /// Processes each incoming lambda event and returns a [`Result`] with the lambda's output.
    /// # Arguments
    ///
    /// * `event` - The JSON event as a string slice, should be deserialized by the implementation.
    /// * `context` - A shared reference to the current event context.
    /// `Ctx` Defines the context object type, typically a [`crate::data::context::EventContext`].
    fn on_event<Ctx: LambdaContext>(
        &mut self,
        event: &str,
        context: &Ctx,
    ) -> Result<Self::EventOutput, Self::EventError>;
}
