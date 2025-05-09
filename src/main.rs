mod common;

use std::collections::HashMap;
use common::{create_udp_socket, Message};
use std::sync::Arc;
use crate::common::generate_fingerprint_plain;
use axum::{
    routing::{get, post},
    Router, Json, extract::Query,
};
use std::io::{self, Error, Write};
use axum::extract::Path;
// use std::io::prelude::*;
use futures::{Stream, TryStreamExt};
use tokio_util::io::StreamReader;
use tokio::{fs::File, io::BufWriter};
async fn register_handler(my_response: Arc<Message>, Json(payload): Json<Message>) -> Json<Message> {
    // Here you can process the payload as needed
    println!("axum register_handler received message: {:?}", payload);

    // Use my_response instead of creating a new response Message
    // Return the pre-defined response as JSON
    Json((*my_response).clone())
}

async fn prepare_upload_handler(Query(pin): Query<Option<String>>, Json(payload): Json<PrepareUploadRequest>) -> Json<HashMap<String, String>> {
    println!("Received request with pin: {:?}", pin);
    println!("Payload: {:?}", payload);

    // Generate session ID and file tokens
    let session_id = format!("mySessionId"); // Replace with actual session ID generation logic
    let mut files_tokens = HashMap::new();

    for (file_id, _) in &payload.files.files {
        let token = format!("token_for_{}", file_id); // Replace with actual token generation logic
        files_tokens.insert(file_id.clone(), token);
    }

    Json({
        let mut response = HashMap::new();
        response.insert("sessionId".to_string(), session_id);
        response.insert("files".to_string(), serde_json::to_value(files_tokens).unwrap().to_string());
        response
    })
}


