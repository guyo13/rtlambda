// Copyright 2022-2023 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::api::LambdaContext;

use serde::Serialize;
use std::fmt::Display;

/// Defines an event handler plus any initilization logic required for implementing the lambda.
/// This is the main interface users of the library should implement.
pub trait EventHandler {
    /// Defines the lambda's output type which must implement or derive [`serde::Serialize`] in order to be sent as a JSON to the RuntimeAPI.
    type Output: Serialize;
    /// Defines the lambda's error output type which must implement or derive [`Display`].
    type Error: Display;
    /// Sets up any resources that are reusable across the lifetime of the lambda instance.
    /// Returns a [`Result`] that indicates whether initialization succeeded and if not contains an error object.
    fn initialize(&mut self) -> Result<(), Self::Error>;
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
    ) -> Result<Self::Output, Self::Error>;
}
