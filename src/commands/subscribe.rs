use bytes::Bytes;

use super::{Command, MessageStream};
use crate::{
    connection::Connection,
    frame::{Frame, FrameError},
    parse::{Parse, ParseError},
};

#[cfg(feature = "server")]
use {
    super::Execute,
    crate::server::{database::database::Database, shutdown_listener::ShutdownListener},
    async_stream::stream,
    async_trait::async_trait,
    tokio::{select, sync::broadcast},
    tokio_stream::{StreamExt, StreamMap},
};

#[derive(Debug)]
pub(crate) struct Subscribe {
    channels: Vec<String>,
}

impl Subscribe {
    pub(crate) fn new(channels: Vec<String>) -> Self {
        Self { channels }
    }

    async fn subscribe_to_channel(
        channel_name: String,
        subs: &mut StreamMap<String, MessageStream>,
        db: &Database,
        conn: &mut Connection,
    ) -> anyhow::Result<()> {
        let mut rx = db.subscribe(channel_name.clone());

        let stream = Box::pin(stream! {
            loop {
                match rx.recv().await {
                    Ok(m) => yield m,
                    Err(broadcast::error::RecvError::Lagged(_)) => {/* do nothing */},
                    Err(_) => break,
                }
            }
        });

        subs.insert(channel_name.clone(), stream);
        conn.write_frame(&Self::assemble_subscribe_response(
            channel_name,
            subs.len() as u64,
        )?)
        .await?;
        Ok(())
    }

    fn assemble_subscribe_response(channel_name: String, num: u64) -> anyhow::Result<Frame> {
        let mut frame = Frame::Array(vec![]);
        frame.push_bulk(Bytes::from(Self::representation().as_bytes().to_owned()))?;
        frame.push_bulk(Bytes::from(channel_name.into_bytes()))?;
        frame.push_int(num)?;
        Ok(frame)
    }

    fn assemble_message(channel_name: String, message: Bytes) -> anyhow::Result<Frame> {
        let mut frame = Frame::Array(vec![]);
        frame.push_bulk(Bytes::from("message".as_bytes()))?;
        frame.push_bulk(Bytes::from(channel_name.into_bytes()))?;
        frame.push_bulk(message)?;
        Ok(frame)
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl Execute for Subscribe {
    async fn execute(
        self,
        db: &Database,
        conn: &mut Connection,
        shutdown_listener: &mut ShutdownListener,
    ) -> anyhow::Result<()> {
        // Some options for supporting other commands while subscribed:
        // * [client/server] Use a channel for message passing.
        // * [client] Spin up a unique thread for each command which awaits responses from the server.

        let mut subs = StreamMap::new();
        loop {
            // this can be optimized
            for ch in self.channels.iter() {
                Self::subscribe_to_channel(ch.clone(), &mut subs, db, conn).await?;
            }

            select! {
                Some((ch, m)) = subs.next() => conn.write_frame(&Self::assemble_message(ch, m)?).await?,
                _ = shutdown_listener.subscribe() => return Ok(())
            }
        }
    }
}

impl Command for Subscribe {
    fn representation<'a>() -> &'a str {
        "subscribe"
    }

    fn parse_from_frame(parser: &mut Parse) -> anyhow::Result<Self> {
        let mut channels = vec![parser.next_string()?];

        loop {
            match parser.next_string() {
                Ok(s) => channels.push(s),
                Err(ParseError::EndOfStream) => break,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(Self { channels })
    }
}

impl TryInto<Frame> for Subscribe {
    type Error = FrameError;

    fn try_into(self) -> Result<Frame, Self::Error> {
        let mut frame = Frame::Array(vec![]);
        frame.push_bulk(Bytes::from(Self::representation().as_bytes().to_owned()))?;

        for ch in self.channels {
            frame.push_bulk(Bytes::from(ch.into_bytes()))?;
        }

        Ok(frame)
    }
}