async fn upload_handler(
    Query(query_params): Query<UploadQuery>,
    body: axum::body::Body,
) -> Json<Result<(), String>> {
    // Verify the session_id, file_id, and token for security
    if query_params.session_id != "mySessionId" || query_params.file_id != "file_id" || query_params.token != "someFileToken" {
        return Json(Err("Invalid session, fileId or token".to_string()));
    }
    let res = async {
        let path = format!("/tmp/{:?}", query_params.file_id);
        // Save binary data to the file
        let body_with_io_error = body.into_data_stream().map_err(io::Error::other);
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);
        // Create the file. `File` implements `AsyncWrite`.
        let mut file = BufWriter::new(File::create(path).await.unwrap());
        tokio::io::copy(&mut body_reader, &mut file).await.unwrap();
        Ok::<_, io::Error>(())
    }
            .await;
    match res {
        Ok(_) => {Json(Ok(()))}
        Err(_) => {Json(Err(String::from("Invalid session, fileId or token")))}
    }
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = 8848;
    let udp = create_udp_socket(port)?;
    let mut buf = [0; 1024];
    let my_fingerprint = generate_fingerprint_plain();
    let my_response = Arc::new(Message {
        alias: "example".to_string(),
        version: "1.0".to_string(),
        device_model: Some("model_x".to_string()),
        device_type: Some("type_y".to_string()),
        fingerprint: my_fingerprint.clone(),
        port,
        protocol: "UDP".to_string(),
        download: Some(true),
        announce: false,
    });
    let addr: std::net::Ipv4Addr = "224.0.0.48".parse().unwrap();
    let my_response_for_route = Arc::clone(&my_response);
    let my_response_for_announce = Arc::clone(&my_response);

    tokio::spawn(announce(my_response_for_announce));
    let app = Router::new()
            .route("/api/localsend/v2/register", post(move |Json(payload): Json<Message>| {
                register_handler(Arc::clone(&my_response_for_route), Json::from(payload))
            }))
            .route("/api/localsend/v2/prepare-upload", post(prepare_upload_handler))
            .route("/api/localsend/v2/upload", post(upload_handler))
            .route("/", get(|| async { "This is an axum server" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    loop {
        let (count, remote_addr) = udp.recv_from(&mut buf).await?;
        let data = buf[..count].to_vec();
        let udp_clone = Arc::clone(&udp);
        let response_clone = my_response.clone();
        let my_fingerprint_clone = my_fingerprint.clone();
        
        tokio::spawn(async move {
            if let Ok(parsed) = serde_json::from_slice::<Message>(&data) {
                if parsed.fingerprint == my_fingerprint_clone {
                    println!("skip my own fingerprint");
                    return;
                }
                println!("received multicast message from {:?}: {:?}", remote_addr, parsed);
                udp_clone.send_to(
                    &serde_json::to_vec(&*response_clone).expect("Failed to serialize Message"),
                    (addr, port),
                ).await.expect("Send error");
            } else {
                println!("Failed to parse message");
            }
        });
    }
}

async fn announce(my_response: Arc<Message>) -> std::io::Result<()> {
    let port = 8848;
    let udp = create_udp_socket(port)?;
    let addr: std::net::Ipv4Addr = "224.0.0.48".parse().unwrap();
    let mut count = 0;
    // loop {
    println!("{}", count);
    let response_clone = my_response.clone();
    udp.send_to(
        &serde_json::to_vec(&*my_response)
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

#[derive(serde::Deserialize, Debug)]
struct PrepareUploadRequest {
    info: Info,
    files: Files,
}

#[derive(serde::Deserialize, Debug)]
struct Info {
    alias: String,
    version: String, // protocol version (major.minor)
    device_model: Option<String>, // nullable
    device_type: Option<DeviceType>, // mobile | desktop | web | headless | server, nullable
    fingerprint: String, // ignored in HTTPS mode
    port: u16,
    protocol: Protocol,
    download: bool, // if download API (section 5.2, 5.3) is active (optional, default: false)
}

#[derive(serde::Deserialize, Debug)]
enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Headless,
    Server,
}

#[derive(serde::Deserialize, Debug)]
enum Protocol {
    Http,
    Https,
}

#[derive(serde::Deserialize, Debug)]
struct Files {
    // Use serde_json's custom key deserialization to handle dynamic file IDs
    #[serde(flatten)]
    files: std::collections::HashMap<String, UploadFile>,
}

#[derive(serde::Deserialize, Debug)]
struct UploadFile {
    id: String,
    file_name: String,
    size: u64, // bytes
    file_type: String,
    sha256: Option<String>, // nullable
    preview: Option<Vec<u8>>, // nullable
    metadata: Option<Metadata>,
}

#[derive(serde::Deserialize, Debug)]
struct Metadata {
    modified: Option<std::time::SystemTime>,
    accessed: Option<std::time::SystemTime>,
}

#[derive(serde::Deserialize, Debug)]
struct UploadQuery {
    session_id: String,
    file_id: String,
    token: String,
}

#[tokio::test]
async fn client_test() -> std::io::Result<()> {
    // cargo test -- --nocapture
    // https://stackoverflow.com/questions/25106554/why-doesnt-println-work-in-rust-unit-tests
    let my_fingerprint = generate_fingerprint_plain();
    println!("test client fingerprint : {}", my_fingerprint);
    let port = 8848;
    let my_response = Arc::new(Message {
        alias: "example".to_string(),
        version: "1.0".to_string(),
        device_model: Some("model_x".to_string()),
        device_type: Some("type_y".to_string()),
        fingerprint: my_fingerprint.clone(),
        port,
        protocol: "UDP".to_string(),
        download: Some(true),
        announce: false,
    });

    let my_response_for_announce = Arc::clone(&my_response);
    let my_response_clone = Arc::clone(&my_response);

    announce(my_response_for_announce).await?;
    // POST to "/api/localsend/v2/register"
    let client = reqwest::Client::new();
    let res = client.post(format!("http://127.0.0.1:8848/api/localsend/v2/register"))
            .json(&*my_response_clone)
            .send()
            .await;
    match res {
        Ok(response) => {
            println!("Response: {:?}", response);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
    Ok(())
}