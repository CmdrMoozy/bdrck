use anyhow::Result;
use std::pin::Pin;
pub use tokio::signal::unix::SignalKind;
use tokio::sync::oneshot;
use tokio_stream::wrappers::SignalStream;
use tokio_stream::{StreamExt, StreamMap};

async fn handle_signals(
    mut streams: StreamMap<SignalKind, Pin<Box<SignalStream>>>,
    mut shutdown_tx: Option<oneshot::Sender<()>>,
) {
    while let Some((_kind, _)) = streams.next().await {
        if let Some(tx) = shutdown_tx.take() {
            // Don't really care about failures, this is best effort.
            if cfg!(debug_assertions) {
                println!("graceful shutdown: caught {:?}", _kind);
            }
            if tx.send(()).is_err() && cfg!(debug_assertions) {
                println!("graceful shutdown: failed to send graceful shutdown signal");
            }
        }
    }
}

/// Handle graceful server shutdown. Spawns a task which listens for any of the given signals, and
/// upon receiving one, perform a graceful shutdown via sending on the given channel.
///
/// If sending on the given channel fails (e.g. because the receiver was already dropped), the
/// signal will be a no-op (under the assumption that the server is already shutting down /
/// finished shutting down, and no further action is needed).
pub fn handle_graceful_shutdown(
    signals: &[SignalKind],
    shutdown_tx: oneshot::Sender<()>,
) -> Result<()> {
    let mut streams: StreamMap<SignalKind, Pin<Box<SignalStream>>> = StreamMap::new();
    for kind in signals {
        streams.insert(
            *kind,
            Box::pin(SignalStream::new(tokio::signal::unix::signal(*kind)?)),
        );
    }

    tokio::spawn(handle_signals(streams, Some(shutdown_tx)));
    Ok(())
}
