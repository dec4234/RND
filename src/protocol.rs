use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

pub trait PacketKind: Clone + Copy + PartialEq + Eq {

}

#[derive(PartialEq, Clone)]
pub enum PacketDirection {
    ServerBound,
    ClientBound,
    Both,
}

#[macro_export]
macro_rules! define_protocol {
    ($version: literal, $packet: ident, $rawpacket: ident, $rawpacketbody: ident, $packettype: ident => {
        $($nam: ident, $id: literal, $direction: ident => $body: ident {
            $($fnam: ident: $ftyp: ty),* }),*
        }
    ) => {
        $crate::as_item! {
            #[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
            pub enum $packet {
                $($nam($body)),*,
            }
        }

        $($crate::structify!($body { $($fnam: $ftyp),* });)*
    };
}

#[macro_export]
macro_rules! structify {
    ($body: ident { }) => {
        #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
        pub struct $body;
    };

    ($body: ident $(<$($g: ident),*>)? {
          $($fname: ident: $ftyp: ty),+
    }) => {
        $crate::as_item! {
            #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
            pub struct $body$(<$($g),*> where $($g: Serialize + alloc::fmt::Debug + Clone + PartialEq + Deserialize + Serialize),*)? {
                $(pub $fname: $ftyp),+
            }
        }

        #[allow(unused_parens)]
        impl $(<$($g),*>)? From<($($ftyp), +)> for $body$(<$($g),*>)? $(where $($g: alloc::fmt::Debug + Clone + PartialEq),*)? {
            fn from(other: ($($ftyp),+)) -> Self {
                let ($($fname),+) = other;
                Self { $($fname),+ }
            }
        }

        #[allow(unused_parens)]
        impl $(<$($g),*>)? From<$body$(<$($g),*>)?> for ($($ftyp),+) $(where $($g: alloc::fmt::Debug + Clone + PartialEq),*)? {
            fn from(other: $body$(<$($g),*>)?) -> Self {
                ($(other.$fname),+)
            }
        }
    };
}

#[macro_export]
macro_rules! as_item {
    ($i:item) => {
        $i
    };
}