use linux_errnos::Errno;
use spin::Mutex;
use spin::RwLock;
use std::any::Any;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;

use crate::socket::inet::common::PortManager;
use crate::socket::inet::InetSocket;

pub mod tap;

pub trait Iface: Sync + Send + Debug + Any {
    /// # `common`
    /// 获取网卡的公共信息
    fn common(&self) -> &IfaceCommon;

    /// # `mac`
    /// 获取网卡的MAC地址
    fn mac(&self) -> smoltcp::wire::EthernetAddress;

    // /// # `name`
    // /// 获取网卡名
    // fn iface_name(&self) -> String;

    // /// # `nic_id`
    // /// 获取网卡id
    // fn nic_id(&self) -> usize {
    //     self.common().iface_id
    // }

    /// # `poll`
    /// 用于轮询接口的状态。
    /// ## 参数
    /// - `sockets` ：一个可变引用到 `smoltcp::iface::SocketSet`，表示要轮询的套接字集
    /// ## 返回值
    /// - 成功返回 `Ok(())`
    /// - 如果轮询失败，返回 `Err(Errno::EAGAIN_OR_EWOULDBLOCK)`，表示需要再次尝试或者操作会阻塞
    fn poll(&self);

    /// # `update_ip_addrs`
    /// 用于更新接口的 IP 地址
    /// ## 参数
    /// - `ip_addrs` ：一个包含 `smoltcp::wire::IpCidr` 的切片，表示要设置的 IP 地址和子网掩码
    /// ## 返回值
    /// - 如果 `ip_addrs` 的长度不为 1，返回 `Err(Errno::EINVAL)`，表示输入参数无效
    fn update_ip_addrs(&self, ip_addrs: &[smoltcp::wire::IpCidr]) -> Result<(), Errno> {
        self.common().update_ip_addrs(ip_addrs)
    }

    /// @brief 获取smoltcp的网卡接口类型
    #[inline(always)]
    fn smol_iface(&self) -> &Mutex<smoltcp::iface::Interface> {
        &self.common().smol_iface
    }
    // fn as_any_ref(&'static self) -> &'static dyn core::any::Any;

    /// # `sockets`
    /// 获取网卡的套接字集
    fn sockets(&self) -> &Mutex<smoltcp::iface::SocketSet<'static>> {
        &self.common().sockets
    }

    /// # `port_manager`
    /// 用于管理网卡的端口
    fn port_manager(&self) -> &PortManager {
        &self.common().port_manager
    }

    /// Get the raw file descriptor if this interface has one
    /// Returns None if this interface doesn't have a file descriptor
    fn raw_fd(&self) -> Option<std::os::unix::io::RawFd> {
        None
    }

    // fn addr_assign_type(&self) -> u8;

    // fn net_device_type(&self) -> u16;

    // fn net_state(&self) -> NetDeivceState;

    // fn set_net_state(&self, state: NetDeivceState);

    // fn operstate(&self) -> Operstate;

    // fn set_operstate(&self, state: Operstate);
}

/// 网络设备的公共数据
#[derive(Debug)]
pub struct NetDeviceCommonData {
    /// 表示网络接口的地址分配类型
    pub addr_assign_type: u8,
    /// 表示网络接口的类型
    pub net_device_type: u16,
    // /// 表示网络接口的状态
    // pub state: NetDeivceState,
    // /// 表示网络接口的操作状态
    // pub operstate: Operstate,
}

impl Default for NetDeviceCommonData {
    fn default() -> Self {
        Self {
            addr_assign_type: 0,
            net_device_type: 1,
            // state: NetDeivceState::empty(),
            // operstate: Operstate::IF_OPER_UNKNOWN,
        }
    }
}

// /// 将网络设备注册到sysfs中
// /// 参考：https://code.dragonos.org.cn/xref/linux-2.6.39/net/core/dev.c?fi=register_netdev#5373
// fn register_netdevice(dev: Arc<dyn Iface>) -> Result<(), Errno> {
//     // 在sysfs中注册设备
//     netdev_register_kobject(dev.clone())?;

//     // 标识网络设备在系统中存在
//     dev.set_net_state(NetDeivceState::__LINK_STATE_PRESENT);

//     return Ok(());
// }

