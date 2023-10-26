pub(crate) mod error;

pub(crate) mod connection;
pub(crate) mod database;
pub(crate) mod frame;
pub(crate) mod jobs;
pub(crate) mod parse;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {}

// #[tokio::main]
// #[allow(unused_results)]
// async fn main() -> Result<(), DatabaseError<dyn std::error::Error>> {
//     use tracing_subscriber::prelude::*;
//     let _ = tracing_subscriber::Registry::default()
//         .with(
//             tracing_subscriber::fmt::layer()
//                 .pretty()
//                 .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
//         )
//         .init();
//
//     let ipv4 = "0.0.0.0:6379";
//     let listener = TcpListener::bind(ipv4).await.map_err(|e| {
//         error!(error = %e, "unable to bind to port {ipv4}");
//         DatabaseError::StartupError {
//             cause: e.to_string(),
//         }
//     })?;
//
//     info!("Listening on {ipv4}");
//
//     let pool = tokio::runtime::Builder::new_multi_thread()
//         .thread_keep_alive(Duration::from_secs(30))
//         .on_thread_start(|| {
//             let c = std::thread::current();
//             info!("thread {:?} responding", c.id());
//         })
//         .enable_io()
//         .build()
//         .map_err(|e| {
//             error!(error = %e, "unable to initalize thread pool");
//             return DatabaseError::StartupError {
//                 cause: e.to_string(),
//             };
//         })?;
//
//     loop {
//         let res = listener.accept().await.map_err(|e| {
//             warn!(error = %e, "unable to accept client");
//             DatabaseError::ConnectionError {
//                 cause: e.to_string(),
//             }
//         });
//
//         if res.is_err() {
//             continue;
//         }
//
//         let (mut socket, addr) = res.unwrap();
//
//         pool.spawn(async move {
//             let mut buf = vec![0; 4096];
//
//             loop {
//                 let read_res = socket.read(&mut buf).await.map_err(|e| {
//                     error!(error = %e, "reading from socket failed");
//                     DatabaseError::ConnectionError {
//                         cause: e.to_string(),
//                     }
//                 });
//
//                 if let Err(e) = read_res {
//                     // should respond
//                     warn!("handling read error");
//                     let encoded = e.encode::<String, String>();
//
//                     if let Ok(enc_err) = encoded {
//                         info!(encoded=%enc_err);
//                         let _ = socket
//                             .write(enc_err.as_bytes())
//                             .await
//                             .map_err(|e| error!(error = %e, "writing error to socket failed"));
//                     } else {
//                         error!(error = %e, "serialzing error failed");
//                     }
//
//                     continue;
//                 }
//
//                 info!("writing to socket");
//                 let bytes_read = read_res.unwrap();
//                 let out = socket
//                     .write({
//                         let enc = serialization::SimpleString(
//                             buf.iter()
//                                 .take(bytes_read)
//                                 .map(|b| b.clone() as char)
//                                 .collect::<String>(),
//                         );
//
//                         enc.encode::<String, String>()
//                             .map(|v| {
//                                 info!("encoded: {:?}", v);
//                                 v
//                             })
//                             .map_err(|e| error!(error = %e))
//                             .unwrap()
//                             .as_bytes()
//                     })
//                     .await
//                     .map(|v| {
//                         info!("{v:?}");
//                         v
//                     })
//                     .map_err(|e| async move {
//                         error!(error = %e, "writing error to socket failed");
//                     });
//
//                 if out.is_err() {
//                     break;
//                 }
//             }
//         });
//     }
// }
