use async_trait::async_trait;

use crate::parse::Parse;

use crate::{connection::Connection, frame::Frame};

#[cfg(feature = "server")]
use crate::server::{database::database::Database, shutdown_listener::ShutdownListener};

pub(crate) mod get;
pub(crate) mod ping;

use get::Get;
use ping::Ping;

pub(crate) enum SupportedCommand {
    Get(Get),
    Ping(Ping),
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
        // unrecognized command
        _ => todo!(),
    };

    parser.finish()?;
    Ok(cmd)
}
