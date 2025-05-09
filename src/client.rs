use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;

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
    let mut count = 0;
    loop {
        udp
                .send_to(&*(format!("{}", count).into_bytes()), (addr, port))
                .await
                .expect("cannot send message to socket");
        tokio::time::sleep(Duration::from_secs(1)).await;
        count += 1;
    }

    Ok(())
}
