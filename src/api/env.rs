// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

/// An enum representing the `InitializationType` choices set as an env-var on the instance by AWS Lambda.
/// See [Defined runtime environment variables](https://docs.aws.amazon.com/lambda/latest/dg/configuration-envvars.html#configuration-envvars-runtime).
#[derive(Clone, Copy, Debug)]
pub enum InitializationType {
    OnDemand,
    ProvisionedConcurrency,
    Unknown,
}

impl InitializationType {
    /// Returns the [`InitializationType`] value corresponding to the input string.
    /// itype must be lowercase.
    pub fn from_string(itype: &str) -> InitializationType {
        match itype {
            "on-demand" => Self::OnDemand,
            "provisioned-concurrency" => Self::ProvisionedConcurrency,
            // Shouldn't reach here but if for some reason AWS doesn't get it right...
            _ => Self::Unknown,
        }
    }
}

/// An interface trait for reading the environment variables set by the AWS Lambda service.
///
/// Based on - [Defined runtime environment variables](https://docs.aws.amazon.com/lambda/latest/dg/configuration-envvars.html#configuration-envvars-runtime).
pub trait RuntimeEnvVars: Default {
    fn get_handler(&self) -> Option<&str>;
    fn get_region(&self) -> Option<&str>;
    fn get_trace_id(&self) -> Option<&str>;
    fn get_execution_env(&self) -> Option<&str>;
    fn get_function_name(&self) -> Option<&str>;
    fn get_function_memory_size(&self) -> Option<usize>;
    fn get_function_version(&self) -> Option<&str>;
    fn get_initialization_type(&self) -> InitializationType;
    fn get_log_group_name(&self) -> Option<&str>;
    fn get_log_stream_name(&self) -> Option<&str>;
    fn get_access_key(&self) -> Option<&str>;
    fn get_access_key_id(&self) -> Option<&str>;
    fn get_secret_access_key(&self) -> Option<&str>;
    fn get_session_token(&self) -> Option<&str>;
    fn get_runtime_api(&self) -> Option<&str>;
    fn get_task_root(&self) -> Option<&str>;
    fn get_runtime_dir(&self) -> Option<&str>;
    fn get_tz(&self) -> Option<&str>;
    /// Returns the string value of an env-var `var_name` wrapped in an [`Option`],
    /// or `None` if the env-var is not set or the [`std::env::var`] function returns an error.
    fn get_var(var_name: &str) -> Option<String> {
        use std::env;
        env::var(var_name).ok()
    }
    /// Signals that the previous tracing id has changed as a result of a new incoming event.
    fn set_trace_id(&mut self, new_id: Option<&str>);
}
