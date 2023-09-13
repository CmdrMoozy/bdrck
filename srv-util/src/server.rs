use crate::error::*;

pub use tokio::signal::unix::SignalKind;

use axum::body::Body;
use axum::{Router, Server};
use libc::c_int;
use std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use std::pin::Pin;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::SignalStream;
use tokio_stream::{StreamExt, StreamMap};

async fn handle_signals(
    mut streams: StreamMap<SignalKind, Pin<Box<SignalStream>>>,
    mut shutdown_tx: Option<oneshot::Sender<c_int>>,
) {
    while let Some((kind, _)) = streams.next().await {
        if let Some(tx) = shutdown_tx.take() {
            // Don't really care about failures, this is best effort.
            if cfg!(debug_assertions) {
                println!("graceful shutdown: caught {:?}", kind);
            }
            if let Err(e) = tx.send(kind.as_raw_value()) {
                if cfg!(debug_assertions) {
                    println!(
                        "graceful shutdown: failed to send graceful shutdown signal: {:?}",
                        e
                    );
                }
            }
        }
    }
}

fn handle_graceful_shutdown(signals: &[SignalKind]) -> Result<Option<oneshot::Receiver<c_int>>> {
    if signals.is_empty() {
        return Ok(None);
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<c_int>();

    let mut streams: StreamMap<SignalKind, Pin<Box<SignalStream>>> = StreamMap::new();
    for kind in signals {
        streams.insert(
            *kind,
            Box::pin(SignalStream::new(tokio::signal::unix::signal(*kind)?)),
        );
    }

    tokio::spawn(handle_signals(streams, Some(tx)));
    Ok(Some(rx))
}

async fn serve_impl(
    listener: TcpListener,
    shutdown_rx: Option<oneshot::Receiver<c_int>>,
    app: Router<(), Body>,
) -> Result<()> {
    let server =
        Server::from_tcp(listener)?.serve(app.into_make_service_with_connect_info::<SocketAddr>());

    Ok(if let Some(rx) = shutdown_rx {
        server
            .with_graceful_shutdown(async {
                let signal = rx.await;
                if cfg!(debug_assertions) {
                    match signal {
                        Err(e) => println!("graceful shutdown: signal handler dropped: {:?}", e),
                        Ok(signal) => {
                            println!("graceful shutdown: server received signal {}", signal)
                        }
                    }
                }
            })
            .await?;
    } else {
        server.await?;
    })
}

pub async fn serve<A: ToSocketAddrs>(
    addr: A,
    shutdown_signals: &[SignalKind],
    app: Router<(), Body>,
) -> Result<JoinHandle<Result<()>>> {
    let listener = TcpListener::bind(addr)?;
    let shutdown_rx = handle_graceful_shutdown(shutdown_signals)?;

    Ok(tokio::spawn(serve_impl(listener, shutdown_rx, app)))
}
