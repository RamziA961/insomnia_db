use std::vec;

use bytes::Bytes;
use thiserror::Error;

use crate::frame::Frame;

#[derive(Error, Debug)]
pub(crate) enum ParseError {
    #[error("End Of Stream")]
    EndOfStream,

    #[error("Protocl Error: {0}")]
    ProtocolError(String),
}

pub(crate) struct Parse {
    parts: vec::IntoIter<Frame>,
}

impl Parse {
    pub(crate) fn new(frame: Frame) -> Result<Self, ParseError> {
        match frame {
            Frame::Array(a) => Ok(Self {
                parts: a.into_iter(),
            }),
            frame => Err(ParseError::ProtocolError(format!(
                "Expected array frame. Found: {frame}"
            ))),
        }
    }

    pub(crate) fn next(&mut self) -> Result<Frame, ParseError> {
        self.parts.next().ok_or(ParseError::EndOfStream)
    }

    pub(crate) fn next_string(&mut self) -> Result<String, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(s),
            Frame::Bulk(bs) => std::str::from_utf8(&bs[..])
                .map(|s| s.to_string())
                .map_err(|_| ParseError::ProtocolError("Invalid bulk string.".to_string())),
            frame => Err(ParseError::ProtocolError(format!(
                "Expected simple frame or bulk frame. Found: {frame}"
            ))),
        }
    }

    pub(crate) fn next_bytes(&mut self) -> Result<Bytes, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
            Frame::Bulk(bs) => Ok(bs),
            frame => Err(ParseError::ProtocolError(format!(
                "Expected simple frame or bulk frame. Found: {frame}"
            ))),
        }
    }

    pub(crate) fn next_int(&mut self) -> Result<u64, ParseError> {
        let err_msg = "Invalid integer.";

        match self.next()? {
            Frame::Integer(i) => Ok(i),
            Frame::Simple(s) => atoi::atoi::<u64>(s.as_bytes())
                .ok_or_else(|| ParseError::ProtocolError(err_msg.to_string())),
            Frame::Bulk(bs) => {
                atoi::atoi::<u64>(&bs).ok_or_else(|| ParseError::ProtocolError(err_msg.to_string()))
            }
            frame => Err(ParseError::ProtocolError(format!(
                "Expected a integer, simple, or bulk frame containing an integer. Found: {frame}"
            ))),
        }
    }

    pub(crate) fn finish(&mut self) -> Result<(), ParseError> {
        if self.parts.next().is_none() {
            Ok(())
        } else {
            Err(ParseError::ProtocolError(
                "Expected end of frame. Stream is not empty.".to_string(),
            ))
        }
    }
}
