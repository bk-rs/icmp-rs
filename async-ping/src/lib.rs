pub use icmp_client;
pub use icmp_packet;

use core::time::Duration;
use std::{
    collections::HashMap,
    io::{Error as IoError, ErrorKind as IoErrorKind},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    sync::Arc,
    time::Instant,
};

use icmp_client::{AsyncClient, Config as ClientConfig};
use icmp_packet::{
    icmpv4::ParseError as Icmpv4ParseError, icmpv6::ParseError as Icmpv6ParseError, Icmp, Icmpv4,
    Icmpv6, PayloadLengthDelimitedEchoRequest,
};
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};
use tracing::{event, Level};

//
type V4RecvFromMap =
    Arc<Mutex<HashMap<SocketAddr, Sender<(Result<Icmpv4, Icmpv4ParseError>, Instant)>>>>;
type V6RecvFromMap =
    Arc<Mutex<HashMap<SocketAddr, Sender<(Result<Icmpv6, Icmpv6ParseError>, Instant)>>>>;

//
pub struct PingClient<C>
where
    C: AsyncClient,
{
    v4_client: Arc<C>,
    v6_client: Arc<C>,
    v4_recv_from_map: V4RecvFromMap,
    v6_recv_from_map: V6RecvFromMap,
}

impl<C> core::fmt::Debug for PingClient<C>
where
    C: AsyncClient,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PingClient").finish()
    }
}

impl<C> Clone for PingClient<C>
where
    C: AsyncClient,
{
    fn clone(&self) -> Self {
        Self {
            v4_client: self.v4_client.clone(),
            v6_client: self.v6_client.clone(),
            v4_recv_from_map: self.v4_recv_from_map.clone(),
            v6_recv_from_map: self.v6_recv_from_map.clone(),
        }
    }
}

