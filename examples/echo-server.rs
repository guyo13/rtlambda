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
pub struct EchoEventHandler {
    my_int: i32, // If any of your resources are not [`std::marker::Sized`], use a Box!
                 // my_db_conn: Box<MyDynDbConnection>
}

impl EventHandler for EchoEventHandler {
    // Defines the Output type which must implement [`serde::Serialize`]
    type EventOutput = EchoMessage;
    // The error types must implement the `Display` trait
    type EventError = String;
    type InitError = String;

    fn initialize() -> Result<Self, Self::InitError> {
        // Initialization logic goes here...
        // Construct your EventHandler object
        Ok(Self { my_int: 42 })
        // If an error occurs during initialization return Err
        //Err("Something bad happened!".to_string())
    }

    fn on_event<Ctx: LambdaContext>(
        &mut self,
        event: &str,
        context: &Ctx,
    ) -> Result<Self::EventOutput, Self::EventError> {
        // Get the aws request id
        let req_id = context.get_aws_request_id().unwrap();

        if event == "\"\"" {
            return Err(format!("Empty input, nothing to echo."));
        }

        // Echo the event back as a string.
        Ok(EchoMessage {
            msg: format!("my_int: {}, ECHO: {}", self.my_int, event),
            req_id: req_id.to_string(),
        })
    }
}

fn main() {
    // Create a runtime instance and run its loop.
    // This is the equivalent of:
    // let mut runtime =  DefaultRuntime::<UreqTransport, EchoEventHandler>::new(LAMBDA_VER);
    let mut runtime = default_runtime!(EchoEventHandler);

    runtime.run();
}
