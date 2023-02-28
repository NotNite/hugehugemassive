use std::{io::Write, net::TcpStream};

const CLOUD_INIT: &str = r#"
#cloud-config

runcmd:
  - curl -fsSL https://tailscale.com/install.sh | sh
  # Exit nodes require IP forwarding
  - echo 1 > /proc/sys/net/ipv4/ip_forward
  - echo 1 > /proc/sys/net/ipv6/conf/all/forwarding
  - tailscale up --authkey=%AUTHKEY% --hostname hhm --advertise-exit-node
  # Establish a killswitch to quietly shut down
  - nc -lp 1337 | grep -qe "shutdown" && tailscale logout
"#;

#[tokio::main]
async fn main() {
    let token = std::env::var("HCLOUD_TOKEN").expect("expected HCLOUD_TOKEN env var");
    let auth_key = std::env::var("TS_AUTHKEY").expect("expected TS_AUTHKEY env var");

    let cloud_init = CLOUD_INIT.trim().replace("%AUTHKEY%", &auth_key);
    let server_config = hugehugemassive::ServerConfig {
        cloud_init: Some(cloud_init),
        ..Default::default()
    };

    let client = hugehugemassive::Hetzner::new(token);

    let server = client
        .create_server(server_config, "hhm-tailscale".to_string())
        .await
        .expect("failed to create server");

    println!("spun up server, press enter to delete");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // hit the killswitch
    let stream = TcpStream::connect_timeout(
        &format!("{}:1337", server.public_net.ipv4.unwrap().ip)
            .parse()
            .unwrap(),
        std::time::Duration::from_secs(10),
    );

    if let Ok(mut stream) = stream {
        if stream.write_all(b"shutdown").is_err() {
            println!("failed to send shutdown message, you will need to manually remove the node");
        }

        if stream.shutdown(std::net::Shutdown::Both).is_err() {
            println!("failed to shutdown stream, example might hang");
        }
    } else {
        println!("failed to connect to killswitch, you will need to manually remove the node");
    }

    client
        .delete_server(server.id)
        .await
        .expect("failed to delete server");
}
