// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::data::env::RuntimeEnvVars;
use crate::data::response::LambdaAPIResponse;
use crate::error::Error;
use std::time::Duration;

use super::env::InitializationType;

/// A trait that should be implemented by types representing a [Context object]([https://docs.aws.amazon.com/lambda/latest/dg/python-context.html]).
///
/// The context object exposes constant data from the instance's environment variables,
/// as well as data - such as request id and execution deadline - that is specific to each event.
pub trait LambdaContext {
    /// A default implementation that calculates the time difference between its time of invocation and the
    /// handler execution deadline specified by AWS Lambda.
    fn get_remaining_time_ms(&self) -> Result<Duration, Error> {
        let now = std::time::SystemTime::now();
        match now.duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(now_since_epoch) => match self.get_deadline() {
                Some(dur) => dur,
                None => return Err(Error::new("Missing deadline info".to_string())),
            }
            .checked_sub(now_since_epoch)
            .ok_or_else(|| Error::new("Duration error".to_string())),
            Err(e) => Err(Error::new(e.to_string())),
        }
    }
    // Per-invocation data (event-related)
    fn get_deadline(&self) -> Option<Duration>;
    fn invoked_function_arn(&self) -> Option<&str>;
    fn aws_request_id(&self) -> Option<&str>;
    // Identity and Client context - see [https://docs.aws.amazon.com/lambda/latest/dg/python-context.html]
    // TODO - parse these structures and return a relevant type
    fn cognito_identity(&self) -> Option<&str>;
    fn client_context(&self) -> Option<&str>;
    // Per-runtime data (constant accross the lifetime of the runtime, taken from env-vars)
    fn function_name(&self) -> Option<&str>;
    fn function_version(&self) -> Option<&str>;
    fn memory_limit_in_mb(&self) -> Option<usize>;
    fn log_group_name(&self) -> Option<&str>;
    fn log_stream_name(&self) -> Option<&str>;
}

/// A generic implementation of [`LambdaContext`] that owns instances of types that implement -
/// [`crate::data::env::RuntimeEnvVars`] for reading environment variables, and
/// [`crate::data::response::LambdaAPIResponse`] for reading event-related data.
///
/// This implementation is used to avoid needlessly copying data that is immutable by definition,
/// however it is assumed that types implementing [`crate::data::response::LambdaAPIResponse`] can be read from
/// immutably - which is not the always case with HTTP Response types,
/// for example [ureq::Response](https://docs.rs/ureq/2.4.0/ureq/struct.Response.html#method.into_string) consumes itself upon reading the response body.
/// See [`crate::data::response::LambdaAPIResponse`].
pub struct EventContext<R: LambdaAPIResponse> {
    pub handler: Option<String>,
    // This value should be set by the runtime after each next invocation request where a new id is given
    pub trace_id: Option<String>,
    pub region: Option<String>,
    // Custom runtimes currently don't have this value set as per AWS docs
    pub execution_env: Option<String>,
    pub function_name: Option<String>,
    pub function_memory_size: Option<usize>,
    pub function_version: Option<String>,
    pub initialization_type: InitializationType,
    pub log_group_name: Option<String>,
    pub log_stream_name: Option<String>,
    pub access_key: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    pub runtime_api: Option<String>,
    pub task_root: Option<String>,
    pub runtime_dir: Option<String>,
    pub tz: Option<String>,
    /// An instance of a type implementing [`crate::data::response::LambdaAPIResponse`].
    pub invo_resp: Option<R>,
}

impl<R: LambdaAPIResponse> Default for EventContext<R> {
    fn default() -> Self {
        use std::env;
        Self {
            handler: env::var("_HANDLER").ok(),
            region: env::var("AWS_REGION").ok(),
            trace_id: None,
            execution_env: env::var("AWS_EXECUTION_ENV").ok(),
            function_name: env::var("AWS_LAMBDA_FUNCTION_NAME").ok(),
            function_memory_size: match env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE").ok() {
                Some(v) => v.parse::<usize>().ok(),
                None => None,
            },
            function_version: env::var("AWS_LAMBDA_FUNCTION_VERSION").ok(),
            initialization_type: match env::var("AWS_LAMBDA_INITIALIZATION_TYPE").ok() {
                Some(v) => InitializationType::from_string(&v),
                None => InitializationType::Unknown,
            },
            log_group_name: env::var("AWS_LAMBDA_LOG_GROUP_NAME").ok(),
            log_stream_name: env::var("AWS_LAMBDA_LOG_STREAM_NAME").ok(),
            access_key: env::var("AWS_ACCESS_KEY").ok(),
            access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
            secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            session_token: env::var("AWS_SESSION_TOKEN").ok(),
            runtime_api: env::var("AWS_LAMBDA_RUNTIME_API").ok(),
            task_root: env::var("LAMBDA_TASK_ROOT").ok(),
            runtime_dir: env::var("LAMBDA_RUNTIME_DIR").ok(),
            tz: env::var("TZ").ok(),
            invo_resp: None,
        }
    }
}

