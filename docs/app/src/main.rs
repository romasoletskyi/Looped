use std::sync::{Arc, Mutex};

use hyper::server::conn::AddrStream;
use hyper::header::{self, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{info, warn};

use core::database::Database;

fn enable_cors(response: &mut Response<Body>) {
    let headers = response.headers_mut();

    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
    headers.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static("300"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let addr = ([0, 0, 0, 0], 3000).into();
    let database = Arc::new(Mutex::new(Database::new()));

    let service = make_service_fn(move |con: &AddrStream| {
        let address = con.remote_addr().to_string();
        let database = database.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let address = address.clone();
                let database = database.clone();
                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/database") => {
                            let mut dat = database.lock().unwrap();
                            let mut response = Response::new(Body::from(dat.total_clone().to_string()));
                            response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

                            dat.updated(&address);
                            info!("sent database to {}", address);

                            enable_cors(&mut response);
                            Ok::<_, hyper::Error>(response)
                        }
                        (&Method::POST, "/database") => {
                            let bytes = hyper::body::to_bytes(req.into_body()).await?;
                            let mut dat = database.lock().unwrap();
                            let difference = dat.difference(&address);

                            if let Some(got_database) = Database::from_slice(&bytes) {
                                dat.merge(got_database);
                            } else {
                                warn!("database difference from {} wasn't merged", address);
                            }

                            let mut response = Response::new(Body::from(difference.to_string()));
                            response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
                            dat.updated(&address);
                            info!("updated database at {}", address);
                            
                            enable_cors(&mut response);
                            Ok::<_, hyper::Error>(response)
                        }
                        (&Method::OPTIONS, _) => {
                            let mut response = Response::default();
                            enable_cors(&mut response);
                            Ok::<_, hyper::Error>(response)
                        }
                        _ => {
                            warn!("invalid request from {} with path {}", address, req.uri().path());

                            let mut response = Response::default();
                            *response.status_mut() = StatusCode::NOT_FOUND;

                            Ok::<_, hyper::Error>(response)
                        }
                    }
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);
    info!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}
