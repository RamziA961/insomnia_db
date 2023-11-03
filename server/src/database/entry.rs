use bytes::Bytes;
use thiserror::Error;
use tokio::time::Instant;

#[derive(Error, Debug)]
#[error("[BuilderError] {0}")]
pub(crate) struct BuilderError(#[from] anyhow::Error);

#[derive(Debug)]
pub(crate) struct Entry {
    pub(super) buf: Bytes,
    pub(super) expiration: Option<Instant>,
}

pub(crate) struct Builder {
    buf: Option<Bytes>,
    expiration: Option<Instant>,
}

impl Entry {
    pub(crate) fn has_expired(&self) -> bool {
        match self.expiration {
            Some(expiry) => expiry < Instant::now(),
            None => false,
        }
    }

    pub(crate) fn builder() -> Builder {
        Builder::default()
    }
}

impl Builder {
    pub(crate) fn new() -> Self {
        Self {
            buf: None,
            expiration: None,
        }
    }

    pub(crate) fn with_bytes(mut self, buffer: Bytes) -> Self {
        self.buf = Some(buffer);
        self
    }

    pub(crate) fn with_expiration(mut self, expiry: Option<Instant>) -> Self {
        self.expiration = expiry;
        self
    }

    pub(crate) fn build(self) -> Result<Entry, BuilderError> {
        if self.buf.is_none() {
            return Err(anyhow::anyhow!("Buffer is empty").into());
        }

        Ok(Entry {
            buf: self.buf.clone().unwrap(),
            expiration: self.expiration.clone(),
        })
    }

    pub(crate) fn build_consume(mut self) -> Result<Entry, BuilderError> {
        if self.buf.is_none() {
            return Err(anyhow::anyhow!("Buffer is empty").into());
        }

        Ok(Entry {
            buf: self.buf.unwrap(),
            expiration: self.expiration,
        })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            buf: None,
            expiration: None,
        }
    }
}
