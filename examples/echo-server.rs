use rtlambda::prelude::*;
use serde::Serialize;

// Import the [`default_runtime`] macro from rtlambda.
#[macro_use]
extern crate rtlambda;

// Create a struct representing the lambda's response, and derive the [`serde::Serialize`] trait.
#[derive(Serialize, Clone)]
pub struct EchoMessage {
    msg: String,
    req_id: String,
}

// Implement the event handler
pub struct EchoEventHandler {}

impl EventHandler for EchoEventHandler {
    // Defines the Output type which must implement [`serde::Serialize`]
    type Output = EchoMessage;
    // The error type must implement the `Display` trait
    type Error = String;

    fn initialize(&mut self) -> Result<(), Self::Error> {
        // Initialization logic goes here...
        Ok(())
    }

    fn on_event<Ctx: LambdaContext>(
        &mut self,
        event: &str,
        context: &Ctx,
    ) -> Result<Self::Output, Self::Error> {
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
    // let mut runtime =  DefaultRuntime::<UreqTransport, EchoEventHandler>::new(LAMBDA_VER, EchoEventHandler {});
    let mut runtime = default_runtime!(EchoEventHandler {});

    runtime.run();
}
