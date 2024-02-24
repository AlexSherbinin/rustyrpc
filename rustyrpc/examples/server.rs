mod common;

use std::sync::Arc;

use common::{
    auth_service::{AuthService, AuthServiceWrapper},
    hello_service::{HelloService, HelloServiceWrapper},
};
use quinn::ServerConfig;
use rustyrpc::{
    format::{rkyv::RkyvFormat, Encode, EncodingFormat},
    server::{Server, ServerBuilder, ServiceRef},
    service::IntoService,
    transport,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let server_config = generate_server_config();
    let listener =
        transport::quic::ConnectionListener::new(server_config, "127.0.0.1:8888".parse().unwrap())
            .unwrap();

    let server: Arc<Server<_, RkyvFormat>> = ServerBuilder::default()
        .with_service(AuthServiceImpl)
        .build(listener)
        .into();

    server.listen().await;
}

fn generate_server_config() -> ServerConfig {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_chain = vec![rustls::Certificate(cert.serialize_der().unwrap())];

    let private_key = rustls::PrivateKey(cert.serialize_private_key_der());

    ServerConfig::with_single_cert(cert_chain, private_key).unwrap()
}

struct AuthServiceImpl;

impl<Format: EncodingFormat> AuthService<Format> for AuthServiceImpl
where
    Option<ServiceRef>: Encode<Format>,
{
    async fn auth(
        &self,
        username: &str,
        password: &str,
    ) -> Option<Box<dyn rustyrpc::service::Service<Format>>> {
        const USERNAME: &str = "admin";
        const PASSWORD: &str = "admin";

        if username == USERNAME && password == PASSWORD {
            Some(Box::new(HelloServiceImpl.into_service()))
        } else {
            None
        }
    }
}

impl<Format: EncodingFormat> IntoService<Format> for AuthServiceImpl
where
    Option<ServiceRef>: Encode<Format>,
{
    type Wrapper = AuthServiceWrapper<Self, Format>;
}

struct HelloServiceImpl;

impl<Format: EncodingFormat> HelloService<Format> for HelloServiceImpl {
    async fn hello(&self) -> String {
        "Hello from server".to_string()
    }
}

impl<Format: EncodingFormat> IntoService<Format> for HelloServiceImpl {
    type Wrapper = HelloServiceWrapper<Self, Format>;
}
