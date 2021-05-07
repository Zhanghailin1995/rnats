use tokio_util::codec::Decoder;
use subslice::SubsliceExt;
use bytes::{BytesMut, Buf, Bytes};

struct NatsMessageDecoder;

#[derive(Debug, Clone, PartialEq)]
pub struct NatsMsg {
    pub subject: String,
    pub sid: usize,
    pub size: usize,
    pub payload: Bytes,
}

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
    type Item = String;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.starts_with(b"SUB ") {
            let line_end = if let Some(end) = src.find(b"\r\n") {
                end
            } else {
                return Ok(None);
            };
            let mut parts = src[4..line_end].split(|c|c==&b' ');
            // let subject =
        }

        unimplemented!()
    }
}



