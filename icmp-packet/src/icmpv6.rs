use pnet_packet::{
    icmpv6::{echo_reply::EchoReplyPacket, Icmpv6Code, Icmpv6Packet, Icmpv6Type, Icmpv6Types},
    Packet,
};

use crate::{
    echo_reply::PayloadLengthDelimitedEchoReply, types::Payload, LenWithPayloadLengthDelimited,
    ICMP_HEADER_SIZE,
};

//
#[derive(Debug, Clone)]
pub enum Icmpv6 {
    EchoReply(PayloadLengthDelimitedEchoReply),
    Other(Icmpv6Type, Icmpv6Code, Payload),
}

impl Icmpv6 {
    pub fn parse_from_packet_bytes(bytes: &[u8]) -> Result<Option<Self>, ParseError> {
        if bytes.len() < ICMP_HEADER_SIZE {
            return Ok(None);
        }

        let icmp_packet = if let Some(x) = Icmpv6Packet::new(bytes) {
            x
        } else {
            return Err(ParseError::NotIcmpPacket);
        };

        match icmp_packet.get_icmpv6_type() {
            Icmpv6Types::EchoReply => {
                let echo_reply_packet =
                    EchoReplyPacket::owned(bytes[..ICMP_HEADER_SIZE].to_owned())
                        .ok_or(ParseError::NotEchoReplyPacket)?;

                let char_a = if let Some(x) = bytes.get(ICMP_HEADER_SIZE) {
                    x
                } else {
                    return Ok(None);
                };
                let char_b = if let Some(x) = bytes.get(ICMP_HEADER_SIZE + 1) {
                    x
                } else {
                    return Ok(None);
                };
                let len = LenWithPayloadLengthDelimited::from_bytes([*char_a, *char_b]);

                if bytes.len()
                    < ICMP_HEADER_SIZE
                        + LenWithPayloadLengthDelimited::size()
                        + (*len.inner()) as usize
                {
                    return Ok(None);
                }

                return Ok(Some(Icmpv6::EchoReply(
                    PayloadLengthDelimitedEchoReply::new(
                        echo_reply_packet.get_identifier().into(),
                        echo_reply_packet.get_sequence_number().into(),
                        len,
                        bytes[ICMP_HEADER_SIZE + LenWithPayloadLengthDelimited::size()
                            ..ICMP_HEADER_SIZE
                                + LenWithPayloadLengthDelimited::size()
                                + (*len.inner()) as usize]
                            .to_vec()
                            .into(),
                    ),
                )));
            }
            icmp_type => Ok(Some(Icmpv6::Other(
                icmp_type,
                icmp_packet.get_icmpv6_code(),
                icmp_packet.payload().to_vec().into(),
            ))),
        }
    }
}

//
#[derive(Debug)]
pub enum ParseError {
    NotIcmpPacket,
    NotEchoReplyPacket,
}
impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::echo_request::PayloadLengthDelimitedEchoRequest;

    #[test]
    fn test_parse_from_packet_bytes() {
        let echo_request =
            PayloadLengthDelimitedEchoRequest::new(Some(1.into()), Some(2.into()), b"1234");
        let mut bytes = echo_request.render_v6_packet_bytes();
        bytes[0] = Icmpv6Types::EchoReply.0;

        match Icmpv6::parse_from_packet_bytes(&bytes) {
            Ok(Some(Icmpv6::EchoReply(PayloadLengthDelimitedEchoReply {
                identifier,
                sequence_number,
                len,
                payload,
            }))) => {
                assert_eq!(identifier, echo_request.identifier);
                assert_eq!(sequence_number, echo_request.sequence_number);
                assert_eq!(len, echo_request.len());
                assert_eq!(payload.inner(), echo_request.payload());
            }
            x => panic!("{x:?}"),
        }
    }
}
