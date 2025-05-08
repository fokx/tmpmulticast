use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, TcpListener};
use socket2::{Socket, Domain, Type, Protocol};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = 8848;
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    let addr = "224.0.0.48".parse().unwrap();
    socket.join_multicast_v4(&addr, &Ipv4Addr::UNSPECIFIED)?;
    socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into())?;
    let udp = Arc::new(tokio::net::UdpSocket::from_std(socket.into())?);

    loop {
        let udp_clone = Arc::clone(&udp);
        tokio::spawn(async move {
            let mut buf = [0; 64];
            loop {
                let recv_res = udp_clone.recv_from(&mut buf).await;
                let (count, remote_addr) = recv_res.expect("cannot receive from udp socket");
                if let Ok(parsed) = core::str::from_utf8(&buf[..count]) {
                    println!("{:?}: {:?}", remote_addr, parsed);
                }
            }
        });
    }

    Ok(())
}