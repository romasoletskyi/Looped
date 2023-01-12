use std::sync::{Arc, Mutex};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use core::data::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();
    let database = Arc::new(Mutex::new(Database::new()));

    let service = make_service_fn(move |_| {
        let database = database.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let database = database.clone();
                async move {
                    let mut response = Response::default();

                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/database") => {
                            response =
                                Response::new(Body::from(database.lock().unwrap().to_string()));
                            Ok::<_, hyper::Error>(response)
                        }
                        (&Method::POST, "/database") => {
                            let bytes = hyper::body::to_bytes(req.into_body()).await?;
                            let mut dat = database.lock().unwrap();

                            if let Some(got_database) = Database::from_slice(&bytes) {
                                dat.merge(got_database);
                            }

                            response = Response::new(Body::from(dat.to_string()));
                            Ok::<_, hyper::Error>(response)
                        }
                        _ => {
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            Ok::<_, hyper::Error>(response)
                        }
                    }
                }
            }))
        }
    });

    // TODO think on updating database on server/client sides
    // TODO Logging + dumping json to gitlab

    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}
