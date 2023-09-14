use crate::error::*;

pub use tokio::signal::unix::SignalKind;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::response::Response;
use axum::{Router, Server};
use libc::c_int;
use std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio_stream::wrappers::SignalStream;
use tokio_stream::{StreamExt, StreamMap};
use tower_http::trace::TraceLayer;
use tracing::{debug, info_span, Span};

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

    let (tx, rx) = oneshot::channel::<c_int>();

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

fn add_logging_layer(app: Router<(), Body>) -> Router<(), Body> {
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
    Custom(oneshot::Receiver<()>),
    None,
}

impl GracefulShutdownKind {
    fn is_none(&self) -> bool {
        matches!(self, GracefulShutdownKind::None)
    }

    fn get_signals(&self) -> &[SignalKind] {
        match self {
            GracefulShutdownKind::Signal(signals) => signals,
            _ => &[],
        }
    }
}

pub async fn serve_with(
    listener: TcpListener,
    shutdown_kind: GracefulShutdownKind,
    should_add_logging_layer: bool,
    app: Router<(), Body>,
) -> Result<()> {
    let signal_rx = handle_graceful_shutdown(shutdown_kind.get_signals())?;

    let app = if should_add_logging_layer {
        add_logging_layer(app)
    } else {
        app
    };

    let server =
        Server::from_tcp(listener)?.serve(app.into_make_service_with_connect_info::<SocketAddr>());

    if !shutdown_kind.is_none() {
        server
            .with_graceful_shutdown(async {
                if let Some(signal_rx) = signal_rx {
                    let signal = signal_rx.await;
                    if cfg!(debug_assertions) {
                        match signal {
                            Err(e) => {
                                println!("graceful shutdown: signal handler dropped: {:?}", e)
                            }
                            Ok(signal) => {
                                println!("graceful shutdown: server received signal {}", signal)
                            }
                        }
                    }
                } else if let GracefulShutdownKind::Custom(rx) = shutdown_kind {
                    if let Err(e) = rx.await {
                        if cfg!(debug_assertions) {
                            println!("graceful shutdown: custom channel dropped: {:?}", e);
                        }
                    }
                }
            })
            .await?;
    } else {
        server.await?;
    }

    Ok(())
}

pub async fn serve<A: ToSocketAddrs>(
    addr: A,
    shutdown_kind: GracefulShutdownKind,
    should_add_logging_layer: bool,
    app: Router<(), Body>,
) -> Result<()> {
    serve_with(
        TcpListener::bind(addr)?,
        shutdown_kind,
        should_add_logging_layer,
        app,
    )
    .await
}
