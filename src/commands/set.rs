use bytes::Bytes;
use std::time::Duration;
use tracing::{debug, instrument};

#[cfg(feature = "server")]
use async_trait::async_trait;

use crate::{
    commands::{Command, Execute},
    connection::Connection,
    frame::{Frame, FrameError},
    parse::{Parse, ParseError},
};

#[cfg(feature = "server")]
use crate::server::{database::database::Database, shutdown_listener::ShutdownListener};

#[derive(Debug)]
pub(crate) struct Set {
    key: String,
    value: Bytes,
    expiration: Option<Duration>,
}

impl Set {
    pub(crate) fn new(key: impl ToString, value: Bytes, expiration: Option<Duration>) -> Self {
        Self {
            key: key.to_string(),
            value,
            expiration,
        }
    }

    pub(crate) fn key(&self) -> &str {
        &self.key
    }

    pub(crate) fn value(&self) -> &Bytes {
        &self.value
    }

    pub(crate) fn expiration(&self) -> Option<Duration> {
        self.expiration
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl Execute for Set {
    #[instrument(skip(db, conn))]
    async fn execute(
        self,
        db: &Database,
        conn: &mut Connection,
        shutdown: &mut ShutdownListener,
    ) -> anyhow::Result<()> {
        db.set(self.key, self.value, self.expiration);

        let res = Frame::Simple("OK".to_string());
        debug!(?res);
        conn.write_frame(&res).await?;
        Ok(())
    }
}

impl Command for Set {
    fn representation<'a>() -> &'a str {
        "set"
    }

    fn parse_from_frame(parser: &mut Parse) -> anyhow::Result<Self> {
        let key = parser.next_string()?;
        let value = parser.next_bytes()?;

        // handle data
        let expiration = match parser.next_string() {
            Ok(s) if s.to_lowercase() == "ex" => {
                let secs = parser.next_int()?;
                Some(Duration::from_secs(secs))
            }
            Ok(s) if s.to_lowercase() == "px" => {
                let ms = parser.next_int()?;
                Some(Duration::from_millis(ms))
            }
            Ok(_) => return Err(anyhow::anyhow!("`SET` only supports an expiration option.")),
            Err(ParseError::EndOfStream) => None,
            Err(e) => return Err(e.into()),
        };

        Ok(Self {
            key,
            value,
            expiration,
        })
    }
}

impl TryInto<Frame> for Set {
    type Error = FrameError;

    fn try_into(self) -> Result<Frame, Self::Error> {
        let mut frame = Frame::Array(vec![]);

        frame.push_bulk(Bytes::from(Self::representation().as_bytes().to_owned()))?;
        frame.push_bulk(Bytes::from(self.key.into_bytes()))?;
        frame.push_bulk(self.value)?;

        if let Some(t) = self.expiration {
            frame.push_bulk(Bytes::from("px".as_bytes()))?;
            frame.push_int(t.as_millis() as u64)?;
        }

        Ok(frame)
    }
}