pub struct IfaceCommon {
    iface_id: usize,
    smol_iface: Mutex<smoltcp::iface::Interface>,
    /// 存smoltcp网卡的套接字集
    sockets: Mutex<smoltcp::iface::SocketSet<'static>>,
    /// 存 kernel wrap smoltcp socket 的集合
    bounds: RwLock<Vec<Arc<dyn InetSocket>>>,
    /// 端口管理器
    port_manager: PortManager,
    /// 下次轮询的时间
    poll_at_ms: core::sync::atomic::AtomicU64,
    /// 默认网卡标识
    /// TODO: 此字段设置目的是解决对bind unspecified地址的分包问题，需要在inet实现多网卡监听或路由子系统实现后移除
    default_iface: bool,
}

impl fmt::Debug for IfaceCommon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IfaceCommon")
            .field("iface_id", &self.iface_id)
            .field("sockets", &self.sockets)
            // .field("bounds", &self.bounds)
            .field("port_manager", &self.port_manager)
            .field("poll_at_ms", &self.poll_at_ms)
            .finish()
    }
}

impl IfaceCommon {
    pub fn new(iface_id: usize, default_iface: bool, iface: smoltcp::iface::Interface) -> Self {
        IfaceCommon {
            iface_id,
            smol_iface: Mutex::new(iface),
            sockets: Mutex::new(smoltcp::iface::SocketSet::new(Vec::new())),
            bounds: RwLock::new(Vec::new()),
            port_manager: PortManager::new(),
            poll_at_ms: core::sync::atomic::AtomicU64::new(0),
            default_iface,
        }
    }

    pub fn poll<D>(&self, device: &mut D)
    where
        D: smoltcp::phy::Device + ?Sized,
    {
        let timestamp = std::time::Instant::now().into();
        let mut sockets = self.sockets.lock();
        let mut interface = self.smol_iface.lock();

        let (has_events, poll_at) = {
            (
                matches!(
                    interface.poll(timestamp, device, &mut sockets),
                    smoltcp::iface::PollResult::SocketStateChanged
                ),
                loop {
                    let poll_at = interface.poll_at(timestamp, &sockets);
                    let Some(instant) = poll_at else {
                        break poll_at;
                    };
                    if instant > timestamp {
                        break poll_at;
                    }
                },
            )
        };

        // drop sockets here to avoid deadlock
        drop(interface);
        drop(sockets);

        use core::sync::atomic::Ordering;
        if let Some(instant) = poll_at {
            let _old_instant = self.poll_at_ms.load(Ordering::Relaxed);
            let new_instant = instant.total_millis() as u64;
            self.poll_at_ms.store(new_instant, Ordering::Relaxed);

            // TODO: poll at
            // if old_instant == 0 || new_instant < old_instant {
            //     self.polling_wait_queue.wake_all();
            // }
        } else {
            self.poll_at_ms.store(0, Ordering::Relaxed);
        }

        self.bounds.read().iter().for_each(|bound_socket| {
            // incase our inet socket missed the event, we manually notify it each time we poll
            if has_events {
                bound_socket.on_iface_events();
                bound_socket.wait_queue().wakeup();
            }
        });

        // TODO: remove closed sockets
        // let closed_sockets = self
        //     .closing_sockets
        //     .lock_irq_disabled()
        //     .extract_if(|closing_socket| closing_socket.is_closed())
        //     .collect::<Vec<_>>();
        // drop(closed_sockets);
    }

    pub fn update_ip_addrs(&self, ip_addrs: &[smoltcp::wire::IpCidr]) -> Result<(), Errno> {
        if ip_addrs.len() != 1 {
            return Err(Errno::EINVAL);
        }

        self.smol_iface.lock().update_ip_addrs(|addrs| {
            let dest = addrs.iter_mut().next();

            if let Some(dest) = dest {
                *dest = ip_addrs[0];
            } else {
                addrs.push(ip_addrs[0]).expect("Push ipCidr failed: full");
            }
        });
        Ok(())
    }

    // 需要bounds储存具体的Inet Socket信息，以提供不同种类inet socket的事件分发
    pub fn bind_socket(&self, socket: Arc<dyn InetSocket>) {
        self.bounds.write().push(socket);
    }

    pub fn unbind_socket(&self, socket: Arc<dyn InetSocket>) {
        let mut bounds = self.bounds.write();
        if let Some(index) = bounds.iter().position(|s| Arc::ptr_eq(s, &socket)) {
            bounds.remove(index);
            log::debug!("unbind socket success");
        }
    }

    // TODO: 需要在inet实现多网卡监听或路由子系统实现后移除
    pub fn is_default_iface(&self) -> bool {
        self.default_iface
    }
}
