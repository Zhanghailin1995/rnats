use tokio::net::TcpListener;
use rnats::errors::Error;
use rnats::server;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    // Bind a TCP listener
    let port = 1234;
    let listener = TcpListener::bind(&format!("192.168.1.83:{}", port)).await?;

    server::run(listener).await
}