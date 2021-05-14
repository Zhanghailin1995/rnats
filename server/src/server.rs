use crate::errors::Error;
use crate::{connection::Connection, protocol::NatsProtocol};
use bytes::Bytes;
use futures_util::stream::StreamExt;
use log::{error, info, trace};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
    time::{self, Duration},
};

use std::future::Future;
#[derive(Debug)]
struct Listener {
    db: Db,
    listener: TcpListener,
}

impl Listener {
    async fn run(&mut self) -> Result<(), Error> {
        trace!("accepting inbound connections");
        loop {
            let socket = self.accept().await?;
            let mut handler = Handler {
                db: self.db.clone(),
                conn: Connection::new(socket),
            };

            tokio::spawn(async move {
                let _ = handler.run().await;
            });
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
    db: Db,
}

impl Handler {
    /// Process a single connection.
    async fn run(&mut self) -> Result<(), Error> {
        loop {
            if let Some(next) = self.conn.stream.next().await {
                next?.apply(&self.db, &mut self.conn).await?;
            } else {
                info!("connect closed");
                break;
            }
        }
        Ok(())
    }
}

pub async fn run(listener: TcpListener, shutdown: impl Future) -> Result<(), Error> {
    let mut server = Listener {
        listener,
        db: Db::new(),
    };
    info!("server run:{}", 1234);

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!("server err {:?}", err);
            }
        }
        _ = shutdown => {
            info!("shutting down");
        }
    }
    Ok(())
    //server.run().await
}

#[derive(Debug, Clone)]
pub(crate) struct Db {
    /// Handle to shared state. The background task will also have an
    /// `Arc<Shared>`.
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    /// The shared state is guarded by a mutex. This is a `std::sync::Mutex` and
    /// not a Tokio mutex. This is because there are no asynchronous operations
    /// being performed while holding the mutex. Additionally, the critical
    /// sections are very small.
    state: Mutex<State>,
}

#[derive(Debug)]
struct State {
    /// The pub/sub key-space.
    pub_sub: HashMap<String, broadcast::Sender<Bytes>>,
    // shutdown: bool,
}

impl Db {
    /// Create a new, empty, `Db` instance. Allocates shared state and spawns a
    /// background task to manage key expiration.
    pub(crate) fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                pub_sub: HashMap::new(),
                // shutdown: false,
            }),
        });

        Db { shared }
    }

    /// Returns a `Receiver` for the requested channel.
    ///
    /// The returned `Receiver` is used to receive values broadcast by `PUB`
    /// commands.
    pub(crate) fn subscribe(&self, key: String) -> broadcast::Receiver<Bytes> {
        use std::collections::hash_map::Entry;

        // Acquire the mutex
        let mut state = self.shared.state.lock().unwrap();

        match state.pub_sub.entry(key) {
            Entry::Occupied(e) => e.get().subscribe(),
            Entry::Vacant(e) => {
                // No broadcast channel exists yet, so create one.
                //
                // The channel is created with a capacity of `1024` messages. A
                // message is stored in the channel until **all** subscribers
                // have seen it. This means that a slow subscriber could result
                // in messages being held indefinitely.
                //
                // When the channel's capacity fills up, publishing will result
                // in old messages being dropped. This prevents slow consumers
                // from blocking the entire system.
                let (tx, rx) = broadcast::channel(1024);
                e.insert(tx);
                rx
            }
        }
    }

    /// Publish a message to the channel. Returns the number of subscribers
    /// listening on the channel.
    pub(crate) fn publish(&self, key: &str, value: Bytes) -> usize {
        let state = self.shared.state.lock().unwrap();

        state
            .pub_sub
            .get(key)
            // On a successful message send on the broadcast channel, the number
            // of subscribers is returned. An error indicates there are no
            // receivers, in which case, `0` should be returned.
            .map(|tx| tx.send(value).unwrap_or(0))
            // If there is no entry for the channel key, then there are no
            // subscribers. In this case, return `0`.
            .unwrap_or(0)
    }
}
