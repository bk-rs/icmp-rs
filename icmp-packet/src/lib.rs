//
pub use pnet_packet;

//
pub mod echo_reply;
pub use echo_reply::PayloadLengthDelimitedEchoReply;

pub mod echo_request;
pub use echo_request::PayloadLengthDelimitedEchoRequest;

pub mod icmp;
pub use icmp::Icmp;

pub mod icmpv4;
pub use icmpv4::Icmpv4;

pub mod icmpv6;
pub use icmpv6::Icmpv6;

pub mod types;
pub use types::{Identifier, LenWithPayloadLengthDelimited, Payload, SequenceNumber};

//
// https://docs.rs/pnet_packet/0.33.0/src/pnet_packet/icmp.rs.html#304-314
pub const ICMP_HEADER_SIZE: usize = 8;