impl<C> PingClient<C>
where
    C: AsyncClient,
{
    pub fn new(
        mut v4_client_config: ClientConfig,
        mut v6_client_config: ClientConfig,
    ) -> Result<Self, IoError> {
        if v4_client_config.is_ipv6() {
            return Err(IoError::new(IoErrorKind::Other, "v4_client_config invalid"));
        }
        if v4_client_config.bind.is_none() {
            v4_client_config.bind = Some(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0).into());
        }
        if !v6_client_config.is_ipv6() {
            return Err(IoError::new(IoErrorKind::Other, "v4_client_config invalid"));
        }
        if v6_client_config.bind.is_none() {
            v6_client_config.bind =
                Some(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0, 0, 0).into());
        }

        let v4_client = Arc::new(C::with_config(&v4_client_config)?);
        let v6_client = Arc::new(C::with_config(&v6_client_config)?);

        let v4_recv_from_map = Arc::new(Mutex::new(HashMap::new()));
        let v6_recv_from_map = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            v4_client,
            v6_client,
            v4_recv_from_map,
            v6_recv_from_map,
        })
    }

    pub async fn handle_v4_recv_from(&self) {
        let mut buf = [0; 2048];
        let bytes_present_map: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        loop {
            match self.v4_client.recv_from(&mut buf).await {
                Ok((n, addr)) => {
                    let instant_end = Instant::now();
                    let bytes_read = buf[..n].to_owned();

                    let v4_recv_from_map = self.v4_recv_from_map.clone();
                    let bytes_present_map = bytes_present_map.clone();

                    tokio::spawn(async move {
                        let bytes = if let Some(mut bytes_present) =
                            bytes_present_map.lock().await.remove(&addr)
                        {
                            bytes_present.extend_from_slice(&bytes_read);
                            bytes_present
                        } else {
                            bytes_read
                        };

                        match Icmpv4::parse_from_packet_bytes(&bytes) {
                            Ok(Some(icmpv4)) => {
                                if let Some(tx) = v4_recv_from_map.lock().await.remove(&addr) {
                                    if let Err(err) = tx.try_send((Ok(icmpv4), instant_end)) {
                                        event!(
                                            Level::ERROR,
                                            "tx.send failed, err:{err} addr:{addr}"
                                        );
                                    }
                                } else {
                                    event!(
                                        Level::WARN,
                                        "v4_recv_from_map.remove None, addr:{addr}"
                                    );
                                }
                            }
                            Ok(None) => {
                                bytes_present_map.lock().await.insert(addr, bytes);
                            }
                            Err(err) => {
                                if let Some(tx) = v4_recv_from_map.lock().await.remove(&addr) {
                                    if let Err(err) = tx.try_send((Err(err), instant_end)) {
                                        event!(
                                            Level::ERROR,
                                            "tx.send failed, err:{err} addr:{addr}"
                                        );
                                    }
                                } else {
                                    event!(
                                        Level::WARN,
                                        "v4_recv_from_map.remove None, addr:{addr}"
                                    );
                                }
                            }
                        }
                    });
                }
                Err(err) => {
                    event!(Level::ERROR, "v4_client.recv_from failed, err:{err}");
                }
            }
        }
    }

    pub async fn handle_v6_recv_from(&self) {
        let mut buf = [0; 2048];
        let bytes_present_map: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        loop {
            match self.v6_client.recv_from(&mut buf).await {
                Ok((n, addr)) => {
                    let instant_end = Instant::now();
                    let bytes_read = buf[..n].to_owned();

                    let v6_recv_from_map = self.v6_recv_from_map.clone();
                    let bytes_present_map = bytes_present_map.clone();

                    tokio::spawn(async move {
                        let bytes = if let Some(mut bytes_present) =
                            bytes_present_map.lock().await.remove(&addr)
                        {
                            bytes_present.extend_from_slice(&bytes_read);
                            bytes_present
                        } else {
                            bytes_read
                        };

                        match Icmpv6::parse_from_packet_bytes(&bytes) {
                            Ok(Some(icmpv6)) => {
                                if let Some(tx) = v6_recv_from_map.lock().await.remove(&addr) {
                                    if let Err(err) = tx.try_send((Ok(icmpv6), instant_end)) {
                                        event!(
                                            Level::ERROR,
                                            "tx.send failed, err:{err} addr:{addr}"
                                        );
                                    }
                                } else {
                                    event!(
                                        Level::WARN,
                                        "v6_recv_from_map.remove None, addr:{addr}"
                                    );
                                }
                            }
                            Ok(None) => {
                                bytes_present_map.lock().await.insert(addr, bytes);
                            }
                            Err(err) => {
                                if let Some(tx) = v6_recv_from_map.lock().await.remove(&addr) {
                                    if let Err(err) = tx.try_send((Err(err), instant_end)) {
                                        event!(
                                            Level::ERROR,
                                            "tx.send failed, err:{err} addr:{addr}"
                                        );
                                    }
                                } else {
                                    event!(
                                        Level::WARN,
                                        "v6_recv_from_map.remove None, addr:{addr}"
                                    );
                                }
                            }
                        }
                    });
                }
                Err(err) => {
                    event!(Level::ERROR, "v6_client.recv_from failed, err:{err}");
                }
            }
        }
    }

    pub async fn ping(
        &self,
        ip: IpAddr,
        identifier: Option<u16>,
        sequence_number: Option<u16>,
        payload: impl AsRef<[u8]>,
        timeout_dur: Duration,
    ) -> Result<(Icmp, Duration), PingError> {
        //
        let echo_request = PayloadLengthDelimitedEchoRequest::new(
            identifier.map(Into::into),
            sequence_number.map(Into::into),
            payload,
        );
        let echo_request_bytes = match ip {
            IpAddr::V4(_) => echo_request.render_v4_packet_bytes(),
            IpAddr::V6(_) => echo_request.render_v6_packet_bytes(),
        };

        //
        let rx = match ip {
            IpAddr::V4(_) => {
                let (tx, rx) = mpsc::channel(1);

                self.v4_recv_from_map
                    .lock()
                    .await
                    .insert((ip, 0).into(), tx);

                Ok(rx)
            }
            IpAddr::V6(_) => {
                let (tx, rx) = mpsc::channel(1);

                self.v6_recv_from_map
                    .lock()
                    .await
                    .insert((ip, 0).into(), tx);

                Err(rx)
            }
        };

        //
        let client = match ip {
            IpAddr::V4(_) => &self.v4_client,
            IpAddr::V6(_) => &self.v6_client,
        };

        let instant_begin = Instant::now();

        {
            let mut n_write = 0;
            while !echo_request_bytes[n_write..].is_empty() {
                let n = client
                    .send_to(&echo_request_bytes[n_write..], (ip, 0))
                    .await
                    .map_err(PingError::Send)?;
                n_write += n;

                if n == 0 {
                    return Err(PingError::Send(IoErrorKind::WriteZero.into()));
                }
            }
        }

        //
        match rx {
            Ok(mut rx) => {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(timeout_dur.as_millis() as u64),
                    rx.recv(),
                )
                .await
                {
                    Ok(Some((Ok(icmpv4), instant_end))) => Ok((
                        Icmp::V4(icmpv4),
                        instant_end
                            .checked_duration_since(instant_begin)
                            .unwrap_or(instant_begin.elapsed()),
                    )),
                    Ok(Some((Err(err), _))) => Err(PingError::Icmpv4ParseError(err)),
                    Ok(None) => Err(PingError::Unknown("rx.recv None".to_string())),
                    Err(_) => Err(PingError::RecvTimedOut),
                }
            }
            Err(mut rx) => {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(timeout_dur.as_millis() as u64),
                    rx.recv(),
                )
                .await
                {
                    Ok(Some((Ok(icmpv6), instant_end))) => Ok((
                        Icmp::V6(icmpv6),
                        instant_end
                            .checked_duration_since(instant_begin)
                            .unwrap_or(instant_begin.elapsed()),
                    )),
                    Ok(Some((Err(err), _))) => Err(PingError::Icmpv6ParseError(err)),
                    Ok(None) => Err(PingError::Unknown("rx.recv None".to_string())),
                    Err(_) => Err(PingError::RecvTimedOut),
                }
            }
        }
    }

    pub async fn ping_v4(
        &self,
        ip: Ipv4Addr,
        identifier: Option<u16>,
        sequence_number: Option<u16>,
        payload: impl AsRef<[u8]>,
        timeout_dur: Duration,
    ) -> Result<(Icmpv4, Duration), PingError> {
        let (icmp, dur) = self
            .ping(ip.into(), identifier, sequence_number, payload, timeout_dur)
            .await?;
        match icmp {
            Icmp::V4(icmp) => Ok((icmp, dur)),
            Icmp::V6(_) => Err(PingError::Unknown("unreachable".to_string())),
        }
    }

    pub async fn ping_v6(
        &self,
        ip: Ipv6Addr,
        identifier: Option<u16>,
        sequence_number: Option<u16>,
        payload: impl AsRef<[u8]>,
        timeout_dur: Duration,
    ) -> Result<(Icmpv6, Duration), PingError> {
        let (icmp, dur) = self
            .ping(ip.into(), identifier, sequence_number, payload, timeout_dur)
            .await?;
        match icmp {
            Icmp::V4(_) => Err(PingError::Unknown("unreachable".to_string())),
            Icmp::V6(icmp) => Ok((icmp, dur)),
        }
    }
}

