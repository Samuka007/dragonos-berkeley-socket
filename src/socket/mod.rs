pub mod common;
pub mod endpoint;
pub mod inet;

use crate::{libs::wait_queue::WaitQueue, posix::SOCK};
use core::any::Any;
use core::fmt::Debug;
use linux_errnos::Errno as SystemError;
use std::sync::Arc;

use super::{
    posix::{PMSG, PSOL},
    // SocketInode,
};
use common::shutdown::ShutdownTemp;
use endpoint::Endpoint;

/// # `Socket` methods
/// ## Reference
/// - [Posix standard](https://pubs.opengroup.org/onlinepubs/9699919799/)
#[allow(unused_variables)]
pub trait Socket: Sync + Send + Debug + Any {
    /// # `wait_queue`
    /// 获取socket的wait queue
    fn wait_queue(&self) -> &WaitQueue;
    /// # `socket_poll`
    /// 获取socket的事件。
    fn poll(&self) -> usize;

    fn send_buffer_size(&self) -> usize;
    fn recv_buffer_size(&self) -> usize;
    // /// # `accept`
    // /// 接受连接，仅用于listening stream socket
    // /// ## Block
    // /// 如果没有连接到来，会阻塞
    fn accept(&self) -> Result<(Arc<dyn Socket>, Endpoint), SystemError> {
        Err(SystemError::ENOSYS)
    }
    /// # `bind`
    /// 对应于POSIX的bind函数，用于绑定到本机指定的端点
    fn bind(&self, endpoint: Endpoint) -> Result<(), SystemError> {
        Err(SystemError::ENOSYS)
    }
    /// # `close`
    /// 关闭socket
    fn close(&self) -> Result<(), SystemError> {
        Ok(())
    }
    /// # `connect`
    /// 对应于POSIX的connect函数，用于连接到指定的远程服务器端点
    fn connect(&self, endpoint: Endpoint) -> Result<(), SystemError> {
        Err(SystemError::ENOSYS)
    }
    // fnctl
    // freeaddrinfo
    // getaddrinfo
    // getnameinfo
    /// # `get_peer_name`
    /// 获取对端的地址
    fn get_peer_name(&self) -> Result<Endpoint, SystemError> {
        Err(SystemError::ENOSYS)
    }
    /// # `get_name`
    /// 获取socket的地址
    fn get_name(&self) -> Result<Endpoint, SystemError> {
        Err(SystemError::ENOSYS)
    }
    /// # `get_option`
    /// 对应于 Posix `getsockopt` ，获取socket选项
    fn get_option(&self, level: PSOL, name: usize, value: &mut [u8]) -> Result<usize, SystemError> {
        log::warn!("getsockopt is not implemented");
        Ok(0)
    }
    /// # `listen`
    /// 监听socket，仅用于stream socket
    fn listen(&self, backlog: usize) -> Result<(), SystemError> {
        Err(SystemError::ENOSYS)
    }
    // poll
    // pselect
    /// # `read`
    fn read(&self, buffer: &mut [u8]) -> Result<usize, SystemError> {
        self.recv(buffer, PMSG::empty())
    }
    /// # `recv`
    /// 接收数据，`read` = `recv` with flags = 0
    fn recv(&self, buffer: &mut [u8], flags: PMSG) -> Result<usize, SystemError> {
        Err(SystemError::ENOSYS)
    }
    /// # `recv_from`
    fn recv_from(
        &self,
        buffer: &mut [u8],
        flags: PMSG,
        address: Option<Endpoint>,
    ) -> Result<(usize, Endpoint), SystemError> {
        Err(SystemError::ENOSYS)
    }
    // /// # `recv_msg`
    // fn recv_msg(&self, msg: &mut MsgHdr, flags: PMSG) -> Result<usize, SystemError> {
    //     Err(SystemError::ENOSYS)
    // }
    // select
    /// # `send`
    fn send(&self, buffer: &[u8], flags: PMSG) -> Result<usize, SystemError> {
        Err(SystemError::ENOSYS)
    }
    // /// # `send_msg`
    // fn send_msg(&self, msg: &MsgHdr, flags: PMSG) -> Result<usize, SystemError> {
    //     Err(SystemError::ENOSYS)
    // }
    /// # `send_to`
    fn send_to(&self, buffer: &[u8], flags: PMSG, address: Endpoint) -> Result<usize, SystemError> {
        Err(SystemError::ENOSYS)
    }
    /// # `set_option`
    /// Posix `setsockopt` ，设置socket选项
    /// ## Parameters
    /// - level 选项的层次
    /// - name 选项的名称
    /// - value 选项的值
    /// ## Reference
    /// https://code.dragonos.org.cn/s?refs=sk_setsockopt&project=linux-6.6.21
    fn set_option(&self, level: PSOL, name: usize, val: &[u8]) -> Result<(), SystemError> {
        log::warn!("setsockopt is not implemented");
        Ok(())
    }
    /// # `shutdown`
    fn shutdown(&self, how: ShutdownTemp) -> Result<(), SystemError> {
        // TODO 构建shutdown系统调用
        // set shutdown bit
        Err(SystemError::ENOSYS)
    }
    // sockatmark
    // socket
    // socketpair
    /// # `write`
    fn write(&self, buffer: &[u8]) -> Result<usize, SystemError> {
        self.send(buffer, PMSG::empty())
    }
    // fn write_buffer(&self, _buf: &[u8]) -> Result<usize, SystemError> {
    //     todo!()
    // }
}

pub trait Family {
    fn socket(stype: SOCK, protocol: u32) -> Result<Arc<dyn Socket>, SystemError>;
}
