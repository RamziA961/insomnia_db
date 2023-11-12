use std::{sync::Arc, time::Duration};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc, Semaphore},
};
use tracing::{error, info};

use crate::connection::Connection;

use super::{
    database::database_guard::DatabaseGuard, handler::Handler, shutdown_listener::ShutdownListener,
};

#[allow(unused)]
#[derive(Debug)]
struct Listener {
    /// Shared database handle
    db_owner: DatabaseGuard,

    /// Tcp listener for client requests
    listener: TcpListener,

    /// Mechanism to limit number of concurrent connections
    ///
    /// When a processing a connection is complete, a permit is returned to the semaphore
    connection_limit: Arc<Semaphore>,

    /// Server shutdown broadcast channel
    shutdown_notifier: broadcast::Sender<()>,

    /// Channel need to ensure all connections are processed before server shutdown
    shutdown_complete_channel: mpsc::Sender<()>,
}

impl Listener {
    pub(super) async fn run(&mut self) -> anyhow::Result<()> {
        info!("Server is live. Awaiting inbound connections.");

        loop {
            let permit = self.connection_limit.clone().acquire_owned().await.unwrap();

            let socket = self.accept().await?;

            let mut handler = Handler {
                database: self.db_owner.inner(),
                connection: Connection::new(socket),
                shutdown_listener: ShutdownListener::new(self.shutdown_notifier.subscribe()),
            };

            tokio::spawn(async move {
                if let Err(e) = handler.run().await {
                    error!(error = ?e, "connection handler failed")
                }
            });

            drop(permit)
        }
    }

    async fn accept(&mut self) -> anyhow::Result<TcpStream> {
        let mut backoff_period = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(e) => {
                    if backoff_period > 128 {
                        error!(error = %e, "failed to accept connection");
                        return Err(anyhow::anyhow!(e));
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(backoff_period)).await;
            backoff_period *= 2;
        }
    }
}
