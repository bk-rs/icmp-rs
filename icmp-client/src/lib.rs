//
use std::{io::Error as IoError, net::SocketAddr};

use async_trait::async_trait;

#[async_trait]
pub trait AsyncClient {
    fn with_config(config: &Config) -> Result<Self, IoError>
    where
        Self: Sized;

    async fn send_to<A: Into<SocketAddr> + Send>(
        &self,
        buf: &[u8],
        addr: A,
    ) -> Result<usize, IoError>;
    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), IoError>;
}

//
pub mod config;
pub use config::Config;

pub mod utils;

//
#[cfg(feature = "impl_async_io")]
pub mod impl_async_io;
#[cfg(feature = "impl_tokio")]
pub mod impl_tokio;

#[cfg(any(feature = "impl_async_io", feature = "impl_tokio"))]
#[cfg(test)]
pub(crate) mod tests_helper {
    use super::*;

    use std::net::{Ipv4Addr, Ipv6Addr};

    use icmp_packet::{Icmpv4, Icmpv6, PayloadLengthDelimitedEchoRequest};

    pub(crate) async fn ping_ipv4<C: AsyncClient>(
        ip: Ipv4Addr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // ipv4
        let client = C::with_config(&Config::new().ttl(64))?;

        let echo_request =
            PayloadLengthDelimitedEchoRequest::new(Some(1.into()), Some(2.into()), b"1234");

        let echo_request_bytes = echo_request.render_v4_packet_bytes();
        client.send_to(&echo_request_bytes, (ip, 0)).await?;

        let mut buf = vec![0; 1024];
        let (n, addr_recv_from) = client.recv_from(&mut buf).await?;
        assert_eq!(addr_recv_from, (ip, 0).into());

        match Icmpv4::parse_from_packet_bytes(&buf[..n]) {
            Ok(Some(Icmpv4::EchoReply(echo_reply))) => {
                // TODO, why not eq
                // assert_eq!(echo_request.identifier, echo_reply.identifier);
                println!("{echo_reply:?}");
            }
            x => panic!("{x:?}"),
        }

        Ok(())
    }

    pub(crate) async fn ping_ipv6<C: AsyncClient>(
        ip: Ipv6Addr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = C::with_config(&Config::with_ipv6().ttl(64))?;

        let echo_request =
            PayloadLengthDelimitedEchoRequest::new(Some(1.into()), Some(2.into()), b"1234");

        let echo_request_bytes = echo_request.render_v6_packet_bytes();
        client.send_to(&echo_request_bytes, (ip, 0)).await?;

        let mut buf = vec![0; 1024];
        let (n, addr_recv_from) = client.recv_from(&mut buf).await?;
        assert_eq!(addr_recv_from, (ip, 0).into());

        match Icmpv6::parse_from_packet_bytes(&buf[..n]) {
            Ok(Some(Icmpv6::EchoReply(echo_reply))) => {
                // TODO, why not eq
                // assert_eq!(echo_request.identifier, echo_reply.identifier);
                println!("{echo_reply:?}");
            }
            x => panic!("{x:?}"),
        }

        Ok(())
    }
}
