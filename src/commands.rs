#[cfg(feature = "server")]
use async_trait::async_trait;

use bytes::Bytes;
use std::pin::Pin;
use tokio_stream::Stream;

use crate::{connection::Connection, frame::Frame, parse::Parse};

#[cfg(feature = "server")]
use crate::server::{database::database::Database, shutdown_listener::ShutdownListener};

pub(crate) mod get;
pub(crate) mod ping;
pub(crate) mod publish;
pub(crate) mod set;
pub(crate) mod subscribe;

use get::Get;
use ping::Ping;
use publish::Publish;
use set::Set;
use subscribe::Subscribe;

type MessageStream = Pin<Box<dyn Stream<Item = Bytes> + Send + Sync>>;

pub(crate) enum SupportedCommand {
    Get(Get),
    Ping(Ping),
    Publish(Publish),
    Set(Set),
    Subscribe(Subscribe),
}

#[cfg(feature = "server")]
#[async_trait]
pub(crate) trait Execute {
    /// Apply queried commands.
    async fn execute(
        self,
        db: &Database,
        conn: &mut Connection,
        shutdown: &mut ShutdownListener,
    ) -> anyhow::Result<()>;
}

pub(crate) trait Command
where
    Self: Sized,
{
    /// String representation of the command to identify commands in client requests.
    fn representation<'a>() -> &'a str;

    /// Extract `Command` from `Frame`.
    fn parse_from_frame(parser: &mut Parse) -> anyhow::Result<Self>;
}

#[cfg(feature = "server")]
pub(crate) fn from_frame(frame: Frame) -> anyhow::Result<SupportedCommand> {
    use tracing::error;

    let mut parser = Parse::new(frame).map_err(|e| {
        error!(error = %e, "received frame could not be parsed.");
        e
    })?;

    let cmd_rep = parser.next_string()?.to_lowercase();

    let cmd = match cmd_rep {
        rep if rep == Ping::representation() => {
            SupportedCommand::Ping(Ping::parse_from_frame(&mut parser)?)
        }
        rep if rep == Get::representation() => {
            SupportedCommand::Get(Get::parse_from_frame(&mut parser)?)
        }
        rep if rep == Set::representation() => {
            SupportedCommand::Set(Set::parse_from_frame(&mut parser)?)
        }
        rep if rep == Publish::representation() => {
            SupportedCommand::Publish(Publish::parse_from_frame(&mut parser)?)
        }
        rep if rep == Subscribe::representation() => {
            SupportedCommand::Subscribe(Subscribe::parse_from_frame(&mut parser)?)
        }
        // unrecognized command
        _ => todo!(),
    };

    parser.finish()?;
    Ok(cmd)
}
