// Copyright 2022-2023 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::api::{
    LambdaAPIResponse, Transport, AWS_CLIENT_CTX, AWS_COG_ID, AWS_DEADLINE_MS, AWS_FUNC_ARN,
    AWS_REQ_ID, AWS_TRACE_ID,
};
use crate::error::Error;
use ureq::Agent;
use ureq::Response;

use std::time::Duration;

/// A wrapper that processes a [ureq::Response] and implements the [`crate::api::LambdaAPIResponse`] trait.
pub struct UreqResponse {
    resp: ureq::Response,
}

impl UreqResponse {
    /// A constructor that consumes a [ureq::Response] by copying the relevant headers and reading the request body.
    fn from_response(resp: ureq::Response) -> Result<Self, Error> {
        Ok(Self { resp })
    }
}

impl LambdaAPIResponse for UreqResponse {
    #[inline(always)]
    fn get_body(self) -> Result<String, Error> {
        match self.resp.into_string() {
            Ok(data) => Ok(data),
            Err(err) => Err(Error::new(format!("{}", err))),
        }
    }

    #[inline(always)]
    fn get_status_code(&self) -> u16 {
        self.resp.status()
    }

    #[inline]
    fn get_aws_request_id(&self) -> Option<&str> {
        self.resp.header(AWS_REQ_ID)
    }

    #[inline]
    fn get_deadline(&self) -> Option<u64> {
        match self.resp.header(AWS_DEADLINE_MS) {
            Some(ms) => match ms.parse::<u64>() {
                Ok(val) => Some(val),
                Err(_) => None,
            },
            None => None,
        }
    }

    #[inline]
    fn get_invoked_function_arn(&self) -> Option<&str> {
        self.resp.header(AWS_FUNC_ARN)
    }

    #[inline]
    fn get_x_ray_tracing_id(&self) -> Option<&str> {
        self.resp.header(AWS_TRACE_ID)
    }

    #[inline]
    fn get_client_context(&self) -> Option<&str> {
        self.resp.header(AWS_CLIENT_CTX)
    }

    #[inline]
    fn get_cognito_identity(&self) -> Option<&str> {
        self.resp.header(AWS_COG_ID)
    }
}

/// Wraps a [`ureq::Agent`] to implement the [`crate::transport::Transport`] trait.
///
/// AWS runtime instructs the implementation to disable timeout on the next invocation call.
/// This implementation achieves this by creating a [`ureq::Agent`] with 1 day in seconds of timeout.
pub struct UreqTransport {
    agent: Agent,
}

impl Default for UreqTransport {
    /// Creates a new transport objects with an underlying [ureq::Agent] that will (practically) not time out.
    fn default() -> Self {
        let agent = ureq::builder().timeout(Duration::from_secs(86400)).build();
        UreqTransport { agent }
    }
}

impl UreqTransport {
    /// Sends a request using the underlying agent.
    fn request(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        headers: Option<(Vec<&str>, Vec<&str>)>,
    ) -> Result<Response, Error> {
        let mut req = self.agent.request(method, url);
        if let Some(headers) = headers {
            let (keys, values) = headers;
            let len = std::cmp::min(keys.len(), values.len());
            for i in 0..len {
                req = req.set(keys[i], values[i]);
            }
        }
        if let Some(body) = body {
            return req
                .send_string(body)
                .map_err(|err| Error::new(format!("{}", err)));
        }
        req.call().map_err(|err| Error::new(format!("{}", err)))
    }
}

impl Transport for UreqTransport {
    type Response = UreqResponse;

    fn get(
        &self,
        url: &str,
        body: Option<&str>,
        headers: Option<(Vec<&str>, Vec<&str>)>,
    ) -> Result<Self::Response, Error> {
        let res = self.request("GET", url, body, headers);
        if let Ok(res) = res {
            return Self::Response::from_response(res);
        }
        Err(res.unwrap_err())
    }

    fn post(
        &self,
        url: &str,
        body: Option<&str>,
        headers: Option<(Vec<&str>, Vec<&str>)>,
    ) -> Result<Self::Response, Error> {
        let res = self.request("POST", url, body, headers);
        if let Ok(res) = res {
            return Self::Response::from_response(res);
        }
        Err(res.unwrap_err())
    }
}
