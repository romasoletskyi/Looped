use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::pin::Pin;
use std::{fs, io};

use futures_util::{ready, Future};
use futures_util::task::{Context, Poll};
use hyper::server::conn::{AddrStream, AddrIncoming};
use hyper::server::accept::Accept;
use hyper::header::{self, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use rustls::ServerConfig;
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

fn load_certs(filename: &str) -> io::Result<Vec<rustls::Certificate>> {
    let certfile = fs::File::open(filename)?;
    let mut reader = io::BufReader::new(certfile);

    let certs = rustls_pemfile::certs(&mut reader)?;
    Ok(certs
        .into_iter()
        .map(rustls::Certificate)
        .collect())
}

fn load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
    let keyfile = fs::File::open(filename)?;
    let mut reader = io::BufReader::new(keyfile);

    let keys = rustls_pemfile::rsa_private_keys(&mut reader)?;
    Ok(rustls::PrivateKey(keys[0].clone()))
}

enum State {
    Handshaking(tokio_rustls::Accept<AddrStream>),
    Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

pub struct TlsStream {
    state: State,
}

impl TlsStream {
    fn new(stream: AddrStream, config: Arc<ServerConfig>) -> TlsStream {
        let accept = tokio_rustls::TlsAcceptor::from(config).accept(stream);
        TlsStream {
            state: State::Handshaking(accept),
        }
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        match &self.state {
            State::Handshaking(stream) => stream.get_ref().map(|x| x.remote_addr()),
            State::Streaming(stream) => Some(stream.get_ref().0.remote_addr())
        }
    }
}

impl AsyncRead for TlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_read(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TlsStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_write(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

pub struct TlsAcceptor {
    config: Arc<ServerConfig>,
    incoming: AddrIncoming,
}

impl TlsAcceptor {
    pub fn new(config: Arc<ServerConfig>, incoming: AddrIncoming) -> TlsAcceptor {
        TlsAcceptor { config, incoming }
    }
}

impl Accept for TlsAcceptor {
    type Conn = TlsStream;
    type Error = io::Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<TlsStream, Self::Error>>> {
        let pin = self.get_mut();
        match ready!(Pin::new(&mut pin.incoming).poll_accept(cx)) {
            Some(Ok(sock)) => Poll::Ready(Some(Ok(TlsStream::new(sock, pin.config.clone())))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let addr = ([0, 0, 0, 0], 3000).into();
    let database = Arc::new(Mutex::new(Database::new()));

    let tls_cfg = {
        let certs = load_certs("certificate.crt")?;
        let key = load_private_key("private.key")?;

        let mut cfg = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
        cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Arc::new(cfg)
    };

    let service = make_service_fn(move |con: &TlsStream| {
        let address = con.remote_addr().map(|x| x.to_string()).unwrap_or_default();
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

    let server = Server::builder(TlsAcceptor::new(tls_cfg, AddrIncoming::bind(&addr)?)).serve(service);
    info!("Listening on https://{}", addr);

    server.await?;
    Ok(())
}
