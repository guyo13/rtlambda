// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::data::context::LambdaContext;

use serde::Serialize;
use std::fmt::Display;

pub trait EventHandler<O: Serialize, E: Display, I: LambdaContext> {
    /// The initialization method runs once in order to up any resources that are reusable across the lifetime of the lambda.
    /// It may fail during runtime (e.g if a db connection was not succesfully opened, etc..) and in that case
    /// the method returns an Err variant of the same [`E`] type defined for the event handler.
    fn initialize(&mut self) -> Result<(), E>;
    /// Processes each incoming lambda event.
    /// The [`event`] argument contains the JSON event data as a string slice, which should be deserialized by the implementation.
    fn on_event(&mut self, event: &str, context: &I) -> Result<O, E>;
}
