use core::fmt;
use byteorder::{ByteOrder, NetworkEndian};

enum_with_unknown! {
    /// Ethernet protocol type.
    pub enum ProtocolType(u16) {
        Ipv4 = 0x0800,
        Arp  = 0x0806,
        Iv6  = 0x86DD
    }
}

/// A six-octet Ethernet II address.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Address(pub [u8; 6]);

impl Address {
    /// Construct an Ethernet address from a sequence of octets, in big-endian.
    ///
    /// # Panics
    /// The function panics if `data` is not six octets long.
    pub fn from_bytes(data: &[u8]) -> Address {
        let mut bytes = [0; 6];
        bytes.copy_from_slice(data);
        Address(bytes)
    }

    /// Return an Ethernet address as a sequence of octets, in big-endian.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.0;
        write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
               bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5])
    }
}

/// A read/write wrapper around an Ethernet II frame.
#[derive(Debug)]
pub struct Frame<T: AsRef<[u8]>>(T);

mod field {
    use ::field::*;

    pub const SOURCE:      Field     =  0..6;
    pub const DESTINATION: Field     =  6..12;
    pub const LENGTH:      Field     = 12..14;
    pub const PAYLOAD:     FieldFrom = 14..;
}

impl<T: AsRef<[u8]>> Frame<T> {
    /// Wrap a buffer with an Ethernet frame. Returns an error if the buffer
    /// is too small or too large to contain one.
    pub fn new(storage: T) -> Result<Frame<T>, ()> {
        let len = storage.as_ref().len();
        if !(64..1518).contains(len) {
            Err(()) // TODO: error type?
        } else {
            Ok(Frame(storage))
        }
    }

    /// Consumes the frame, returning the underlying buffer.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Return the source address field.
    #[inline(always)]
    pub fn source(&self) -> Address {
        let bytes = self.0.as_ref();
        Address::from_bytes(&bytes[field::SOURCE])
    }

    /// Return the destination address field.
    #[inline(always)]
    pub fn destination(&self) -> Address {
        let bytes = self.0.as_ref();
        Address::from_bytes(&bytes[field::DESTINATION])
    }

    /// Return the length field, without checking for 802.1Q.
    #[inline(always)]
    pub fn length(&self) -> u16 {
        let bytes = self.0.as_ref();
        NetworkEndian::read_u16(&bytes[field::LENGTH])
    }

    /// Return a pointer to the payload, without checking for 802.1Q.
    #[inline(always)]
    pub fn payload(&self) -> &[u8] {
        let bytes = self.0.as_ref();
        &bytes[field::PAYLOAD]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Frame<T> {
    /// Set the source address field.
    #[inline(always)]
    pub fn set_source(&mut self, value: Address) {
        let bytes = self.0.as_mut();
        bytes[field::SOURCE].copy_from_slice(value.as_bytes())
    }

    /// Set the destination address field.
    #[inline(always)]
    pub fn set_destination(&mut self, value: Address) {
        let bytes = self.0.as_mut();
        bytes[field::DESTINATION].copy_from_slice(value.as_bytes())
    }

    /// Set the length field.
    #[inline(always)]
    pub fn set_length(&mut self, value: u16) {
        let bytes = self.0.as_mut();
        NetworkEndian::write_u16(&mut bytes[field::LENGTH], value)
    }

    /// Return a mutable pointer to the payload.
    #[inline(always)]
    pub fn payload_mut(&mut self) -> &mut [u8] {
        let bytes = self.0.as_mut();
        &mut bytes[field::PAYLOAD]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static FRAME_BYTES: [u8; 64] =
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
         0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
         0x00, 0x40,
         0xaa, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0xff];

    static PAYLOAD_BYTES: [u8; 50] =
        [0xaa, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
         0x00, 0xff];

    #[test]
    fn test_deconstruct() {
        let frame = Frame::new(&FRAME_BYTES[..]).unwrap();
        assert_eq!(frame.source(), Address([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]));
        assert_eq!(frame.destination(), Address([0x11, 0x12, 0x13, 0x14, 0x15, 0x16]));
        assert_eq!(frame.length(), FRAME_BYTES.len() as u16);
        assert_eq!(frame.payload(), &PAYLOAD_BYTES[..]);
    }

    #[test]
    fn test_construct() {
        let mut bytes = vec![0; 64];
        let mut frame = Frame::new(&mut bytes).unwrap();
        frame.set_source(Address([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]));
        frame.set_destination(Address([0x11, 0x12, 0x13, 0x14, 0x15, 0x16]));
        frame.set_length(64);
        frame.payload_mut().copy_from_slice(&PAYLOAD_BYTES[..]);
        assert_eq!(&frame.into_inner()[..], &FRAME_BYTES[..]);
    }
}
