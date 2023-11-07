// Copyright 2022-2023 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

/// Defines the public API of `rtlambda`.
pub mod api;
/// Implementations of the `rtlambda` API for different HTTP backends.
pub mod backends;
/// Defines the library's core data structures.
pub mod data;
/// Defines error types and constants.
pub mod error;
/// Defines the [`crate::runtime::DefaultRuntime`] which implements the Rust lambda runtime.
pub mod runtime;

/// The current Lambda API version used on AWS.
pub static LAMBDA_VER: &str = "2018-06-01";

/// A prelude that contains all the relevant imports when using the library's default runtime implementation,
/// which currently ships with a [ureq](https://crates.io/crates/ureq) based HTTP Backend and [serde_json](https://crates.io/crates/serde_json) for serialization.
pub mod prelude {
    pub use crate::api::*;
    pub use crate::backends::ureq::*;
    pub use crate::data::context::EventContext;
    pub use crate::runtime::DefaultRuntime;
    pub use crate::LAMBDA_VER;
}

/// Creates a [`crate::runtime::DefaultRuntime`] with the given transport, handler, env, out, err types as well as version and initializer.
#[macro_export]
macro_rules! create_runtime {
    ($transport:ty, $ver:expr, $ev_handler:ty) => {
        DefaultRuntime::<$transport, $ev_handler>::new($ver);
    };
}

/// Creates a [`crate::runtime::DefaultRuntime`] with ureq based HTTP backend and the default implementation of env-vars handling.
#[macro_export]
macro_rules! default_runtime {
    ($ev_handler:ty) => {
        create_runtime!(UreqTransport, LAMBDA_VER, $ev_handler)
    };
}
