use async_trait::async_trait;
use bytes::Bytes;
use tracing::{debug, instrument};

use crate::{
    connection::Connection,
    frame::{Frame, FrameError},
    parse::Parse,
    server::{database::database::Database, shutdown_listener::ShutdownListener},
};

use super::{Command, Execute};

#[derive(Debug)]
pub(crate) struct Get {
    key: String,
}

impl Get {
    pub(crate) fn new(key: impl ToString) -> Self {
        Self {
            key: key.to_string(),
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl Execute for Get {
    #[instrument(skip(db, conn))]
    async fn execute(
        self,
        db: &Database,
        conn: &mut Connection,
        shutdown: &mut ShutdownListener,
    ) -> anyhow::Result<()> {
        let res = if let Some(v) = db.get(&self.key) {
            Frame::Bulk(v)
        } else {
            Frame::Null
        };

        debug!(?res);

        conn.write_frame(&res).await?;
        Ok(())
    }
}

impl Command for Get {
    fn representation<'a>() -> &'a str {
        "get"
    }

    fn parse_from_frame(parser: &mut Parse) -> anyhow::Result<Self> {
        Ok(Get {
            key: parser.next_string()?,
        })
    }
}

impl TryInto<Frame> for Get {
    type Error = FrameError;

    fn try_into(self) -> Result<Frame, Self::Error> {
        let mut frame = Frame::Array(vec![]);

        frame.push_bulk(Bytes::from(Self::representation().as_bytes().to_owned()))?;
        frame.push_bulk(Bytes::from(self.key.into_bytes()))?;

        Ok(frame)
    }
}
