use alloc::sync::Arc;
use linux_errnos::Errno as SystemError;
use smoltcp::{self, wire::IpProtocol};

use crate::{
    posix::SOCK,
    socket::{
        inet::UdpSocket,
        Family,
        Socket, // SocketInode,
    },
};

fn create_inet_socket(
    version: smoltcp::wire::IpVersion,
    socket_type: SOCK,
    protocol: smoltcp::wire::IpProtocol,
) -> Result<Arc<dyn Socket>, SystemError> {
    // log::debug!("type: {:?}, protocol: {:?}", socket_type, protocol);
    match socket_type {
        SOCK::Datagram => match protocol {
            IpProtocol::HopByHop | IpProtocol::Udp => {
                Ok(UdpSocket::new(false))
            }
            _ => {
                Err(SystemError::EPROTONOSUPPORT)
            }
        },
        // SOCK::Stream => match protocol {
        //     IpProtocol::HopByHop | IpProtocol::Tcp => {
        //         log::debug!("create tcp socket");
        //         return Ok(TcpSocket::new(false, version));
        //     }
        //     _ => {
        //         return Err(SystemError::EPROTONOSUPPORT);
        //     }
        // },
        SOCK::Raw => {
            todo!("raw")
        }
        _ => {
            Err(SystemError::EPROTONOSUPPORT)
        }
    }
}

pub struct Inet;
impl Family for Inet {
    fn socket(stype: SOCK, protocol: u32) -> Result<Arc<dyn Socket>, SystemError> {
        create_inet_socket(
            smoltcp::wire::IpVersion::Ipv4,
            stype,
            smoltcp::wire::IpProtocol::from(protocol as u8),
        )
    }
}

// pub struct Inet6;
// impl Family for Inet6 {
//     fn socket(stype: PSOCK, protocol: u32) -> Result<Arc<dyn Socket>, SystemError> {
//         create_inet_socket(
//             smoltcp::wire::IpVersion::Ipv6,
//             stype,
//             smoltcp::wire::IpProtocol::from(protocol as u8),
//         )
//     }
// }
