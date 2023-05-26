use std::net::UdpSocket;

use crate::{config::Config, AsyncClientWithConfigError};

// Ref https://github.com/kolapapa/surge-ping/blob/0.7.3/src/client.rs#L36-L54
pub fn new_socket2_socket(config: &Config) -> Result<socket2::Socket, AsyncClientWithConfigError> {
    use socket2::{Domain, Protocol, SockAddr, Socket, Type};

    let socket = if config.is_ipv6() {
        Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::ICMPV6)).map_err(|err| {
            if err.raw_os_error() == Some(93) {
                AsyncClientWithConfigError::IcmpV6ProtocolNotSupported(err)
            } else {
                AsyncClientWithConfigError::OtherIoError(err)
            }
        })?
    } else {
        Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?
    };

    socket.set_nonblocking(true)?;

    if let Some(bind) = config.bind {
        socket.bind(&SockAddr::from(bind))?;
    }
    #[cfg(any(
        target_os = "ios",
        target_os = "macos",
        target_os = "tvos",
        target_os = "watchos",
    ))]
    if let Some(interface_index) = config.interface_index {
        socket.bind_device_by_index(Some(interface_index))?;
    }
    if let Some(ttl) = config.ttl {
        socket.set_ttl(ttl)?;
    }
    #[cfg(target_os = "freebsd")]
    if let Some(fib) = config.fib {
        socket.set_fib(fib)?;
    }

    Ok(socket)
}

//
pub fn new_std_udp_socket(config: &Config) -> Result<UdpSocket, AsyncClientWithConfigError> {
    #[cfg(unix)]
    use std::os::fd::{FromRawFd as _, IntoRawFd as _};
    #[cfg(windows)]
    use std::os::windows::{FromRawSocket as _, IntoRawSocket as _};

    let socket = new_socket2_socket(config)?;

    #[cfg(unix)]
    let udp_socket = unsafe { UdpSocket::from_raw_fd(socket.into_raw_fd()) };
    #[cfg(windows)]
    let udp_socket = unsafe { UdpSocket::from_raw_socket(socket.into_raw_socket()) };

    Ok(udp_socket)
}
