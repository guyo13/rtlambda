// Copyright 2022-2023 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::error::*;

pub static AWS_REQ_ID: &str = "Lambda-Runtime-Aws-Request-Id";
pub static AWS_DEADLINE_MS: &str = "Lambda-Runtime-Deadline-Ms";
pub static AWS_FUNC_ARN: &str = "Lambda-Runtime-Invoked-Function-Arn";
pub static AWS_TRACE_ID: &str = "Lambda-Runtime-Trace-Id";
pub static AWS_CLIENT_CTX: &str = "Lambda-Runtime-Client-Context";
pub static AWS_COG_ID: &str = "Lambda-Runtime-Cognito-Identity";
pub static AWS_FUNC_ERR_TYPE: &str = "Lambda-Runtime-Function-Error-Type";

//Based on [https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html#runtimes-api-next]
/// A trait serving as an abstraction of the response from the [AWS Lambda runtime API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
pub trait LambdaAPIResponse {
    // TODO: find out whether lambda might send a non-UTF-8 encoded json event and change signature if needed
    fn get_body(self) -> Result<String, Error>;
    fn get_status_code(&self) -> u16;
    fn get_aws_request_id(&self) -> Option<&str>;
    fn get_deadline(&self) -> Option<u64>;
    fn get_invoked_function_arn(&self) -> Option<&str>;
    fn get_x_ray_tracing_id(&self) -> Option<&str>;
    fn get_client_context(&self) -> Option<&str>;
    fn get_cognito_identity(&self) -> Option<&str>;
    fn is_success(&self) -> bool {
        matches!(self.get_status_code(), 200..=299)
    }

    fn is_client_err(&self) -> bool {
        matches!(self.get_status_code(), 400..=499)
    }

    fn is_server_err(&self) -> bool {
        matches!(self.get_status_code(), 500..=599)
    }

    fn is_err(&self) -> bool {
        matches!(self.get_status_code(), 400..=499 | 500..=599)
    }
}
