use crate::types::{Identifier, LenWithPayloadLengthDelimited, Payload, SequenceNumber};

//
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PayloadLengthDelimitedEchoReply {
    pub identifier: Identifier,
    pub sequence_number: SequenceNumber,
    pub len: LenWithPayloadLengthDelimited,
    pub payload: Payload,
}

impl PayloadLengthDelimitedEchoReply {
    pub fn new(
        identifier: Identifier,
        sequence_number: SequenceNumber,
        len: LenWithPayloadLengthDelimited,
        payload: Payload,
    ) -> Self {
        Self {
            identifier,
            sequence_number,
            len,
            payload,
        }
    }
}
