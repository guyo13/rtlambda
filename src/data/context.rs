// Copyright 2022-2023 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::api::{
    InitializationType, LambdaContext, LambdaContextSetter, LambdaEnvSetter, LambdaEnvVars,
};
use std::env::{remove_var, set_var};
use std::time::Duration;

static _X_AMZN_TRACE_ID: &str = "_X_AMZN_TRACE_ID";
static _HANDLER: &str = "_HANDLER";
static AWS_DEFAULT_REGION: &str = "AWS_DEFAULT_REGION";
static AWS_REGION: &str = "AWS_REGION";
static AWS_EXECUTION_ENV: &str = "AWS_EXECUTION_ENV";
static AWS_LAMBDA_FUNCTION_NAME: &str = "AWS_LAMBDA_FUNCTION_NAME";
static AWS_LAMBDA_FUNCTION_MEMORY_SIZE: &str = "AWS_LAMBDA_FUNCTION_MEMORY_SIZE";
static AWS_LAMBDA_FUNCTION_VERSION: &str = "AWS_LAMBDA_FUNCTION_VERSION";
static AWS_LAMBDA_INITIALIZATION_TYPE: &str = "AWS_LAMBDA_INITIALIZATION_TYPE";
static AWS_LAMBDA_LOG_GROUP_NAME: &str = "AWS_LAMBDA_LOG_GROUP_NAME";
static AWS_LAMBDA_LOG_STREAM_NAME: &str = "AWS_LAMBDA_LOG_STREAM_NAME";
static AWS_ACCESS_KEY: &str = "AWS_ACCESS_KEY";
static AWS_ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
static AWS_SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";
static AWS_SESSION_TOKEN: &str = "AWS_SESSION_TOKEN";
static AWS_LAMBDA_RUNTIME_API: &str = "AWS_LAMBDA_RUNTIME_API";
static LAMBDA_TASK_ROOT: &str = "LAMBDA_TASK_ROOT";
static LAMBDA_RUNTIME_DIR: &str = "LAMBDA_RUNTIME_DIR";
static TZ: &str = "TZ";

/// An implementation of [`LambdaContext`], [`LambdaContextSetter`] and [`LambdaEnvSetter`].
pub struct EventContext {
    pub handler: Option<String>,
    pub region: Option<String>,
    pub default_region: Option<String>,
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
    // These values are set by the runtime after each next invocation request
    pub trace_id: Option<String>,
    pub deadline: Option<Duration>,
    pub function_arn: Option<String>,
    pub request_id: Option<String>,
    pub cognito_id: Option<String>,
    pub client_context: Option<String>,
}

impl Default for EventContext {
    fn default() -> Self {
        use std::env;
        Self {
            handler: env::var(_HANDLER).ok(),
            default_region: env::var(AWS_DEFAULT_REGION).ok(),
            region: env::var(AWS_REGION).ok(),
            trace_id: None,
            execution_env: env::var(AWS_EXECUTION_ENV).ok(),
            function_name: env::var(AWS_LAMBDA_FUNCTION_NAME).ok(),
            function_memory_size: match env::var(AWS_LAMBDA_FUNCTION_MEMORY_SIZE).ok() {
                Some(v) => v.parse::<usize>().ok(),
                None => None,
            },
            function_version: env::var(AWS_LAMBDA_FUNCTION_VERSION).ok(),
            initialization_type: match env::var(AWS_LAMBDA_INITIALIZATION_TYPE).ok() {
                Some(v) => InitializationType::from_string(&v),
                None => InitializationType::Unknown,
            },
            log_group_name: env::var(AWS_LAMBDA_LOG_GROUP_NAME).ok(),
            log_stream_name: env::var(AWS_LAMBDA_LOG_STREAM_NAME).ok(),
            access_key: env::var(AWS_ACCESS_KEY).ok(),
            access_key_id: env::var(AWS_ACCESS_KEY_ID).ok(),
            secret_access_key: env::var(AWS_SECRET_ACCESS_KEY).ok(),
            session_token: env::var(AWS_SESSION_TOKEN).ok(),
            runtime_api: env::var(AWS_LAMBDA_RUNTIME_API).ok(),
            task_root: env::var(LAMBDA_TASK_ROOT).ok(),
            runtime_dir: env::var(LAMBDA_RUNTIME_DIR).ok(),
            tz: env::var(TZ).ok(),
            deadline: None,
            function_arn: None,
            request_id: None,
            cognito_id: None,
            client_context: None,
        }
    }
}

