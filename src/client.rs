use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Message {
    pub alias: String,
    pub version: String,
    pub device_model: Option<String>,
    pub device_type: Option<String>,
    pub fingerprint: String,
    pub port: u16,
    pub protocol: String,
    pub download: Option<bool>,
    pub announce: bool,
}

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
        println!("{}", count);
        udp
                .send_to(&serde_json::to_vec(&Message {
                    alias: "example".to_string(),
                    version: "1.0".to_string(),
                    device_model: Some("model_x".to_string()),
                    device_type: Some("type_y".to_string()),
                    fingerprint: "unique_fingerprint".to_string(),
                    port,
                    protocol: "UDP".to_string(),
                    download: Some(false),
                    announce: true,
                }).expect("Failed to serialize Message"), (addr, port))
                .await
                .expect("cannot send message to socket");
        tokio::time::sleep(Duration::from_secs(1)).await;
        count += 1;
    }

    Ok(())
}
