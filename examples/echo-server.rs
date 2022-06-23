use rtlambda::{
    data::{env::RuntimeEnvVars, response::LambdaAPIResponse},
    prelude::*,
};
use serde::Serialize;

// Import the [`default_runtime`] macro from rtlambda.
#[macro_use]
extern crate rtlambda;

// Create a struct representing the lambda's response, and derive the [`serde::Serialize`] trait.
#[derive(Serialize, Clone)]
struct EchoMessage {
    msg: String,
    req_id: String,
}

// Define output and error types for berevity.
// The Output type must implement [`serde::Serialize`]
type OUT = EchoMessage;
// The error type must implement the `Display` trait
type ERR = String;

// Implement the event handler
pub struct EchoEventHandler {}

impl<R: LambdaAPIResponse, ENV: RuntimeEnvVars> EventHandler<OUT, ERR, EventContext<ENV, R>>
    for EchoEventHandler
{
    fn initialize(&mut self) -> Result<(), ERR> {
        // Initialization logic goes here...
        Ok(())
    }
    fn on_event(&mut self, event: &str, context: &EventContext<ENV, R>) -> Result<OUT, ERR> {
        // Get the aws request id
        let req_id = context.aws_request_id().unwrap();

        if event == "\"\"" {
            return Err(format!("Empty input, nothing to echo."));
        }

        // Echo the event back as a string.
        Ok(EchoMessage {
            msg: format!("ECHO: {}", event),
            req_id: req_id.to_string(),
        })
    }
}

fn main() {
    // Create a runtime instance and run its loop.
    // This is the equivalent of:
    // let mut runtime =  DefaultRuntime::<UreqResponse, UreqTransport, EchoEventHandler, LambdaRuntimeEnv, OUT, ERR>::new(LAMBDA_VER, initialize);
    let mut runtime = default_runtime!(EchoEventHandler, OUT, ERR, LAMBDA_VER, EchoEventHandler {});

    runtime.run();
}
