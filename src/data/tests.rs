#[cfg(test)]
mod tests {
    use crate::api::{InitializationType, LambdaContext, LambdaContextSetter, LambdaEnvVars};
    use crate::data::context::EventContext;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_context_default_from_env() {
        // Set some env vars
        env::set_var("AWS_LAMBDA_FUNCTION_NAME", "my-func");
        env::set_var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE", "128");
        env::set_var("AWS_LAMBDA_INITIALIZATION_TYPE", "on-demand");
        env::set_var("_HANDLER", "index.handler");

        let ctx = EventContext::default();

        assert_eq!(ctx.get_lambda_function_name(), Some("my-func"));
        assert_eq!(ctx.get_lambda_function_memory_size(), Some(128));
        assert!(matches!(
            ctx.get_lambda_initialization_type(),
            InitializationType::OnDemand
        ));
        assert_eq!(ctx.get_handler_location(), Some("index.handler"));

        // Clean up
        env::remove_var("AWS_LAMBDA_FUNCTION_NAME");
        env::remove_var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE");
        env::remove_var("AWS_LAMBDA_INITIALIZATION_TYPE");
        env::remove_var("_HANDLER");
    }

    #[test]
    fn test_context_setters() {
        let mut ctx = EventContext::default();

        ctx.set_aws_request_id(Some("req-123"));
        assert_eq!(ctx.get_aws_request_id(), Some("req-123"));

        ctx.set_invoked_function_arn(Some("arn:aws:lambda:us-east-1:123456789012:function:my-func"));
        assert_eq!(
            ctx.get_invoked_function_arn(),
            Some("arn:aws:lambda:us-east-1:123456789012:function:my-func")
        );
    }
}
