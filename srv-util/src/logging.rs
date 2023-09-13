use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::response::Response;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, OnceLock, Weak};
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tower_layer::Layer;
use tracing::{debug, info_span, Span};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

/// A guard that flushes logging events when dropped.
///
/// Depending on which logging implementation we're using, we may be vulnerable to situations
/// where, if the program exits "abruptly" (e.g. due to a panic or a std::process::exit), the last
/// few log events may not be written.
///
/// This guard provides a mechanism to ensure these last few logs are actually flushed. A reference
/// to this should be held e.g. in `main` or another "entrypoint".
pub struct WorkerGuard {
    _inner: Option<tracing_appender::non_blocking::WorkerGuard>,
}

static INIT: OnceLock<Option<Weak<WorkerGuard>>> = OnceLock::new();

fn build_env_filter(default_filter: &str) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| default_filter.into())
}

#[cfg(feature = "console-subscriber")]
fn init_logging_impl(default_filter: &str, logfile: Option<&Path>) -> Option<Arc<WorkerGuard>> {
    let r = tracing_subscriber::registry()
        .with(console_subscriber::spawn())
        .with(build_env_filter(default_filter));

    if cfg!(not(debug_assertions)) {
        if let Ok(jl) = tracing_journald::layer() {
            r.with(jl).init();
            return None;
        }

        if let Some(logfile) = logfile {
            if let Ok(lf) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(logfile)
            {
                let (al, g) = tracing_appender::non_blocking(lf);
                r.with(fmt::Layer::new().with_writer(al)).init();
                return Some(Arc::new(WorkerGuard { _inner: Some(g) }));
            }
        }
    }

    r.with(fmt::layer()).init();

    None
}

#[cfg(not(feature = "console-subscriber"))]
fn init_logging_impl(default_filter: &str, logfile: Option<&Path>) -> Option<Arc<WorkerGuard>> {
    let r = tracing_subscriber::registry().with(build_env_filter(default_filter));

    if cfg!(not(debug_assertions)) {
        if let Ok(jl) = tracing_journald::layer() {
            r.with(jl).init();
            return None;
        }

        if let Some(logfile) = logfile {
            if let Ok(lf) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(logfile)
            {
                let (al, g) = tracing_appender::non_blocking(lf);
                r.with(fmt::Layer::new().with_writer(al)).init();
                return Some(Arc::new(WorkerGuard { _inner: Some(g) }));
            }
        }
    }

    r.with(fmt::layer()).init();

    None
}

/// Initialize tracing-subscriber to capture log output.
///
/// A filter is configured using the RUST_LOG environment variable, or the given default filter if
/// the environment variable was invalid / unset.
///
/// If this is a debug build, log output is just written to stdout/stderr directly.
///
/// For release builds, we first attempt to send logging output to journald. If this fails (e.g.
/// because we're running on a non-systemd system), we fallback to writing to the given logfile (if
/// a path to use is provided). Failing both of those, we fallback to stdout/stderr again.
#[must_use]
pub fn init_logging(default_filter: &str, logfile: Option<&Path>) -> Option<Arc<WorkerGuard>> {
    let mut new_guard: Option<Arc<WorkerGuard>> = None;
    let maybe_guard = INIT
        .get_or_init(|| -> Option<Weak<WorkerGuard>> {
            init_logging_impl(default_filter, logfile).map(|guard| {
                let weak = Arc::downgrade(&guard);
                new_guard = Some(guard);
                weak
            })
        })
        .clone();

    // If we just created a new guard, return it directly. If init_logging was called previously,
    // try to upgrade our weak reference to the previously-created guard. If all previous caller(s)
    // already dropped their references, too bad.
    new_guard.or(maybe_guard.map(|weak| weak.upgrade()).flatten())
}

/// Constructs a new Layer which can be added to an axum Router to add request logging.
pub fn new_logging_layer<S, Req, Res>() -> impl Layer<S> {
    TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Req>| {
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
        .on_response(
            |response: &Response<Res>, latency: Duration, _span: &Span| {
                debug!(
                    "response '{}' generated in {:?}",
                    response.status(),
                    latency
                )
            },
        )
}
