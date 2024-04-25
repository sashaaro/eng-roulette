use std::fmt::{Debug, Display};
use clap::Parser;
use tonic::transport::Server;
use crate::server::helloworld::greeter_server;
use crate::server::helloworld::{HelloRequest, HelloReply};
use crate::greeter_server::GreeterServer;

mod cli;
mod r#box;
mod r#server;

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl greeter_server::Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> { // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(tonic::Response::new(reply)) // Send back our formatted greeting
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    println!("Room service starting..");
    


    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}

