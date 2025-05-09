mod common;

use common::{create_udp_socket, Message};
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = 8848;
    let udp = create_udp_socket(port)?;
    let mut buf = [0; 1024];

    loop {
        let (count, remote_addr) = udp.recv_from(&mut buf).await?;
        let data = buf[..count].to_vec();

        tokio::spawn(async move {
            if let Ok(parsed) = serde_json::from_slice::<Message>(&data) {
                println!("{:?}: {:?}", remote_addr, parsed);
            }
        });
    }
}


#[tokio::test]
async fn client_test() -> std::io::Result<()> {
    // cargo test -- --nocapture
    // https://stackoverflow.com/questions/25106554/why-doesnt-println-work-in-rust-unit-tests
    let port = 8848;
    let udp = create_udp_socket(port)?;
    let addr: std::net::Ipv4Addr = "224.0.0.48".parse().unwrap();
    let mut count = 0;

    loop {
        println!("{}", count);
        udp.send_to(
            &serde_json::to_vec(&Message {
                alias: "example".to_string(),
                version: "1.0".to_string(),
                device_model: Some("model_x".to_string()),
                device_type: Some("type_y".to_string()),
                fingerprint: "unique_fingerprint".to_string(),
                port,
                protocol: "UDP".to_string(),
                download: Some(false),
                announce: true,
            })
                    .expect("Failed to serialize Message"),
            (addr, port),
        )
                .await
                .expect("cannot send message to socket");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        count += 1;
    }
}