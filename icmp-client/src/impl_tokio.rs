use std::{io::Error as IoError, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use tokio::net::UdpSocket;

use crate::{config::Config, utils::new_std_udp_socket, AsyncClient};

//
#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<UdpSocket>,
}

impl Client {
    pub fn new(config: &Config) -> Result<Self, IoError> {
        let udp_socket = new_std_udp_socket(config)?;
        let inner = UdpSocket::from_std(udp_socket)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

#[async_trait]
impl AsyncClient for Client {
    fn with_config(config: &Config) -> Result<Self, IoError> {
        Client::new(config)
    }

    async fn send_to<A: Into<SocketAddr> + Send>(
        &self,
        buf: &[u8],
        addr: A,
    ) -> Result<usize, IoError> {
        self.inner.send_to(buf, addr.into()).await
    }
    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), IoError> {
        self.inner.recv_from(buf).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client() -> Result<(), Box<dyn std::error::Error>> {
        crate::tests_helper::ping_ipv4::<Client>("127.0.0.1".parse().expect("Never")).await?;
        crate::tests_helper::ping_ipv6::<Client>("::1".parse().expect("Never")).await?;

        Ok(())
    }
}
