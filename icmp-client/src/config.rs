use core::num::NonZeroU32;
use std::net::SocketAddr;

//
#[derive(Debug, Clone, Default)]
pub struct Config {
    is_ipv6: bool,
    pub bind: Option<SocketAddr>,
    pub interface_index: Option<NonZeroU32>,
    pub ttl: Option<u32>,
    pub fib: Option<u32>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ipv6() -> Self {
        Self {
            is_ipv6: true,
            ..Default::default()
        }
    }

    pub fn is_ipv6(&self) -> bool {
        self.bind.map(|x| x.is_ipv6()).unwrap_or(self.is_ipv6)
    }
}

impl Config {
    pub fn bind(mut self, bind: SocketAddr) -> Self {
        self.bind = Some(bind);
        self
    }

    pub fn interface_index(mut self, interface_index: NonZeroU32) -> Self {
        self.interface_index = Some(interface_index);
        self
    }

    pub fn ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn fib(mut self, fib: u32) -> Self {
        self.fib = Some(fib);
        self
    }
}
