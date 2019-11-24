use async_std::fs;
use async_std::io;
use async_std::net::{SocketAddr, ToSocketAddrs};
use async_std::sync::Arc;
use async_std::task;
use clap::{App, Arg};
use serde_derive::Deserialize;
use std::process::exit;

#[macro_use]
extern crate clap;

#[derive(Deserialize)]
struct Config {
    tcp_clone: Vec<TcpCloneConfig>,
}

#[derive(Deserialize)]
struct TcpCloneConfig {
    server: ServerConfig,
    target: TargetConfig,
    client_tx_observer: Option<Vec<ClientTxObserverConfig>>,
    client_rx_observer: Option<Vec<ClientRxObserverConfig>>,
}

#[derive(Deserialize)]
struct ServerConfig {
    listen_addr: SocketAddr,
}

#[derive(Deserialize)]
struct TargetConfig {
    addr: String,
}

type ClientTxObserverConfig = TargetConfig;
type ClientRxObserverConfig = TargetConfig;

async fn resolve_addrs(addrs: Vec<&String>) -> io::Result<Vec<SocketAddr>> {
    let mut tasks: Vec<task::JoinHandle<io::Result<SocketAddr>>> = Vec::with_capacity(addrs.len());

    let mut res = Vec::with_capacity(addrs.len());
    for addr in addrs.iter() {
        let addr = (*addr).clone();
        let task = task::spawn(async move { Ok(addr.to_socket_addrs().await?.next().unwrap()) });
        tasks.push(task);
    }

    for task in tasks {
        res.push(task.await?);
    }

    Ok(res)
}

async fn resolve_addr(addr: &str) -> io::Result<SocketAddr> {
    Ok(addr.to_socket_addrs().await?.next().unwrap())
}

async fn resolve_observer_addrs(
    observer_cfg: &[ClientTxObserverConfig],
) -> io::Result<Vec<SocketAddr>> {
    let mut addrs = Vec::new();
    for cfg in observer_cfg.iter() {
        addrs.push(&cfg.addr);
    }
    resolve_addrs(addrs).await
}

async fn spawn_tcp_clone_task(
    tcp_clone_cfg: TcpCloneConfig,
) -> task::JoinHandle<std::result::Result<(), std::io::Error>> {
    task::spawn(async move {
        let tcp_clone_cfg = Arc::new(tcp_clone_cfg);

        let cfg = tcp_clone_cfg.clone();
        let client_tx_observers = task::spawn(async move {
            if cfg.client_tx_observer.is_none() {
                return Ok(vec![]);
            }
            resolve_observer_addrs(&cfg.client_tx_observer.as_ref().unwrap()).await
        });

        let cfg = tcp_clone_cfg.clone();
        let client_rx_observers = task::spawn(async move {
            if cfg.client_rx_observer.is_none() {
                return Ok(vec![]);
            }
            resolve_observer_addrs(&cfg.client_rx_observer.as_ref().unwrap()).await
        });

        let cfg = tcp_clone_cfg.clone();
        let target_addr = task::spawn(async move { resolve_addr(&cfg.target.addr).await });

        let cfg = tcp_clone_cfg.clone();
        tcp_clone::run(
            cfg.server.listen_addr,
            target_addr.await?,
            client_tx_observers.await?,
            client_rx_observers.await?,
        )
        .await
    })
}

async fn run(cfg_path: &str) -> io::Result<()> {
    let cfg = fs::read_to_string(cfg_path).await?;
    let cfg: Config = toml::from_str(&cfg)?;
    let mut servers: Vec<task::JoinHandle<std::result::Result<(), std::io::Error>>> = vec![];
    for tcp_clone in cfg.tcp_clone {
        servers.push(spawn_tcp_clone_task(tcp_clone).await);
    }
    for server in servers {
        server.await?
    }
    Ok(())
}

fn main() {
    let cli = App::new(crate_name!())
        .version(&format!("v{}", crate_version!())[..])
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .default_value("tcp-clone.toml"),
        )
        .get_matches();
    let cfg_path = cli.value_of("config").unwrap();

    if let Err(err) = task::block_on(async {
        let cfg_path = &(*cfg_path);
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
