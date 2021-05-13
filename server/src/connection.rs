use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::protocol::NatsMessageCodec;

#[derive(Debug)]
pub struct Connection {
    pub stream: Framed<TcpStream, NatsMessageCodec>,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: Framed::new(socket, NatsMessageCodec::new()),
        }
    }
}
