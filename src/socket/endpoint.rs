// use crate::{filesystem::vfs::InodeId, net::socket};
// use alloc::{string::String, sync::Arc};

pub use smoltcp::wire::IpEndpoint;

// use super::unix::ns::abs::AbsHandle;

#[derive(Debug, Clone)]
pub enum Endpoint {
    // /// 链路层端点
    // LinkLayer(LinkLayerEndpoint),
    /// 网络层端点
    Ip(IpEndpoint),
    Other,
    // /// inode端点,Unix实际保存的端点
    // Inode((Arc<socket::SocketInode>, String)),
    // /// Unix传递id索引和path所用的端点
    // Unixpath((InodeId, String)),
    // /// Unix抽象端点
    // Abspath((AbsHandle, String)),
}

// /// @brief 链路层端点
// #[derive(Debug, Clone)]
// pub struct LinkLayerEndpoint {
//     /// 网卡的接口号
//     pub interface: usize,
// }

// impl LinkLayerEndpoint {
//     /// @brief 创建一个链路层端点
//     ///
//     /// @param interface 网卡的接口号
//     ///
//     /// @return 返回创建的链路层端点
//     pub fn new(interface: usize) -> Self {
//         Self { interface }
//     }
// }

impl From<IpEndpoint> for Endpoint {
    fn from(endpoint: IpEndpoint) -> Self {
        Self::Ip(endpoint)
    }
}
