use std::{io::Read, net::TcpStream};

const CLOUD_INIT: &str = r#"
#cloud-config

runcmd:
  - echo ":3" | nc -lp 1337
"#;

async fn try_server(ip: String) -> Result<bool, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect_timeout(
        &format!("{}:1337", ip).parse()?,
        std::time::Duration::from_secs(10),
    )?;
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf)?;

    Ok(n > 0)
}

#[tokio::main]
async fn main() {
    let token = std::env::var("HCLOUD_TOKEN").expect("expected HCLOUD_TOKEN env var");

    let server_config = hugehugemassive::ServerConfig {
        cloud_init: Some(CLOUD_INIT.to_string()),
        ..Default::default()
    };

    let client = hugehugemassive::Hetzner::new(token);

    let server = client
        .create_server(server_config, "hhm-spinup-test".to_string())
        .await
        .expect("failed to create server");
    let ip = server.public_net.ipv4.unwrap().ip.to_string();

    println!("created server: {}", ip.clone());

    println!("sleeping for 30 seconds");
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    // Let's try and connect to the IP/port and wait for a message
    let mut tries_left = 10;

    loop {
        if let Ok(true) = try_server(ip.clone()).await {
            println!("received message, deleting server...");
            break;
        }

        tries_left -= 1;
        println!("tries left: {}", tries_left);

        if tries_left == 0 {
            println!("failed to receive message, deleting server...");
            break;
        }
    }

    client
        .delete_server(server.id)
        .await
        .expect("failed to delete server");
}
