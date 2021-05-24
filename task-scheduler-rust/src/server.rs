use futures::{TryFutureExt, TryStreamExt};
use http::{Method, Request, Response, StatusCode};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Server};
use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;

use task_scheduler_rust::apply;
use task_scheduler_rust::echo;

pub struct Handler {
    listener_port: u16,
    request_timeout: Duration,
}

impl Handler {
    pub fn new(listener_port: u16, request_timeout: Duration) -> Self {
        println!(
            "creating handler with listener port {}, request timeout {:?}",
            listener_port, request_timeout,
        );

        Self {
            listener_port,
            request_timeout,
        }
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        println!("starting server");

        let (applier, applier_handle) = apply::Applier::new(self.request_timeout);
        println!("started applier");
        let applier = Arc::new(applier);

        let addr = ([0, 0, 0, 0], self.listener_port).into();
        let svc = make_service_fn(|socket: &AddrStream| {
            let remote_addr = socket.remote_addr();
            let applier = applier.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    handle_request(remote_addr, req, applier.clone()).or_else(
                        |(status, body)| async move {
                            println!("{}", body);
                            Ok::<_, Infallible>(
                                Response::builder()
                                    .status(status)
                                    .body(Body::from(body))
                                    .unwrap(),
                            )
                        },
                    )
                }))
            }
        });

        let server = Server::try_bind(&addr)?
            .serve(svc)
            .with_graceful_shutdown(handle_sigint());

        println!("listener start http://{}", addr);
        if let Err(e) = server.await {
            println!("server error: {}", e);
        }
        println!("listener done http://{}", addr);

        match applier.stop().await {
            Ok(_) => println!("stopped applier"),
            Err(e) => println!("failed to stop applier {}", e),
        }

        applier_handle.await??;

        Ok(())
    }
}

async fn handle_request(
    addr: SocketAddr,
    req: Request<Body>,
    applier: Arc<apply::Applier>,
) -> Result<Response<Body>, (http::StatusCode, String)> {
    let http_version = req.version();
    let method = req.method().clone();
    let cloned_uri = req.uri().clone();
    let path = cloned_uri.path();
    println!(
        "version {:?}, method {}, uri path {}, remote addr {}",
        http_version, method, path, addr,
    );

    let resp = match method {
        Method::POST => {
            let body = req
                .into_body()
                .try_fold(Vec::new(), |mut data, chunk| async move {
                    data.extend_from_slice(&chunk);
                    Ok(data)
                })
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("failed to read request body {}", e),
                    )
                })?;
            // no need to convert to string (e.g. "String::from_utf8(u)")
            // just deserialize from bytes
            println!("read request body {}", body.len());
            let req = match path {
                "/echo" => {
                    let bb = echo::parse_request(&body).map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("failed to parse {}", e),
                        )
                    })?;
                    let mut req = apply::Request::new();
                    req.echo_request = Some(bb);
                    req
                }
                _ => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unknown path {}", path),
                ))?,
            };
            let rs = applier.apply(req).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed to serde_json::from_str {}", e),
                )
            })?;
            Response::new(Body::from(rs))
        }

        _ => Err((
            StatusCode::NOT_FOUND,
            format!("unknown method {} and path {}", method, req.uri().path()),
        ))?,
    };

    Ok(resp)
}

async fn handle_sigint() {
    signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
