use bytes::Bytes;
use futures_util::SinkExt;
use log::info;
use tokio::sync::broadcast;
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt, StreamMap};

use crate::{connection::Connection, errors::Error, protocol::NatsProtocol, server::Db};

#[derive(Clone, Debug)]
pub struct Subscribe {
    pub channels: Vec<String>,
}

type Messages = Pin<Box<dyn Stream<Item = Bytes> + Send>>;


impl Subscribe {
    /// Creates a new `Subscribe` command to listen on the specified channels.
    pub(crate) fn new(channels: &[String]) -> Subscribe {
        Subscribe {
            channels: channels.to_vec(),
        }
    }

    pub(crate) async fn apply(
        mut self,
        db: &Db,
        dst: &mut Connection,
    ) -> Result<(), Error> {
        let mut subscriptions = StreamMap::new();

        loop { 
            for channel_name in self.channels.drain(..) {
                subscribe_to_channel(channel_name, &mut subscriptions, db, dst)?;
            }
            // let res = dst.stream.next().await;
            
            tokio::select! {
                Some((channel_name, msg)) = subscriptions.next() => {
                    // write frame to dst
                    dst.stream.send((channel_name, msg)).await?;
                }
                //res = dst.stream.next() 
                res = dst.stream.next() => {
                    info!("recv new command {:?}", res);
                    let protocol = match res {
                        Some(protocol) => protocol?,
                        None => return Ok(()),
                    };
                    let _ = handle_command(protocol, &mut self.channels, &mut subscriptions, dst);
                }
            }
        }

        // Ok(())
    }
}

fn subscribe_to_channel(
    channel_name: String,
    subscriptions: &mut StreamMap<String, Messages>,
    db: &Db,
    dst: &mut Connection,
) -> Result<(), Error> {
    let mut rx = db.subscribe(channel_name.clone());

    let rx = Box::pin(async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(msg) => yield msg,
                // If we lagged in consuming messages, just resume.
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(_) => break,
            }
        }
    });

    // Track subscription in this client's subscription set.
    subscriptions.insert(channel_name.clone(), rx);

    // TODO Response with successful subscription

    Ok(())
}

fn handle_command(
    protocol: NatsProtocol,
    subscribe_to: &mut Vec<String>,
    subscriptions: &mut StreamMap<String, Messages>,
    dst: &mut Connection,
) -> Result<(), Error> {
    info!("{:?}", protocol);
    match protocol {
        NatsProtocol::Sub(s)=> {
            subscribe_to.extend(s.channels.into_iter());
        }
        _ => {

        }
    }

    Ok(())
}