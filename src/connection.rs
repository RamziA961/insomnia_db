use std::io::{self, Cursor};

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use anyhow::anyhow;
use thiserror::Error;
use tracing::error;

use crate::error::DatabaseError;
use crate::frame::{Frame, FrameError};

#[derive(Error, Debug)]
#[error("{0}")]
pub struct ConnectionError(pub anyhow::Error);

#[derive(Debug)]
pub(crate) struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub(crate) fn new(socket: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub fn with_capacity(socket: TcpStream, buf_capacity: usize) -> Self {
        Self {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(buf_capacity),
        }
    }

    pub(crate) async fn read_frame(&mut self) -> Result<Option<Frame>, DatabaseError> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    // close the connection as it is stale
                    return Err(
                        ConnectionError(anyhow!("Connection reset by peer.".to_string())).into(),
                    );
                }
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, DatabaseError> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::validate(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = Frame::parse(&mut buf)?;
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(e) => Err(e.into()),
        }
    }

    #[async_recursion::async_recursion]
    pub(crate) async fn write_frame(&mut self, frame: &Frame) -> Result<(), DatabaseError> {
        // recursive data structures?
        // async recursion not natively supported in rust
        match frame {
            Frame::Array(f) => {
                self.stream.write_u8(b'*').await?;

                // write num elements in Frame
                self.write_decimal(f.len() as u64).await?;

                for sub_f in f.iter() {
                    self.write_frame(sub_f).await?;
                }

                Ok(())
            }
            _ => self.write_value(frame).await,
        }
    }

    async fn write_value(&mut self, frame: &Frame) -> Result<(), DatabaseError> {
        match frame {
            Frame::Simple(s) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(s.as_bytes()).await
                // .map_err(|e| DatabaseError::from(e))
            }
            Frame::Error(e) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(e.as_bytes()).await
                // .map_err(|e| DatabaseError::from(e))
            }
            Frame::Integer(i) => {
                self.write_decimal(*i).await
                // .map_err(|e| DatabaseError::from(e))
            }
            Frame::Bulk(bs) => {
                self.stream.write_u8(b'$').await?;
                self.write_decimal(bs.len() as u64).await?;
                self.stream.write_all(b"\r\n").await
                // .map_err(|e| DatabaseError::from(e))
            }
            Frame::Null => {
                self.stream.write_u8(b'_').await
                // .map_err(|e| DatabaseError::from(e))
            }
            f => {
                error!(frame = %f, "unmatched frame");
                return Err(
                    FrameError::ProtocolError("Unexpected frame encountered".to_string()).into(),
                );
            }
        };

        self.stream.write_all(b"\r\n").await;
        Ok(())
    }

    async fn write_decimal(&mut self, value: u64) -> Result<(), io::Error> {
        use std::io::Write;

        let mut buf = [0u8; 8];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", value)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;

        Ok(())
    }
}
