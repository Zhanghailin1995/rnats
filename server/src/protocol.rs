use crate::{
    connection::Connection, errors::Error, publish::Publish, server::Db, subscribe::Subscribe,
};
use bytes::{Buf, Bytes, BytesMut};

use log::info;
use subslice::SubsliceExt;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct NatsMessageCodec {
    state: ParseState,
}

impl NatsMessageCodec {
    pub fn new() -> NatsMessageCodec {
        NatsMessageCodec {
            state: ParseState::OpStart,
        }
    }
}
#[derive(Debug)]
pub enum ParseState {
    OpStart,
    OpSub,
    OpPub,
}

#[derive(Debug)]
pub enum NatsProtocol {
    // Msg(NatsMsg),
    Sub(Subscribe),
    Pub(Publish),
}

impl NatsProtocol {
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> Result<(), Error> {
        use NatsProtocol::*;
        match self {
            Sub(s) => s.apply(db, dst).await,
            Pub(p) => p.apply(db, dst).await,
        }
    }
}

impl Decoder for NatsMessageCodec {
    type Item = NatsProtocol;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        use ParseState::*;
        info!("recv msg {:?}", std::str::from_utf8(&src[..]));
        loop {
            match self.state {
                OpStart => {
                    if src.starts_with(b"SUB ") {
                        self.state = OpSub;
                        src.advance(4);
                    } else if src.starts_with(b"PUB ") {
                        self.state = OpPub;
                        src.advance(4);
                    } else {
                        return Ok(None);
                    }
                }
                OpSub => {
                    let line_end = match src.find(b"\r\n") {
                        Some(end) => end,
                        None => return Ok(None),
                    };
                    // SUB <subject> [subjects...]\r\n

                    let parts = src[..line_end].split(|c| c == &b' ');
                    let mut channels = Vec::new();
                    // first arg is always subject
                    for i in parts.into_iter() {
                        channels.push(std::str::from_utf8(i)?.to_string())
                    }
                    self.state = OpStart;
                    src.advance(line_end + 2);
                    return Ok(Some(NatsProtocol::Sub(Subscribe { channels })));
                }
                OpPub => {
                    // PUB <subject> <len>\r\n<message>\r\n
                    let line_end = if let Some(end) = src.find(b"\r\n") {
                        end
                    } else {
                        return Ok(None);
                    };
                    let mut parts = src[..line_end].split(|c| c == &b' ');
                    let channel =
                        std::str::from_utf8(parts.next().ok_or_else(|| Error::ProtocolError)?)?
                            .to_string();
                    let size =
                        std::str::from_utf8(parts.next().ok_or_else(|| Error::ProtocolError)?)?
                            .parse::<usize>()?;
                    if line_end + size + 4 <= src.len() {
                        src.advance(line_end + 2);
                        let message = src.split_to(size);
                        src.advance(2);
                        self.state = OpStart;
                        return Ok(Some(NatsProtocol::Pub(Publish {
                            channel,
                            size,
                            message: message.freeze(),
                        })));
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
    }
}

impl Encoder<(String, Bytes)> for NatsMessageCodec {
    type Error = Error;
    // MESSAGE
    // MSG <subject> <size>\r\n
    // <message>\r\n
    fn encode(&mut self, item: (String, Bytes), dst: &mut BytesMut) -> Result<(), Self::Error> {
        // unimplemented!()
        dst.extend_from_slice(b"MSG ");
        dst.extend_from_slice(item.0.as_bytes());
        dst.extend_from_slice(b" ");
        dst.extend_from_slice(format!(" {}\r\n", item.1.len()).as_bytes());
        dst.extend_from_slice(item.1.as_ref());
        Ok(())
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_decode() {
        let mut decoder = NatsMessageCodec {
            state: ParseState::OpStart,
        };
        let mut buf = BytesMut::from("aa".as_bytes());
        assert!(decoder.decode(&mut buf).unwrap().is_none());
        // test pub
        // PUB <subject> <len>\r\n<message>\r\n
        let mut buf = BytesMut::from("PUB subject 5\r\nhello\r\n".as_bytes());
        let result = decoder.decode(&mut buf).unwrap().unwrap();
        println!("{:?}", result);

        // test sub
        // SUB <subject> <sid>\r\n
        let mut buf = BytesMut::from("SUB subject 5\r\n".as_bytes());
        let result = decoder.decode(&mut buf).unwrap().unwrap();
        println!("{:?}", result);

        // test sub
        // SUB <subject> <sid>\r\n
        let mut buf = BytesMut::from("SUB subject queue 5\r\n".as_bytes());
        let result = decoder.decode(&mut buf).unwrap().unwrap();
        println!("{:?}", result);
    }
}
