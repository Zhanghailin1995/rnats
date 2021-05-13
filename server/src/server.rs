use crate::connection::Connection;
use crate::errors::Error;
use futures_util::stream::StreamExt;
use tokio::{
    net::{TcpListener, TcpStream},
    time::{self, Duration},
};
#[derive(Debug)]
struct Listener {
    listener: TcpListener,
}

impl Listener {
    async fn run(&mut self) -> Result<(), Error> {
        println!("accepting inbound connections");
        loop {
            let socket = self.accept().await?;
            let mut handler = Handler {
                conn: Connection::new(socket),
            };

            tokio::spawn(async move { handler.run().await });
        }
    }

    async fn accept(&mut self) -> Result<TcpStream, Error> {
        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        // Accept has failed too many times. Return the error.
                        return Err(err.into());
                    }
                }
            }

            // Pause execution until the back off period elapses.
            time::sleep(Duration::from_secs(backoff)).await;

            // Double the back off
            backoff *= 2;
        }
    }
}
#[derive(Debug)]
struct Handler {
    conn: Connection,
}

impl Handler {
    /// Process a single connection.
    async fn run(&mut self) -> Result<(), Error> {
        // let mut codec = NatsMessageCodec {
        //     state: ParseState::OpStart
        // };
        // loop {
        //     let mut framed = Framed::new(&mut self.conn.stream, &mut codec);
        // }
        loop {
            let next = self.conn.stream.next().await.unwrap();
            println!("{:?}", next.unwrap())
        }
        // unimplemented!()
    }
}

pub async fn run(listener: TcpListener) -> Result<(), Error> {
    // unimplemented!();
    let mut server = Listener { listener };
    println!("server run:{}", 1234);

    server.run().await
}
