use async_trait::async_trait;
use bytes::Bytes;
use tracing::error;

use crate::{
    connection::Connection,
    frame::{Frame, FrameError},
    parse::{Parse, ParseError},
    server::{database::database::Database, shutdown_listener::ShutdownListener},
};

use super::{Command, Execute};

#[derive(Debug, Default)]
pub(crate) struct Ping {
    buffer: Option<Bytes>,
}

impl Ping {
    pub(crate) fn new(buffer: Option<Bytes>) -> Self {
        Self { buffer }
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl Execute for Ping {
    async fn execute(
        self,
        _: &Database,
        conn: &mut Connection,
        _: &mut ShutdownListener,
    ) -> anyhow::Result<()> {
        let res = match self.buffer {
            Some(buffer) => Frame::Bulk(buffer), // respond with received message
            None => Frame::Simple("PONG".to_string()),
        };

        conn.write_frame(&res).await?;
        Ok(())
    }
}

impl Command for Ping {
    fn representation<'a>() -> &'a str {
        "ping"
    }

    fn parse_from_frame(parser: &mut Parse) -> anyhow::Result<Self> {
        match parser.next_bytes() {
            Ok(buffer) => Ok(Self {
                buffer: Some(buffer),
            }),
            Err(ParseError::EndOfStream) => Ok(Ping::default()),
            Err(e) => Err(e.into()),
        }
    }
}

impl TryInto<Frame> for Ping {
    type Error = FrameError;

    fn try_into(self) -> Result<Frame, Self::Error> {
        let mut frame = Frame::Array(vec![]);

        frame.push_bulk(Bytes::from(Self::representation().as_bytes().to_owned()))?;

        if let Some(buffer) = self.buffer {
            frame.push_bulk(buffer).map_err(|e| {
                error!(error = %e, "protocol error encountered converting PING payload to Frame");
                e
            })?;
        }

        Ok(frame)
    }
}
