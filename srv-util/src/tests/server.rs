use crate::logging::init_logging;
use crate::server::*;
use anyhow::{bail, Result};
use axum::routing;
use axum::Router;
use libc::c_int;
use reqwest::{Client, Url};
use std::future::Future;
use std::net::{SocketAddr, TcpListener};
use tokio::sync::watch;
use tokio::task::JoinHandle;

struct TestServer {
    url: Url,
    shutdown_tx: watch::Sender<c_int>,
    handle: JoinHandle<Result<()>>,
}

impl TestServer {
    fn new() -> Result<Self> {
        let addr = "127.0.0.1:0".parse::<SocketAddr>()?;
        let listener = TcpListener::bind(addr)?;
        let url: Url = format!("http://{}/", listener.local_addr()?).parse()?;

        let (tx, rx) = watch::channel(0);

        Ok(TestServer {
            url: url,
            shutdown_tx: tx,
            handle: tokio::spawn(async move {
                let app = Router::new().route("/", routing::get(|| async { "Hello, World!" }));
                serve_with(
                    listener,
                    GracefulShutdownKind::Custom(rx),
                    /*should_add_logging_layer=*/ true,
                    app,
                )
                .await?;

                Ok(())
            }),
        })
    }
}

async fn do_test<Fut: Future<Output = Result<()>>, F: FnOnce(Url) -> Fut>(f: F) -> Result<()> {
    let _guard = init_logging("debug,tower_http=debug,axum::rejection=trace", None);

    let server = TestServer::new()?;

    let res = f(server.url.clone()).await;
    if server.shutdown_tx.send(9).is_err() {
        bail!("graceful shutdown recevier dropped before signalling");
    }
    server.handle.await??;
    res
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_hello_world() -> Result<()> {
    async fn test_impl(server_url: Url) -> Result<()> {
        let client = Client::new();
        tracing::debug!("sending test request...");
        let res = client.get(server_url).send().await?;
        tracing::debug!("done sending test request...");
        if !res.status().is_success() {
            bail!("GET hello world server failed: {:?}", res.status());
        }
        let text = res.text().await?;
        if text != "Hello, World!" {
            bail!("expected 'Hello, World!' response, got: '{}'", text);
        }
        Ok(())
    }

    do_test(test_impl).await
}
