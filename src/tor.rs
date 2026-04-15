use crate::error::OnionError;
use reqwest::Client;
use std::time::Duration;

/// Check if the Tor SOCKS5 proxy is reachable by attempting a TCP connection.
pub async fn check_tor_connection(proxy_addr: &str) -> bool {
    let addr = proxy_addr
        .trim_start_matches("socks5h://")
        .trim_start_matches("socks5://");

    matches!(
        tokio::time::timeout(Duration::from_secs(5), tokio::net::TcpStream::connect(addr),).await,
        Ok(Ok(_))
    )
}

/// Build a reqwest HTTP client configured to route through the Tor SOCKS5 proxy.
/// Uses `socks5h://` so that DNS resolution also happens over Tor.
pub fn build_client(proxy_addr: &str) -> Result<Client, OnionError> {
    let proxy = reqwest::Proxy::all(proxy_addr)
        .map_err(|e| OnionError::TorUnavailable(format!("{}: {}", proxy_addr, e)))?;

    Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(600))
        .connect_timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| OnionError::TorUnavailable(e.to_string()))
}

/// Build a reqwest HTTP client for the normal network.
pub fn build_normal_client() -> Result<Client, OnionError> {
    Client::builder()
        .timeout(Duration::from_secs(600))
        .connect_timeout(Duration::from_secs(120))
        .build()
        .map_err(OnionError::Http)
}
