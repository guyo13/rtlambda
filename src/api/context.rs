use crate::error::Error;
use std::time::Duration;

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

/// A trait defining a setter interface that are used for setting context variables that vary between lambda events.
pub trait LambdaContextSetter {
    fn set_deadline(&mut self, dl: Option<Duration>);
    fn set_invoked_function_arn(&mut self, arn: Option<&str>);
    fn set_aws_request_id(&mut self, request_id: Option<&str>);
    fn set_cognito_identity(&mut self, identity: Option<&str>);
    fn set_client_context(&mut self, ctx: Option<&str>);
}