impl<R: LambdaAPIResponse> RuntimeEnvVars for EventContext<R> {
    #[inline(always)]
    fn get_handler(&self) -> Option<&str> {
        self.handler.as_deref()
    }

    #[inline(always)]
    fn get_region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    #[inline(always)]
    fn get_trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    #[inline(always)]
    fn get_execution_env(&self) -> Option<&str> {
        self.execution_env.as_deref()
    }

    #[inline(always)]
    fn get_function_name(&self) -> Option<&str> {
        self.function_name.as_deref()
    }

    #[inline(always)]
    fn get_function_memory_size(&self) -> Option<usize> {
        self.function_memory_size
    }

    #[inline(always)]
    fn get_function_version(&self) -> Option<&str> {
        self.function_version.as_deref()
    }

    #[inline(always)]
    fn get_initialization_type(&self) -> InitializationType {
        self.initialization_type
    }
    #[inline(always)]
    fn get_log_group_name(&self) -> Option<&str> {
        self.log_group_name.as_deref()
    }

    #[inline(always)]
    fn get_log_stream_name(&self) -> Option<&str> {
        self.log_stream_name.as_deref()
    }

    #[inline(always)]
    fn get_access_key(&self) -> Option<&str> {
        self.access_key.as_deref()
    }

    #[inline(always)]
    fn get_access_key_id(&self) -> Option<&str> {
        self.access_key_id.as_deref()
    }

    #[inline(always)]
    fn get_secret_access_key(&self) -> Option<&str> {
        self.secret_access_key.as_deref()
    }

    #[inline(always)]
    fn get_session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }

    #[inline(always)]
    fn get_runtime_api(&self) -> Option<&str> {
        self.runtime_api.as_deref()
    }

    #[inline(always)]
    fn get_task_root(&self) -> Option<&str> {
        self.task_root.as_deref()
    }

    #[inline(always)]
    fn get_runtime_dir(&self) -> Option<&str> {
        self.runtime_dir.as_deref()
    }

    #[inline(always)]
    fn get_tz(&self) -> Option<&str> {
        self.tz.as_deref()
    }

    #[inline]
    fn set_trace_id(&mut self, new_id: Option<&str>) {
        self.trace_id = new_id.map(|v| v.to_string());
    }
}

impl<R: LambdaAPIResponse> LambdaContext for EventContext<R> {
    #[inline]
    fn get_deadline(&self) -> Option<Duration> {
        self.invo_resp.as_ref().unwrap().deadline()
    }

    #[inline(always)]
    fn invoked_function_arn(&self) -> Option<&str> {
        self.invo_resp.as_ref().unwrap().invoked_function_arn()
    }

    #[inline(always)]
    fn aws_request_id(&self) -> Option<&str> {
        self.invo_resp.as_ref().unwrap().aws_request_id()
    }

    #[inline(always)]
    fn cognito_identity(&self) -> Option<&str> {
        self.invo_resp.as_ref().unwrap().cognito_identity()
    }

    #[inline(always)]
    fn client_context(&self) -> Option<&str> {
        self.invo_resp.as_ref().unwrap().client_context()
    }

    #[inline(always)]
    fn function_name(&self) -> Option<&str> {
        self.get_function_name()
    }

    #[inline(always)]
    fn function_version(&self) -> Option<&str> {
        self.get_function_version()
    }

    #[inline(always)]
    fn memory_limit_in_mb(&self) -> Option<usize> {
        self.get_function_memory_size()
    }

    #[inline(always)]
    fn log_group_name(&self) -> Option<&str> {
        self.get_log_group_name()
    }

    #[inline(always)]
    fn log_stream_name(&self) -> Option<&str> {
        self.get_log_stream_name()
    }
}
