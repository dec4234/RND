use crate::define_protocol;
use serde::{Deserialize, Serialize};

define_protocol!(1, Packet1, RawPacket1, RawPacket1Body, RawPacket1Kind => {
    Handshake, 0x00, ServerBound => HandshakeSpec {
        payload: i64
    },
    Message, 0x01, Both => MessageSpec {
        payload: String
    }
});