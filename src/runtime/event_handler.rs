// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::data::context::LambdaContext;

use serde::Serialize;
use std::fmt::Display;

/// Defines an event handler plus any initilization logic required for implementing the lambda.
/// This is the main interface users of the library should implement, it accepts 3 type parameters:
/// `O` - Defines the lambda's output type which must implement or derive [`serde::Serialize`] in order to be sent as a JSON to the RuntimeAPI.
/// `E` - Defines the lambda's error output type which must implement or derive [`Display`].
/// `I` - Defines the concrete implementation of the context object used at runtime. Typically [`crate::data::context::EventContext`].
pub trait EventHandler<O: Serialize, E: Display, I: LambdaContext> {
    /// Sets up any resources that are reusable across the lifetime of the lambda instance.
    /// Returns a [`Result`] that indicates whether initialization succeeded and if not contains an error object.
    fn initialize(&mut self) -> Result<(), E>;
    /// Processes each incoming lambda event and returns a [`Result`] with the lambda's output.
    /// # Arguments
    ///
    /// * `event` - The JSON event as a string slice, should be deserialized by the implementation.
    /// * `context` - A shared reference to the current event context.
    fn on_event(&mut self, event: &str, context: &I) -> Result<O, E>;
}
