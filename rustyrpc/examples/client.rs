mod common;

use std::{sync::Arc, time::Duration};

use common::{auth_service::AuthServiceClient, hello_service::HelloServiceClient};
use log::{error, info};
use quinn::ClientConfig;
use rustyrpc::{
    client::Client,
    format::{Decode, Encode, EncodingFormat},
    protocol::{PrivateServiceDeallocateRequestResult, RequestKind, ServiceCallRequestResult},
    transport,
};

fn parse_args() -> (String, String) {
    const EXPECTED_ARGUMENTS_ERROR_MESSAGE: &str = "Expected two arguments";
    let mut args = std::env::args().skip(1);

    let username = args.next().expect(EXPECTED_ARGUMENTS_ERROR_MESSAGE);
    let password = args.next().expect(EXPECTED_ARGUMENTS_ERROR_MESSAGE);
    (username, password)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let (username, password) = parse_args();

    let connection = transport::quic::Connection::connect(
        client_config(),
        "0.0.0.0:0".parse().unwrap(),
        "127.0.0.1:8888".parse().unwrap(),
        "localhost",
    )
    .await
    .unwrap();

    let client = Arc::new(Client::from(connection));
    let auth_service_client: AuthServiceClient<_, _> =
        client.clone().get_service_client().await.unwrap();

    if let Some(hello_service_client) = auth_service_client.auth(&username, &password).await {
        info!("Successful authentication");

        start_healthcheck(hello_service_client).await;
    } else {
        error!("Failed to authenticate: invalid username or password");
    }
    tokio::time::sleep(Duration::from_secs(2)).await; // Waiting to allow HelloService deallocation request to be sent.
}

async fn start_healthcheck<Connection: transport::ClientConnection, Format: EncodingFormat>(
    hello_service_client: HelloServiceClient<Connection, Format>,
) where
    for<'a> RequestKind<'a>: Encode<Format>,
    ServiceCallRequestResult: Decode<Format>,
    PrivateServiceDeallocateRequestResult: Decode<Format>,
    (): Encode<Format>,
    String: Decode<Format>,
{
    for _ in 0..3 {
        if let Err(err) = hello_service_client.hello().await {
            error!("Healthcheck attempt failed: {err:?}");
        } else {
            info!("Successful healthcheck attempt");
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

struct SkipCertVerification;

impl rustls::client::ServerCertVerifier for SkipCertVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

fn client_config() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(SkipCertVerification))
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}
