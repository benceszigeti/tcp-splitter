use async_std::fs;
use async_std::io;
use async_std::net::SocketAddr;
use async_std::task;
use serde_derive::Deserialize;
use std::env::args;
use std::process::exit;

#[derive(Deserialize, Clone)]
struct Config {
    tcp_clone: Vec<TcpCloneConfig>,
}

#[derive(Deserialize, Clone)]
struct TcpCloneConfig {
    server: ServerConfig,
    target: TargetConfig,
    observer: Vec<ObserverConfig>,
}

#[derive(Deserialize, Clone)]
struct ServerConfig {
    listen_addr: SocketAddr,
}

#[derive(Deserialize, Clone)]
struct TargetConfig {
    addr: SocketAddr,
}

#[derive(Deserialize, Clone)]
struct ObserverConfig {
    addr: SocketAddr,
}

async fn tcp_clone_task(
    tcp_clone_cfg: TcpCloneConfig,
) -> task::JoinHandle<std::result::Result<(), std::io::Error>> {
    task::spawn(async move {
        tcp_clone::accept_loop(
            tcp_clone_cfg.server.listen_addr,
            tcp_clone_cfg.target.addr,
            tcp_clone_cfg.observer.iter().map(|o| o.addr).collect(),
        )
        .await
    })
}

async fn run(cfg_path: &str) -> io::Result<()> {
    let cfg = fs::read_to_string(cfg_path).await?;
    let cfg: Config = toml::from_str(&cfg)?;
    let mut servers: Vec<task::JoinHandle<std::result::Result<(), std::io::Error>>> = vec![];
    for tcp_clone in cfg.tcp_clone {
        servers.push(tcp_clone_task(tcp_clone.clone()).await);
    }
    for server in servers {
        server.await?
    }
    Ok(())
}

fn main() {
    let cfg_path = args()
        .nth(1)
        .unwrap_or_else(|| "tcp-clone.toml".to_string());
    if let Err(err) = task::block_on(async {
        let cfg_path = cfg_path.clone();
        run(&cfg_path).await
    }) {
        use async_std::io::ErrorKind::{AddrInUse, InvalidData, NotFound};
        eprint!("error: ");
        if err.kind() == AddrInUse {
            eprintln!("address in use.");
        } else if err.kind() == NotFound {
            eprintln!("`{}` not found.", cfg_path);
        } else if err.kind() == InvalidData {
            eprintln!("invalid config.");
        } else {
            eprintln!("unknown error.");
        }
        eprintln!("details: '{}'", err);
        exit(1);
    }
    exit(0);
}
