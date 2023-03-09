use crate::ICMP_HEADER_SIZE;

//
wrapping_macro::wrapping_int! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Identifier(pub u16);
}
impl Identifier {
    pub fn gen() -> Self {
        let pid = std::process::id();
        if pid <= u16::MAX as u32 {
            Self(pid as u16)
        } else {
            #[cfg(feature = "rand")]
            let id = rand::random();
            #[cfg(not(feature = "rand"))]
            let id = 0;
            Self(id)
        }
    }
}

//
wrapping_macro::wrapping_int! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct SequenceNumber(pub u16);
}

//
wrapping_macro::wrapping! {
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct Payload(pub Vec<u8>);
}

//
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LenWithPayloadLengthDelimited(u16);
impl LenWithPayloadLengthDelimited {
    pub const fn size() -> usize {
        core::mem::size_of::<u16>()
    }

    pub const fn max() -> u16 {
        u16::MAX - ICMP_HEADER_SIZE as u16 - core::mem::size_of::<u16>() as u16
    }

    pub fn new(v: usize) -> Self {
        assert!(v <= Self::max() as usize);
        Self(v as u16)
    }

    pub fn inner(&self) -> &u16 {
        &self.0
    }

    pub fn to_bytes(&self) -> [u8; 2] {
        self.0.to_be_bytes()
    }

    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        Self(u16::from_be_bytes(bytes))
    }
}
