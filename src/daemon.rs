use std::net::{Ipv4Addr, SocketAddrV4};
use socket2::{Socket, Domain, Type, Protocol};
use std::sync::Arc;
use tokio::sync::Mutex;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // ensure firewall opens this port
    let port = 8848;
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    let addr = "224.0.0.48".parse().unwrap();
    socket.join_multicast_v4(&addr, &Ipv4Addr::UNSPECIFIED)?;
    socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into())?;
    let udp = Arc::new(tokio::net::UdpSocket::from_std(socket.into())?);

    let mut buf = [0; 64];
    loop {
        // let udp_clone = Arc::clone(&udp);
        let (count, remote_addr) = udp.recv_from(&mut buf).await?;
        let data = buf[..count].to_vec();

        tokio::spawn(async move {
            if let Ok(parsed) = core::str::from_utf8(&data) {
                println!("{:?}: {:?}", remote_addr, parsed);
            }
        });
    }
}