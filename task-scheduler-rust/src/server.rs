use std::{convert::Infallible, net::SocketAddr, time::Duration};

use async_std::sync::Arc;
use futures::TryStreamExt;
use http::{Method, Request, Response, StatusCode, Version};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Server};
use tokio::signal;

use task_scheduler_rust::apply;
use task_scheduler_rust::echo;

#[derive(Debug)]
pub struct Handler {
    listener_port: u16,
    applier: Arc<apply::Applier>,
}

impl Handler {
    pub fn new(listener_port: u16, request_timeout: Duration) -> Self {
        println!(
            "creating handler with listener port {}, request timeout {:?}",
            listener_port, request_timeout,
        );
        Self {
            listener_port: listener_port,
            applier: Arc::new(apply::Applier::new(request_timeout)),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("starting server");
        match self.applier.start().await {
            Ok(_) => println!("started applier"),
            Err(e) => panic!("failed to stop applier {}", e),
        }

        let addr = ([0, 0, 0, 0], self.listener_port).into();
        let svc = make_service_fn(|socket: &AddrStream| {
            let remote_addr = socket.remote_addr();
            let applier = self.applier.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    handle_request(remote_addr, req, applier.clone())
                }))
            }
        });

        let server = Server::bind(&addr)
            .serve(svc)
            .with_graceful_shutdown(handle_sigint());

        println!("listener start http://{}", addr);
        if let Err(e) = server.await {
            println!("server error: {}", e);
        }
        println!("listener done http://{}", addr);

        match self.applier.stop().await {
            Ok(_) => println!("stopped applier"),
            Err(e) => println!("failed to stop applier {}", e),
        }

        Ok(())
    }
}

async fn handle_request(
    addr: SocketAddr,
    req: Request<Body>,
    applier: Arc<apply::Applier>,
) -> Result<Response<Body>, hyper::Error> {
    let http_version = req.version();
    let method = req.method().clone();
    let cloned_uri = req.uri().clone();
    let path = cloned_uri.path();
    println!(
        "version {:?}, method {}, uri path {}, remote addr {}",
        http_version, method, path, addr,
    );

    let resp = match http_version {
        Version::HTTP_11 => {
            match method {
                Method::POST => {
                    let mut resp = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("uninitialized response"))
                        .unwrap();
                    match req
                        .into_body()
                        .try_fold(Vec::new(), |mut data, chunk| async move {
                            data.extend_from_slice(&chunk);
                            Ok(data)
                        })
                        .await
                    {
                        Ok(body) => {
                            // no need to convert to string (e.g. "String::from_utf8(u)")
                            // just deserialize from bytes
                            println!("read request body {}", body.len());
                            let mut success = false;
                            let mut req = apply::Request::new();
                            match path {
                                "/echo" => match echo::parse_request(&body) {
                                    Ok(bb) => {
                                        req.echo_request = Some(bb);
                                        success = true;
                                    }
                                    Err(e) => {
                                        resp = Response::builder()
                                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                                            .body(Body::from(format!("failed to parse {}", e)))
                                            .unwrap();
                                    }
                                },
                                _ => {
                                    println!("unknown path {}", path);
                                    resp = Response::builder()
                                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                                        .body(Body::from(format!("unknown path {}", path)))
                                        .unwrap();
                                }
                            }
                            if success {
                                match applier.apply(req).await {
                                    Ok(rs) => resp = Response::new(Body::from(rs)),
                                    Err(e) => {
                                        resp = Response::builder()
                                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                                            .body(Body::from(format!(
                                                "failed to serde_json::from_str {}",
                                                e
                                            )))
                                            .unwrap();
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("failed to read request body {}", e);
                            resp = Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from(format!("failed to read request body {}", e)))
                                .unwrap()
                        }
                    }
                    resp
                }

                _ => Response::builder()
                    // https://github.com/hyperium/http/blob/master/src/status.rs
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(format!(
                        "unknown method {} and path {}",
                        method,
                        req.uri().path()
                    )))
                    .unwrap(),
            }
        }

        _ => Response::builder()
            .status(StatusCode::HTTP_VERSION_NOT_SUPPORTED)
            .body(Body::from(format!(
                "unknown HTTP version {:?}",
                http_version
            )))
            .unwrap(),
    };
    Ok(resp)
}

async fn handle_sigint() {
    signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
