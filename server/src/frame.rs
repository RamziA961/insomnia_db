use std::{
    fmt::{Debug, Display},
    io::Cursor,
};

use bytes::{Buf, Bytes};
use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("Type Mismatch Error: {0}")]
    TypeMismatch(String),

    #[error("Parsing Error: {0}")]
    ParsingError(String),

    #[error("Protocol Error: {0}")]
    ProtocolError(String),
}

#[derive(Debug, Clone)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    pub(crate) fn push_bulk(&mut self, bytes: Bytes) -> Result<(), FrameError> {
        match self {
            Frame::Array(v) => {
                v.push(Frame::Bulk(bytes));
                Ok(())
            }
            _ => Err(FrameError::TypeMismatch(
                "Frame is not an Array.".to_string(),
            )),
        }
    }

    pub(crate) fn push_int(&mut self, value: u64) -> Result<(), FrameError> {
        match self {
            Frame::Array(v) => {
                v.push(Frame::Integer(value));
                Ok(())
            }
            _ => Err(FrameError::TypeMismatch(
                "Frame is not an Array.".to_string(),
            )),
        }
    }

    pub(crate) fn validate(cursor: &mut Cursor<&[u8]>) -> Result<(), FrameError> {
        match get_next(cursor)? {
            b'+' => {
                // simple string
                get_line(cursor)?;
                Ok(())
            }
            b'-' => {
                // simple error
                get_line(cursor)?.to_vec();
                Ok(())
            }
            b':' => {
                // integer
                get_decimal(cursor)?;
                Ok(())
            }
            b'_' => {
                // null (RESP3 encoding)
                let l = get_line(cursor)?;
                if l == b"_\r\n" {
                    Ok(())
                } else {
                    Err(FrameError::ProtocolError(format!(
                        "Invalid frame type. Bytes: {l:?}"
                    )))
                }
            }
            b'$' => {
                //bulk string
                let len = get_decimal(cursor)?;
                advance(cursor, (len as usize) + 2)
            }
            b'*' => {
                // simple array
                let len = get_decimal(cursor)?;

                for _ in 0..(len as usize) {
                    Frame::validate(cursor)?;
                }
                Ok(())
            }
            unsupported => Err(FrameError::ProtocolError(format!(
                "Invalid frame type. Byte: {unsupported:?}"
            ))),
        }
    }

    pub(crate) fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
        match get_next(cursor)? {
            b'+' => {
                let l = get_line(cursor)?.to_vec();
                String::from_utf8(l)
                    .map_err(|e| {
                        warn!(error = %e, "converting bytes to string failed");
                        FrameError::ParsingError(format!(
                            "Byte to string coversion failed. Bytes: {l:?}"
                        ))
                    })
                    .map(|v| Frame::Simple(v))
            }
            b'-' => {
                let l = get_line(cursor)?.to_vec();
                String::from_utf8(l)
                    .map_err(|e| {
                        warn!(error = %e, "converting bytes to string failed");
                        FrameError::ParsingError(format!(
                            "Byte to string coversion failed. Bytes: {l:?}"
                        ))
                    })
                    .map(|v| Frame::Error(v))
            }
            b':' => {
                let d = get_decimal(cursor)?;
                Ok(Frame::Integer(d))
            }
            b'_' => {
                let l = get_line(cursor)?;
                if l == b"_\r\n" {
                    advance(cursor, 3)?;
                    Ok(Frame::Null)
                } else {
                    Err(FrameError::ProtocolError(format!(
                        "Invalid frame type. Bytes: {l:?}"
                    )))
                }
            }
            b'$' => {
                // read bulk string
                let len = get_decimal(cursor)? as usize;
                let n = len + 2;

                if cursor.remaining() < n {
                    Err(FrameError::ParsingError(
                        "Bulk string length exceeds buffer".to_string(),
                    ))
                } else if cursor.chunk()[len..n] != b"\r\n"[..] {
                    Err(FrameError::ParsingError("Invalid terminator.".to_string()))
                } else {
                    let bulk = Bytes::copy_from_slice(&cursor.chunk()[..len]);
                    advance(cursor, n)?;
                    Ok(Frame::Bulk(bulk))
                }
            }
            b'*' => {
                let len = get_decimal(cursor)?;
                let mut v = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    v.push(Frame::parse(cursor)?);
                }

                Ok(Frame::Array(v))
            }
            unsupported => Err(FrameError::ProtocolError(format!(
                "Invalid frame type. Byte {}",
                unsupported
            ))),
        }
    }
}

impl PartialEq<&str> for Frame {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Frame::Simple(v) => v.eq(other),
            Frame::Bulk(v) => v.eq(other),
            Frame::Error(v) => v.eq(other),
            _ => false,
        }
    }
}

impl PartialEq<u64> for Frame {
    fn eq(&self, other: &u64) -> bool {
        match self {
            Frame::Integer(v) => *v == *other,
            _ => false,
        }
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Frame::Simple(s) => Display::fmt(s, f),
            Frame::Error(e) => Display::fmt(e, f),
            Frame::Integer(i) => Display::fmt(i, f),
            Frame::Bulk(b) => match std::str::from_utf8(b) {
                Ok(s) => Display::fmt(&s, f),
                Err(e) => {
                    warn!(error = %e, "bulk string to utf8 failed");
                    write!(f, "{:?}", e)
                }
            },
            Frame::Null => Display::fmt("(null)", f),
            Frame::Array(v) => v.iter().try_for_each(|frame| {
                Display::fmt(frame, f)?;
                write!(f, " ")
            }),
            // For future data types
            _ => todo!(),
        }
    }
}

fn advance(source: &mut Cursor<&[u8]>, n: usize) -> Result<(), FrameError> {
    if source.remaining() < n {
        Err(FrameError::ParsingError(format!(
            "Advance failed. Advance of {n} bytes while {} bytes remaining.",
            source.remaining()
        )))
    } else {
        source.advance(n);
        Ok(())
    }
}

fn peek_next(source: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !source.has_remaining() {
        Err(FrameError::ParsingError(
            "Peek failed. Buffer exhausted.".to_string(),
        ))
    } else {
        Ok(source.chunk()[0])
    }
}

fn get_next(source: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !source.has_remaining() {
        Err(FrameError::ParsingError(
            "Get failed. Buffer exhausted.".to_string(),
        ))
    } else {
        Ok(source.get_u8())
    }
}

fn get_line<'a>(source: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    let start = source.position() as usize;
    let end = source.get_ref().len() - 1;

    for i in start..end {
        if source.get_ref()[i] == b'\r' && source.get_ref()[i + 1] == b'\n' {
            source.set_position((i + 2) as u64);
            return Ok(&source.get_ref()[start..i]);
        }
    }

    Err(FrameError::ParsingError(
        "Get line failed. Buffer exhausted.".to_string(),
    ))
}

fn get_decimal(source: &mut Cursor<&[u8]>) -> Result<u64, FrameError> {
    let mut l = get_line(source)?;

    if l.len() > 8 {
        return Err(FrameError::ProtocolError(
            "Invalid frame format".to_string(),
        ));
    }

    let l = if l.len() < 8 {
        (0u8..(8 - l.len() as u8))
            .into_iter()
            .chain(l.iter().map(|v| v.clone()))
            .collect::<Vec<_>>()
            .as_slice()
    } else {
        l
    };

    let mut array = [0; 8];
    array.copy_from_slice(l);
    Ok(u64::from_be_bytes(array))
}
