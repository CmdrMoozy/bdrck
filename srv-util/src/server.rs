use crate::error::*;

pub use tokio::signal::unix::SignalKind;

use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::response::Response;
use axum::Router;
use hyper::body::Incoming;
use libc::c_int;
use std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::watch;
use tokio_stream::wrappers::SignalStream;
use tokio_stream::{StreamExt, StreamMap};
use tower::Service;
use tower_http::trace::TraceLayer;
use tracing::{debug, info_span, Span};

async fn handle_signals(
    mut streams: StreamMap<SignalKind, Pin<Box<SignalStream>>>,
    mut shutdown_tx: Option<watch::Sender<c_int>>,
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

fn handle_graceful_shutdown(signals: &[SignalKind]) -> Result<Option<watch::Receiver<c_int>>> {
    if signals.is_empty() {
        return Ok(None);
    }

    let (tx, rx) = watch::channel::<c_int>(0);

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

fn add_logging_layer(app: Router<()>) -> Router<()> {
    app.layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                let remote_addr: Option<SocketAddr> = request
                    .extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ci| ci.0);
                let real_remote_addr = request
                    .headers()
                    .get("x-forwarded-for")
                    .or_else(|| request.headers().get("x-real-ip"))
                    .map(|v| v.to_str())
                    .transpose();
                let referer = request
                    .headers()
                    .get(axum::http::header::REFERER)
                    .map(|v| v.to_str())
                    .transpose();
                let user_agent = request
                    .headers()
                    .get(axum::http::header::USER_AGENT)
                    .map(|v| v.to_str())
                    .transpose();

                info_span!(
                "http-request",
                remote_addr = ?remote_addr,
                real_remote_addr = ?real_remote_addr,
                method = ?request.method(),
                uri = ?request.uri(),
                version = ?request.version(),
                referer = ?referer,
                user_agent = ?user_agent)
            })
            .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                debug!(
                    "response '{}' generated in {:?}",
                    response.status(),
                    latency
                )
            }),
    )
}

pub enum GracefulShutdownKind {
    Signal(Vec<SignalKind>),
    Custom(watch::Receiver<c_int>),
    None,
}

impl GracefulShutdownKind {
    fn get_signals(&self) -> &[SignalKind] {
        match self {
            GracefulShutdownKind::Signal(signals) => signals,
            _ => &[],
        }
    }
}

async fn rx_graceful_shutdown(rx: Option<watch::Receiver<c_int>>) {
    if let Some(mut rx) = rx {
        if let Err(e) = rx.changed().await {
            if cfg!(debug_assertions) {
                println!("graceful shutdown: signal sender dropped: {:?}", e);
            }
        }

        let signal: c_int = *rx.borrow_and_update();
        if cfg!(debug_assertions) {
            println!("graceful shutdown: signal receiver got value {}", signal);
        }
    } else {
        let () = std::future::pending().await;
        unreachable!();
    }
}

pub async fn serve_with(
    listener: TcpListener,
    shutdown_kind: GracefulShutdownKind,
    should_add_logging_layer: bool,
    app: Router<()>,
) -> Result<()> {
    let listener = tokio::net::TcpListener::from_std(listener)?;

    let shutdown_rx =
        handle_graceful_shutdown(shutdown_kind.get_signals())?.or(match shutdown_kind {
            GracefulShutdownKind::Custom(rx) => Some(rx),
            _ => None,
        });

    let app = if should_add_logging_layer {
        add_logging_layer(app)
    } else {
        app
    };

    // See: https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs

    // Channel to track connection handling tasks and wait for them to complete.
    let (close_tx, close_rx) = watch::channel(());

    loop {
        let (socket, remote_addr) = tokio::select! {
            // Either accept a new connection...
            result = listener.accept() => {
                result.unwrap()
            }
            // ...or stop looping if we got a graceful shutdown signal.
            _ = rx_graceful_shutdown(shutdown_rx.clone()) => {
                break;
            }
        };

        debug!("connection {} accepted", remote_addr);

        // Spawn a task to handle the connection.
        let tower_service = app.clone();
        let close_rx = close_rx.clone();
        let shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            // Boilerplate to fit our round peg in hyper's square hole.
            let socket = hyper_util::rt::TokioIo::new(socket);
            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            // Due to deficiencies in the hyper API we have to pick http1 or http2 explicitly.
            // Since a typical configuration is we're behind nginx anyway, just pick http1 for
            // simplicity.
            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(socket, hyper_service)
                .with_upgrades();

            // `graceful_shutdown` requires a pinned connection.
            let mut conn = std::pin::pin!(conn);

            loop {
                tokio::select! {
                    result = conn.as_mut() => {
                        if let Err(e) = result {
                            debug!("failed to serve connection from {}: {:?}", remote_addr, e);
                        }
                        break;
                    }
                    _ = rx_graceful_shutdown(shutdown_rx.clone()) => {
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }

            // Drop our side of the channel to notify the caller that our task is done.
            drop(close_rx);
        });
    }

    // We only care about watch receivers moved into connection tasks so close this residual one.
    drop(close_rx);
    // Stop accepting new connections.
    drop(listener);

    // Wait for all inflight tasks to complete.
    debug!(
        "waiting for {} connection tasks to finish",
        close_tx.receiver_count()
    );
    close_tx.closed().await;

    Ok(())
}

pub async fn serve<A: ToSocketAddrs>(
    addr: A,
    shutdown_kind: GracefulShutdownKind,
    should_add_logging_layer: bool,
    app: Router<()>,
) -> Result<()> {
    serve_with(
        TcpListener::bind(addr)?,
        shutdown_kind,
        should_add_logging_layer,
        app,
    )
    .await
}
