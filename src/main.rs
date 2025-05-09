mod common;

use common::{create_udp_socket, Message};
use std::sync::Arc;
use crate::common::generate_fingerprint_plain;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = 8848;
    let udp = create_udp_socket(port)?;
    let mut buf = [0; 1024];
    let my_fingerprint = generate_fingerprint_plain();
    let response = Message {
        alias: "example".to_string(),
        version: "1.0".to_string(),
        device_model: Some("model_x".to_string()),
        device_type: Some("type_y".to_string()),
        fingerprint: my_fingerprint.clone(),
        port,
        protocol: "UDP".to_string(),
        download: Some(true),
        announce: false,
    };
    let addr: std::net::Ipv4Addr = "224.0.0.48".parse().unwrap();

    loop {
        let (count, remote_addr) = udp.recv_from(&mut buf).await?;
        let data = buf[..count].to_vec();
        let udp_clone = Arc::clone(&udp);
        let response_clone = response.clone(); // Cloning here is crucial
        let my_fingerprint_clone = my_fingerprint.clone(); // Cloning here is crucial
        
        tokio::spawn(async move {
            if let Ok(parsed) = serde_json::from_slice::<Message>(&data) {
                if parsed.fingerprint == my_fingerprint_clone {
                    println!("skip my own fingerprint");
                    return;
                }
                println!("{:?}: {:?}", remote_addr, parsed);
                udp_clone.send_to(
                    &serde_json::to_vec(&response_clone).expect("Failed to serialize Message"),
                    (addr, port),
                ).await.expect("Send error");
            }
        });
    }
}

async fn announce() -> std::io::Result<()> {
    let port = 8848;
    let udp = create_udp_socket(port)?;
    let addr: std::net::Ipv4Addr = "224.0.0.48".parse().unwrap();
    let mut count = 0;
    let fingerprint = generate_fingerprint_plain();
    // loop {
    println!("{}", count);
    udp.send_to(
        &serde_json::to_vec(&Message {
            alias: format!("example {}", count).to_string(),
            version: "1.0".to_string(),
            device_model: Some("model_x".to_string()),
            device_type: Some("type_y".to_string()),
            fingerprint: fingerprint.to_string(),
            port,
            protocol: "UDP".to_string(),
            download: Some(true),
            announce: true,
        })
                .expect("Failed to serialize Message"),
        (addr, port),
    )
            .await
            .expect("cannot send message to socket");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    count += 1;
    // }
    Ok(())
}

#[tokio::test]
async fn client_test() -> std::io::Result<()> {
    // cargo test -- --nocapture
    // https://stackoverflow.com/questions/25106554/why-doesnt-println-work-in-rust-unit-tests
    announce().await?;
    Ok(())
}