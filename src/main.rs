pub(crate) mod error;

pub(crate) mod commands;
pub(crate) mod connection;
pub(crate) mod frame;
pub(crate) mod parse;

#[cfg(feature = "client")]
pub(crate) mod client;

#[cfg(feature = "server")]
pub(crate) mod server;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {}
