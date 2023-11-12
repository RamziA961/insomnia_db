use crate::connection::Connection;

use super::{database::database::Database, shutdown_listener::ShutdownListener};

pub(super) struct Handler {
    pub(super) database: Database,

    /// Tcp connection with encoding/decoding capabilities
    pub(super) connection: Connection,

    ///
    pub(super) shutdown_listener: ShutdownListener,
}

impl Handler {
    pub(super) async fn run(&mut self) -> anyhow::Result<()> {
        while !self.shutdown_listener.has_shutdown() {
            let frame_opt = tokio::select! {
                fr = self.connection.read_frame() => fr?,
                _ = self.shutdown_listener.subscribe() => {
                    return Ok(());
                }
            };

            let frame = if let Some(frame) = frame_opt {
                frame
            } else {
                return Ok(());
            };

            todo!("Finish command struct")
        }

        Ok(())
    }
}
