use crate::{
    types::{Identifier, LenWithPayloadLengthDelimited, Payload, SequenceNumber},
    ICMP_HEADER_SIZE,
};

//
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PayloadLengthDelimitedEchoRequest {
    pub identifier: Identifier,
    pub sequence_number: SequenceNumber,
    payload_with_len: Payload,
}

impl PayloadLengthDelimitedEchoRequest {
    pub fn new(
        identifier: Option<Identifier>,
        sequence_number: Option<SequenceNumber>,
        payload: impl AsRef<[u8]>,
    ) -> Self {
        let payload = payload.as_ref();
        let len = LenWithPayloadLengthDelimited::new(payload.len());

        let mut payload_with_len = len.to_bytes().to_vec();
        payload_with_len.extend_from_slice(payload);

        Self {
            identifier: identifier.unwrap_or_else(Identifier::gen),
            sequence_number: sequence_number.unwrap_or_default(),
            payload_with_len: payload_with_len.into(),
        }
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload_with_len[2..]
    }

    pub fn len(&self) -> LenWithPayloadLengthDelimited {
        LenWithPayloadLengthDelimited::from_bytes(
            self.payload_with_len[..2].try_into().expect("Never"),
        )
    }

    pub fn render_v4_packet_bytes(&self) -> Vec<u8> {
        use pnet_packet::{
            icmp::{checksum, echo_request::MutableEchoRequestPacket, IcmpPacket, IcmpTypes},
            Packet as _,
        };

        //
        let mut buf = vec![0; ICMP_HEADER_SIZE + self.payload_with_len.len()];
        let mut echo_request_packet = MutableEchoRequestPacket::new(&mut buf[..])
            .expect("Never when MutableEchoRequestPacket::new");
        echo_request_packet.set_icmp_type(IcmpTypes::EchoRequest);
        echo_request_packet.set_identifier(self.identifier.into_inner());
        echo_request_packet.set_sequence_number(self.sequence_number.into_inner());
        echo_request_packet.set_payload(&self.payload_with_len);

        let icmp_packet =
            IcmpPacket::new(echo_request_packet.packet()).expect("Never when IcmpPacket::new");
        let checksum = checksum(&icmp_packet);
        echo_request_packet.set_checksum(checksum);

        echo_request_packet.packet().to_vec()
    }

    pub fn render_v6_packet_bytes(&self) -> Vec<u8> {
        use pnet_packet::{
            icmpv6::{echo_request::MutableEchoRequestPacket, Icmpv6Types},
            Packet as _,
        };

        let mut buf = vec![0; ICMP_HEADER_SIZE + self.payload_with_len.len()];
        let mut echo_request_packet = MutableEchoRequestPacket::new(&mut buf[..])
            .expect("Never when MutableEchoRequestPacket::new");
        echo_request_packet.set_icmpv6_type(Icmpv6Types::EchoRequest);
        echo_request_packet.set_identifier(self.identifier.into_inner());
        echo_request_packet.set_sequence_number(self.sequence_number.into_inner());
        echo_request_packet.set_payload(&self.payload_with_len);

        // https://github.com/kolapapa/surge-ping/blob/0.7.3/src/icmp/icmpv6.rs#L26
        // https://tools.ietf.org/html/rfc3542#section-3.1
        // the checksum is omitted, the kernel will insert it.

        echo_request_packet.packet().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let echo_request =
            PayloadLengthDelimitedEchoRequest::new(Some(1.into()), Some(2.into()), b"1234");
        assert_eq!(echo_request.payload(), b"1234");
        assert_eq!(echo_request.len(), LenWithPayloadLengthDelimited::new(4));
    }

    #[test]
    fn test_render() {
        let echo_request =
            PayloadLengthDelimitedEchoRequest::new(Some(1.into()), Some(2.into()), b"1234");
        assert_eq!(
            echo_request.render_v4_packet_bytes(),
            vec![8, 0, 147, 146, 0, 1, 0, 2, 0, 4, 49, 50, 51, 52]
        );
        assert_eq!(
            echo_request.render_v6_packet_bytes(),
            vec![128, 0, 0, 0, 0, 1, 0, 2, 0, 4, 49, 50, 51, 52]
        );
    }
}
