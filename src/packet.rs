use crate::define_protocol;
use serde::{Deserialize, Serialize};

define_protocol!(1, Packet, RawPacket, RawPacketBody, RawPacketKind => {
    Handshake, 0x00, ServerBound => HandshakeSpec {
        payload: i64
    },
    Message, 0x01, Both => MessageSpec {
        payload: String
    },
    EnableEncryption, 0x02, Both => EnableEncryptionSpec {
        public: [u8; 32]
    }
});