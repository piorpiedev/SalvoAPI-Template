use std::fs;

use salvo::conn::rustls::{Keycert, RustlsConfig};
use salvo::prelude::*;
use salvo::server::ServerHandle;
use serde::Serialize;

mod config;
mod hoops;
mod routers;

mod error;
pub use error::AppError;

pub type AppResult<T> = Result<T, AppError>;
pub type JsonResult<T> = Result<Json<T>, AppError>;
pub type EmptyResult = Result<Json<Empty>, AppError>;

pub fn json_ok<T>(data: T) -> JsonResult<T> {
    Ok(Json(data))
}
#[derive(Serialize, ToSchema, Clone, Copy, Debug)]
pub struct Empty {}
pub fn empty_ok() -> JsonResult<Empty> {
    Ok(Json(Empty {}))
}

#[tokio::main]
async fn main() {
    crate::config::init();
    let config: &config::ServerConfig = crate::config::get();

    let _guard = config.log.guard();
    tracing::info!("log level: {}", &config.log.filter_level);

    let service = Service::new(routers::root());
    println!("📖 Scalar running!");
    println!("🔄 Listen on {}", &config.listen_addr);

    // Acme support, automatically get TLS certificate from Let's Encrypt. For example, see https://github.com/salvo-rs/salvo/blob/main/examples/acme-http01-quinn/src/main.rs
    if config.tls.enabled {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");

        let cert = fs::read_to_string(&config.tls.cert_path).expect("unable to read tls cert file");
        let key = fs::read_to_string(&config.tls.key_path).expect("unable to read tls key file");
        let listen_addr = &config.listen_addr;
        let rustls_config = RustlsConfig::new(Keycert::new().cert(cert.clone()).key(key.clone()));
        let listener = TcpListener::new(listen_addr).rustls(rustls_config.clone());
        let acceptor = QuinnListener::new(rustls_config.build_quinn_config().unwrap(), listen_addr)
            .join(listener)
            .bind()
            .await;
        let server = Server::new(acceptor);
        hook_stop(server.handle());
        server.serve(service).await;
    } else {
        let acceptor = TcpListener::new(&config.listen_addr).bind().await;
        let server = Server::new(acceptor);
        hook_stop(server.handle());
        server.serve(service).await;
    }
}

fn hook_stop(handle: ServerHandle) {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        handle.stop_graceful(std::time::Duration::from_secs(60));
    });
}
