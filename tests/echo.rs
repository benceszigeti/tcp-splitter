use async_std::task;
use std::io::prelude::*;
use std::io::Write;
use std::net::TcpListener;
use std::net::{Shutdown, TcpStream};
use std::thread;
use std::time::Duration;

fn echo_server(addr: &str) -> String {
    let server = TcpListener::bind(addr).unwrap();
    let mut stream = server.incoming().next().unwrap().unwrap();
    let mut res = String::new();
    stream.read_to_string(&mut res).unwrap();
    stream.write_all(res.as_bytes()).unwrap();
    res
}

fn echo_client(addr: &str, expected: &str) -> String {
    thread::sleep(Duration::from_secs(1));
    let mut client = TcpStream::connect(addr).unwrap();
    client.write_all(expected.as_bytes()).unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut res = String::new();
    client.read_to_string(&mut res).unwrap();
    res
}

fn run_tcp_clone(
    listen_addr: &'static str,
    target_addr: &'static str,
    client_tx_observer_addrs: Vec<&'static str>,
    client_rx_observer_addrs: Vec<&'static str>,
) {
    thread::spawn(move || {
        task::block_on(async move {
            let _ = tcp_clone::run(
                listen_addr.parse().unwrap(),
                target_addr.parse().unwrap(),
                client_tx_observer_addrs
                    .iter()
                    .map(|addr| addr.parse().unwrap())
                    .collect(),
                client_rx_observer_addrs
                    .iter()
                    .map(|addr| addr.parse().unwrap())
                    .collect(),
            )
            .await;
        })
    });
}

#[test]
fn echo_server_client() {
    thread::spawn(|| assert_eq!("hello", echo_server("127.0.0.1:2000")));
    assert_eq!("hello", echo_client("127.0.0.1:2000", "hello"));
}

#[test]
fn echo_server_proxy() {
    run_tcp_clone("127.0.0.1:1111", "127.0.0.1:2001", vec![], vec![]);
    thread::spawn(|| assert_eq!("hello", echo_server("127.0.0.1:2001")));
    assert_eq!("hello", echo_client("127.0.0.1:1111", "hello"));
}

#[test]
fn echo_server_proxy_with_observers() {
    run_tcp_clone(
        "127.0.0.1:2222",
        "127.0.0.1:2002",
        vec!["127.0.0.1:3002", "127.0.0.1:4002"],
        vec![],
    );
    thread::spawn(|| assert_eq!("hello", echo_server("127.0.0.1:2002")));
    thread::spawn(|| assert_eq!("hello", echo_server("127.0.0.1:3002")));
    thread::spawn(|| assert_eq!("hello", echo_server("127.0.0.1:4002")));
    assert_eq!("hello", echo_client("127.0.0.1:2222", "hello"));
}
