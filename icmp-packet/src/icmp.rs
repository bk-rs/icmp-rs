use crate::{icmpv4::Icmpv4, icmpv6::Icmpv6};

//
#[derive(Debug, Clone)]
pub enum Icmp {
    V4(Icmpv4),
    V6(Icmpv6),
}
