use linux_errnos::Errno;
use num_derive::{FromPrimitive, ToPrimitive};
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum SOCK {
    Stream = 1,
    Datagram = 2,
    Raw = 3,
    RDM = 4,
    SeqPacket = 5,
    DCCP = 6,
    Packet = 10,
}

use super::posix::PosixArgsSocketType;
impl TryFrom<PosixArgsSocketType> for SOCK {
    type Error = Errno;
    fn try_from(x: PosixArgsSocketType) -> Result<Self, Self::Error> {
        use num_traits::FromPrimitive;
        <Self as FromPrimitive>::from_u32(x.types().bits()).ok_or(Self::Error::EINVAL)
    }
}
