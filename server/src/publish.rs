use bytes::Bytes;

use crate::{connection::Connection, errors::Error, server::Db};


#[derive(Debug)]
pub struct Publish {
    pub channel: String,
    pub size: usize,
    pub message: Bytes,
}

impl Publish {
    /// Create a new `Publish` command which sends `message` on `channel`.
    pub(crate) fn new(channel: impl ToString, size:usize, message: Bytes) -> Publish {
        Publish {
            channel: channel.to_string(),
            size,
            message,
        }
    }

    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> Result<(), Error> {
        let _ = db.publish(&self.channel, self.message);

        // TODO wirte response to the client.
        Ok(())
    }
}