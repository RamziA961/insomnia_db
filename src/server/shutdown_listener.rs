use tokio::sync::broadcast;

/// `ShutdownListener` listens for a shutdown singal from the server. Callers (clients) can query
/// the query the server for this event.
///
/// Once a value is sent via the broadcast channel, the server must shutdown.
#[derive(Debug)]
pub(crate) struct ShutdownListener {
    /// Reflects the server status.
    has_shutdown: bool,
    /// The channel which receives the signal
    channel: broadcast::Receiver<()>,
}

impl ShutdownListener {
    pub(crate) fn new(channel: broadcast::Receiver<()>) -> Self {
        Self {
            has_shutdown: false,
            channel,
        }
    }

    pub(crate) fn has_shutdown(&self) -> bool {
        self.has_shutdown
    }

    pub(crate) async fn subscribe(&mut self) {
        if self.has_shutdown {
            return;
        }

        // a value is only ever received through this channel when the server has shutdown
        let _ = self.channel.recv().await;
        self.has_shutdown = true;
    }
}