impl LambdaEnvVars for EventContext {
    #[inline(always)]
    fn get_handler_location(&self) -> Option<&str> {
        self.handler.as_deref()
    }

    #[inline(always)]
    fn get_aws_default_region(&self) -> Option<&str> {
        self.default_region.as_deref()
    }

    #[inline(always)]
    fn get_aws_region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    #[inline(always)]
    fn get_x_ray_tracing_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    #[inline(always)]
    fn get_execution_env(&self) -> Option<&str> {
        self.execution_env.as_deref()
    }

    #[inline(always)]
    fn get_lambda_function_name(&self) -> Option<&str> {
        self.function_name.as_deref()
    }

    #[inline(always)]
    fn get_lambda_function_memory_size(&self) -> Option<usize> {
        self.function_memory_size
    }

    #[inline(always)]
    fn get_lambda_function_version(&self) -> Option<&str> {
        self.function_version.as_deref()
    }

    #[inline(always)]
    fn get_lambda_initialization_type(&self) -> InitializationType {
        self.initialization_type
    }
    #[inline(always)]
    fn get_lambda_log_group_name(&self) -> Option<&str> {
        self.log_group_name.as_deref()
    }

    #[inline(always)]
    fn get_lambda_log_stream_name(&self) -> Option<&str> {
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
    fn get_lambda_runtime_api(&self) -> Option<&str> {
        self.runtime_api.as_deref()
    }

    #[inline(always)]
    fn get_lambda_task_root(&self) -> Option<&str> {
        self.task_root.as_deref()
    }

    #[inline(always)]
    fn get_lambda_runtime_dir(&self) -> Option<&str> {
        self.runtime_dir.as_deref()
    }

    #[inline(always)]
    fn get_tz(&self) -> Option<&str> {
        self.tz.as_deref()
    }
}

impl LambdaEnvSetter for EventContext {
    #[inline]
    fn set_x_ray_tracing_id(&mut self, new_id: Option<&str>) {
        // If AWS returns the "Lambda-Runtime-Trace-Id" header, assign its value to the -
        // "_X_AMZN_TRACE_ID" env var
        if let Some(req_id) = new_id {
            set_var(_X_AMZN_TRACE_ID, req_id);
            self.trace_id = new_id.map(|v| v.to_string());
        } else {
            remove_var(_X_AMZN_TRACE_ID);
            self.trace_id = None;
        };
    }
}

impl LambdaContext for EventContext {
    #[inline]
    fn get_deadline(&self) -> Option<Duration> {
        self.deadline
    }

    #[inline(always)]
    fn get_invoked_function_arn(&self) -> Option<&str> {
        self.function_arn.as_deref()
    }

    #[inline(always)]
    fn get_aws_request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    #[inline(always)]
    fn get_cognito_identity(&self) -> Option<&str> {
        self.cognito_id.as_deref()
    }

    #[inline(always)]
    fn get_client_context(&self) -> Option<&str> {
        self.client_context.as_deref()
    }
}

impl LambdaContextSetter for EventContext {
    fn set_deadline(&mut self, dl: Option<Duration>) {
        self.deadline = dl;
    }

    fn set_invoked_function_arn(&mut self, arn: Option<&str>) {
        self.function_arn = arn.map(|s| s.to_string());
    }

    fn set_aws_request_id(&mut self, request_id: Option<&str>) {
        self.request_id = request_id.map(|s| s.to_string());
    }

    fn set_cognito_identity(&mut self, identity: Option<&str>) {
        self.cognito_id = identity.map(|s| s.to_string());
    }

    fn set_client_context(&mut self, ctx: Option<&str>) {
        self.client_context = ctx.map(|s| s.to_string());
    }
}
