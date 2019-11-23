use async_std::io;
use async_std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::channel;
use async_std::sync::Arc;
use async_std::task;

type Receiver = async_std::sync::Receiver<Arc<Vec<u8>>>;
type Sender = async_std::sync::Sender<Arc<Vec<u8>>>;

struct Addresses {
    target_addr: SocketAddr,
    tx_observer_addrs: Vec<SocketAddr>,
    rx_observer_addrs: Vec<SocketAddr>,
}

impl Addresses {
    fn new(
        target_addr: SocketAddr,
        tx_observer_addrs: Vec<SocketAddr>,
        rx_observer_addrs: Vec<SocketAddr>,
    ) -> Addresses {
        Addresses {
            target_addr,
            tx_observer_addrs,
            rx_observer_addrs,
        }
    }
}

struct Broadcaster {
    txs: Vec<Sender>,
}

impl Broadcaster {
    fn with_capacity(n: usize) -> Broadcaster {
        Broadcaster {
            txs: Vec::with_capacity(n + 1),
        }
    }

    fn new_receiver(&mut self) -> Receiver {
        let (sender, receiver) = channel(1024);
        self.txs.push(sender);
        receiver
    }

    fn write(&mut self, data: Vec<u8>) {
        let data = Arc::new(data);
        for tx in self.txs.iter() {
            let tx = tx.clone();
            let data = data.clone();
            task::spawn(async move {
                tx.send(data.clone()).await;
            });
        }
    }
}

pub async fn run(
    listen_addr: SocketAddr,
    target_addr: SocketAddr,
    tx_observer_addrs: Vec<SocketAddr>,
    rx_observer_addrs: Vec<SocketAddr>,
) -> io::Result<()> {
    let addrs = Arc::new(Addresses::new(
        target_addr,
        tx_observer_addrs,
        rx_observer_addrs,
    ));
    let listener = TcpListener::bind(listen_addr).await?;
    let mut incoming = listener.incoming();
    while let Some(client_stream) = incoming.next().await {
        if let Ok(client_stream) = client_stream {
            let addrs = addrs.clone();
            task::spawn(async move {
                handle_client(client_stream, addrs).await;
            });
        }
    }
    Ok(())
}

async fn handle_client(client_stream: TcpStream, addrs: Arc<Addresses>) {
    if let Ok(target_stream) = TcpStream::connect(addrs.target_addr).await {
        let mut client_tx_broadcaster = spawn_observer_write_loops(&addrs.tx_observer_addrs);
        let mut client_rx_broadcaster = spawn_observer_write_loops(&addrs.rx_observer_addrs);
        let target_receiver = client_tx_broadcaster.new_receiver();
        let client_receiver = client_rx_broadcaster.new_receiver();
        spawn_read_write_loop(target_stream, target_receiver, client_rx_broadcaster);
        spawn_read_write_loop(client_stream, client_receiver, client_tx_broadcaster);
    }
}

fn spawn_observer_write_loops(addrs: &[SocketAddr]) -> Broadcaster {
    let mut broadcaster = Broadcaster::with_capacity(addrs.len() + 1);
    for addr in addrs.iter() {
        let addr = *addr;
        let receiver = broadcaster.new_receiver();
        task::spawn(async move {
            if let Ok(stream) = TcpStream::connect(addr).await {
                let _ = write_loop(&stream, receiver).await;
            }
        });
    }
    broadcaster
}

fn spawn_read_write_loop(stream: TcpStream, rx: Receiver, broadcaster: Broadcaster) {
    let stream = Arc::new(stream);
    let (reader, writer) = (stream.clone(), stream);
    task::spawn(async move {
        let reader = &*reader;
        let _ = read_loop(reader, broadcaster).await;
        let _ = reader.shutdown(Shutdown::Read);
    });
    task::spawn(async move {
        let writer = &*writer;
        let _ = write_loop(&writer, rx).await;
        let _ = writer.shutdown(Shutdown::Write);
    });
}

async fn write_loop(mut stream: &TcpStream, rx: Receiver) -> io::Result<()> {
    while let Some(data) = rx.recv().await {
        stream.write_all(&data).await?;
    }
    Ok(())
}

async fn read_loop(mut stream: &TcpStream, mut broadcaster: Broadcaster) -> io::Result<()> {
    let mut buf = [0; 65535];
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        broadcaster.write(buf[..n].to_vec());
    }
    Ok(())
}
