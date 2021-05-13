
use tokio_util::codec::Decoder;
use subslice::SubsliceExt;
use bytes::{Buf, Bytes, BytesMut};
use crate::errors::Error;
struct NatsMessageDecoder {
    state: ParseState,
}

enum ParseState {
    OpStart,
    OpSub,
    OpPub,
}


// #[derive(Debug, Clone, PartialEq)]
// pub struct NatsMsg {
//     pub subject: String,
//     pub sid: usize,
//     pub size: usize,
//     pub payload: Bytes,
// }

#[derive(Debug, Clone, PartialEq)]
pub struct NatsPub {
    pub subject: String,
    pub size: usize,
    pub payload: Bytes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NatsSub {
    pub subject: String,
    pub sid: usize,
    pub queue: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum NatsProtocol {
    // Msg(NatsMsg),
    Sub(NatsSub),
    Pub(NatsPub),
}

impl Decoder for NatsMessageDecoder {
    type Item = NatsProtocol;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        use ParseState::*;
        loop {
            match self.state {
                OpStart => {
                    if src.starts_with(b"SUB ") {
                        self.state = OpSub;
                        src.advance(4);
                    } else if src.starts_with(b"PUB "){
                        self.state = OpPub;
                        src.advance(4);
                    } else {
                        return Ok(None);
                    }
                },
                OpSub => {
                    let line_end = match src.find(b"\r\n") {
                        Some(end) => end,
                        None => return Ok(None),
                    };
                    // SUB <subject> <sid>\r\n
                    // SUB <subject> <queue> <sid>
                    
                    let mut parts = src[..line_end].split(|c| c == &b' ');
                    // first arg is always subject
                    let subject =
                        std::str::from_utf8(parts.next().ok_or_else(|| Error::ProtocolError)?)?.to_string();
                        let mut queue = Option::<String>::None;
                        let sid;
                    let next_arg = parts.next().ok_or_else(|| Error::ProtocolError)?;
                    if let Some(arg) = parts.next() {
                        queue = Some(std::str::from_utf8(next_arg)?.to_string());
                        sid = std::str::from_utf8(arg)?.parse::<usize>()?;
                    } else {
                        sid = std::str::from_utf8(next_arg)?.parse::<usize>()?;
                    }
                    // skip body and \r\n
                    self.state = OpStart;
                    src.advance(line_end + 2);
                    return Ok(Some(NatsProtocol::Sub(NatsSub {
                        subject,
                        sid,
                        queue,
                    })));
                },
                OpPub => {
                    // PUB <subject> <len>\r\n<message>\r\n
                    let line_end = if let Some(end) = src.find(b"\r\n") {
                        end
                    } else {
                        return Ok(None);
                    };
                    let mut parts = src[..line_end].split(|c| c == &b' ');
                    let subject =
                        std::str::from_utf8(parts.next().ok_or_else(|| Error::ProtocolError)?)?.to_string();
                    let size = std::str::from_utf8(parts.next().ok_or_else(|| Error::ProtocolError)?)?
                        .parse::<usize>()?;
                    if line_end + size + 4 <= src.len() {
                        src.advance(line_end + 2);
                        let message = src.split_to(size);
                        src.advance(2);
                        self.state = OpStart;
                        return Ok(Some(NatsProtocol::Pub(NatsPub {
                            subject,
                            size,
                            payload: message.freeze(),
                        })));
                    } else {
                        return Ok(None)
                    }
                },
            }
        }
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_decode() {
        let mut decoder = NatsMessageDecoder {
            state: ParseState::OpStart
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



