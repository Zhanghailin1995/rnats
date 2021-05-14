use tokio::net::TcpListener;
use rnats::errors::Error;
use rnats::server;
extern crate env_logger;
use tokio::signal;
#[tokio::main]
pub async fn main() -> Result<(), Error> {
    env_logger::init();
    // Bind a TCP listener
    let port = 1234;
    let listener = TcpListener::bind(&format!("192.168.1.83:{}", port)).await?;

    server::run(listener, signal::ctrl_c()).await
}