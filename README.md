# DragonOS Network Subsystem Validation

## How to run

1. configure a bridge interface for testing
```bash
sudo brctl addbr br-tap0
sudo ip addr add 192.168.213.1/24 dev br-tap0
sudo ip link set dev br-tap0 up
```

2. build the binary and run it as root
```bash
cargo build
sudo ./target/debug/berkeley-socket
```

3. check if the `tap0` interface is created
```bash
ip addr show tap0
```

4. create a tap interface and add it to the bridge
```bash
sudo brctl addif br-tap0 tap0
```

5. bring up all interfaces
```bash
sudo ip link set dev tap0 up
```

If you want to stop the main program, just `Ctrl+C` it. The tap interface will be removed automatically but the bridge will remain. So the next time just run the program again, and repeat step 4 and 5 to bring up the tap interface. 

## How to test

An example could be:
```rust
use std::{
    io::{Read, Write},
    net::UdpSocket,
};

fn make_udp_test() -> std::io::Result<()> {
    // Create a UDP socket
    let socket = UdpSocket::bind("192.168.213.1:12345")?;

    // Message to send
    let message = "helloworld";

    // Target address (IP and port)
    let target_address = "192.168.213.2:1234";

    // Send the message
    socket.send_to(message.as_bytes(), target_address)?;

    println!("Sent '{}' to {}", message, target_address);

    let mut buf = [0; 1024]; // Buffer to receive data
    socket.recv(&mut buf)?;
    let received_message = String::from_utf8_lossy(&buf);
    println!("Received: {}", received_message);

    Ok(())
}

fn make_tcp_test() -> std::io::Result<()> {
    // Send a message
    let message = "helloworld";
    let target_address = "192.168.213.2:4321";
    let mut stream = std::net::TcpStream::connect(target_address)?;
    stream.write_all(message.as_bytes())?;
    let mut buf = [0; 1024]; // Buffer to receive data
    let _ = stream.read(&mut buf)?;
    let received_message = String::from_utf8_lossy(&buf);
    println!("Received: {}", received_message);

    Ok(())
}

fn main() -> std::io::Result<()> {
    // Call the function to create and use the UDP socket
    make_udp_test()?;
    // Call the function to create and use the TCP socket
    make_tcp_test()?;

    Ok(())
}
```

Change the socket used in the main program and DIY your own test.
