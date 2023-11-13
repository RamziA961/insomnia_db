use bytes::Bytes;

#[cfg(feature = "server")]
use async_trait::async_trait;

use crate::{
    commands::Command,
    connection::Connection,
    frame::{Frame, FrameError},
    parse::Parse,
};

#[cfg(feature = "server")]
use crate::{
    commands::Execute,
    server::{database::database::Database, shutdown_listener::ShutdownListener},
};

#[derive(Debug)]
pub(crate) struct Publish {
    channel: String,
    message: Bytes,
}

impl Publish {
    pub(crate) fn new(channel: impl ToString, message: Bytes) -> Self {
        Self {
            channel: channel.to_string(),
            message,
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl Execute for Publish {
    async fn execute(
        self,
        db: &Database,
        conn: &mut Connection,
        _: &mut ShutdownListener,
    ) -> anyhow::Result<()> {
        let subs = db.publish(&self.channel, self.message);
        conn.write_frame(&Frame::Integer(subs as u64)).await?;
        Ok(())
    }
}

impl Command for Publish {
    fn representation<'a>() -> &'a str {
        "publish"
    }

    fn parse_from_frame(parser: &mut Parse) -> anyhow::Result<Self> {
        let channel = parser.next_string()?;
        let message = parser.next_bytes()?;

        Ok(Self { channel, message })
    }
}

impl TryInto<Frame> for Publish {
    type Error = FrameError;

    fn try_into(self) -> Result<Frame, Self::Error> {
        let mut frame = Frame::Array(vec![]);

        frame.push_bulk(Bytes::from(Self::representation().as_bytes().to_owned()))?;
        frame.push_bulk(Bytes::from(self.channel.into_bytes()))?;
        frame.push_bulk(Bytes::from(self.message))?;

        Ok(frame)
    }
}
