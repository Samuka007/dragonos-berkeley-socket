use std::{io::{self, Read}, net::Ipv4Addr, sync::Arc};

use berkeley_socket::{
    driver::{irq::start_network_polling_thread, tap::TapDevice},
    interface::{tap::TapIface, Iface},
    posix::SOCK,
    socket::{
        endpoint::Endpoint,
        inet::{common::NET_DEVICES, syscall::Inet},
        Family,
    },
};
use smoltcp::wire::{IpAddress, IpCidr, IpEndpoint, Ipv4Cidr};
use spin::Mutex;

fn make_udp_echo() {
    let socket = Inet::socket(SOCK::Datagram, 0).unwrap();
    socket
        .bind(Endpoint::Ip(IpEndpoint::new(
            IpAddress::v4(192, 168, 213, 2),
            1234,
        )))
        .unwrap();
    socket
        .connect(Endpoint::Ip(IpEndpoint::new(
            IpAddress::v4(192, 168, 213, 1),
            12345,
        )))
        .unwrap();
    let mut buffer = [0u8; 1024];

    loop {
        let len = socket.read(&mut buffer).unwrap();
        log::info!(
            "Received {} bytes: {}",
            len,
            String::from_utf8_lossy(&buffer[..len])
        );
        let len = socket.write(&buffer[..len]).unwrap();
        log::info!(
            "Sent {} bytes: {}",
            len,
            String::from_utf8_lossy(&buffer[..len])
        );
    }
}

fn make_tcp_echo() {
    let socket = Inet::socket(SOCK::Stream, 0).unwrap();
    socket
        .bind(Endpoint::Ip(IpEndpoint::new(
            IpAddress::v4(192, 168, 213, 2),
            4321,
        )))
        .unwrap();
    socket.listen(1).unwrap();

    loop {
        let (client_socket, _) = socket.accept().unwrap();
        log::info!("Accepted connection from {:?}", client_socket);
        let mut buffer = [0u8; 1024];

        loop {
            let len = client_socket.read(&mut buffer).unwrap();
            if len == 0 {
                break;
            }
            log::info!(
                "Received {} bytes: {}",
                len,
                String::from_utf8_lossy(&buffer[..len])
            );
            let len = client_socket.write(&buffer[..len]).unwrap();
            log::info!(
                "Sent {} bytes: {}",
                len,
                String::from_utf8_lossy(&buffer[..len])
            );
        }
    }
}

fn make_request() {
    log::info!("Input a valid IP address and port to connect to:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let parts: Vec<&str> = input.trim().split(':').collect();
    if parts.len() != 2 {
        log::error!("Invalid input format. Use <IP>:<port>.");
        return;
    }
    let ip: Ipv4Addr = match parts[0].parse() {
        Ok(ip) => ip,
        Err(_) => {
            log::error!("Invalid IP address.");
            return;
        }
    };
    let port: u16 = match parts[1].parse() {
        Ok(port) => port,
        Err(_) => {
            log::error!("Invalid port number.");
            return;
        }
    };
    let endpoint = Endpoint::Ip(IpEndpoint::new(IpAddress::Ipv4(Ipv4Addr::from(ip)), port));

    let socket = Inet::socket(SOCK::Stream, 0).unwrap();
    match socket.connect(endpoint) {
        Ok(_) => {
            log::info!("Connected to {}:{}", ip, port);
            let mut buffer = [0u8; 1024];
            loop {
                let len = io::stdin().read(&mut buffer).unwrap();
                if len == 0 {
                    break; // EOF
                }
                let sent_len = socket.write(&buffer[..len]).unwrap();
                log::info!("Sent {} bytes", sent_len);
                match socket.read(&mut buffer) {
                    Ok(received_len) => {
                        if received_len == 0 {
                            log::info!("Socket closed by remote peer.");
                            break;
                        }
                        log::info!(
                            "Received {} bytes: {}",
                            received_len,
                            String::from_utf8_lossy(&buffer[..received_len])
                        );
                    }
                    Err(e) => {
                        log::error!("Socket read error: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to connect: {}", e);
        }
    }
    log::info!("Connection closed.");
}

fn main() {
    env_logger::init();
    let device = TapDevice::new("tap0", smoltcp::phy::Medium::Ethernet).unwrap();
    let iface_inner = TapIface::new(Arc::new(Mutex::new(device)));

    let ip_cidr = IpCidr::Ipv4(Ipv4Cidr::new(Ipv4Addr::new(192, 168, 213, 2), 24));

    let ip_cidr = vec![ip_cidr];

    iface_inner.update_ip_addrs(&ip_cidr).unwrap();

    let iface = Arc::new(iface_inner);

    NET_DEVICES.write().insert(0, iface);
    scopeguard::defer!({
        NET_DEVICES.write().clear();
    });
    let _ = start_network_polling_thread();

    let udp = std::thread::spawn(move || {
        make_udp_echo();
    });
    let tcp = std::thread::spawn(move || {
        make_tcp_echo();
    });

    loop {
        let char = io::stdin().bytes().next().unwrap().unwrap();
        match char {
            b'q' | b'Q' => {
                log::info!("Exiting...");
                break;
            }
            b'r' => {
                make_request();
            }
            _ => {
                log::info!("Press 'q' to exit.");
            }
        }
    }

    // Optionally join threads before exiting
    udp.join().unwrap();
    tcp.join().unwrap();
}