//
#[derive(Debug)]
pub enum PingError {
    Send(IoError),
    Icmpv4ParseError(Icmpv4ParseError),
    Icmpv6ParseError(Icmpv6ParseError),
    RecvTimedOut,
    Unknown(String),
}
impl core::fmt::Display for PingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for PingError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() -> Result<(), Box<dyn std::error::Error>> {
        let client = PingClient::<icmp_client::impl_tokio::Client>::new(
            ClientConfig::new(),
            ClientConfig::with_ipv6(),
        )?;

        {
            let client = client.clone();
            tokio::spawn(async move {
                client.handle_v4_recv_from().await;
            });
        }

        {
            let client = client.clone();
            tokio::spawn(async move {
                client.handle_v6_recv_from().await;
            });
        }

        {
            match client
                .ping(
                    "127.0.0.1".parse().expect("Never"),
                    None,
                    None,
                    vec![0; 32],
                    Duration::from_secs(2),
                )
                .await
            {
                Ok((icmp, dur)) => {
                    println!("{dur:?} {icmp:?}");
                }
                Err(err) => panic!("{err}"),
            }
        }

        {
            match client
                .ping(
                    "::1".parse().expect("Never"),
                    None,
                    None,
                    vec![0; 32],
                    Duration::from_secs(2),
                )
                .await
            {
                Ok((icmp, dur)) => {
                    println!("{dur:?} {icmp:?}");
                }
                Err(err) => panic!("{err}"),
            }
        }

        Ok(())
    }
}
