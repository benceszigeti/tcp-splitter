use async_std::io;
use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::channel;
use async_std::sync::Arc;
use async_std::task;
use std::net::Shutdown;

const BUFFER_LEN: usize = 65535;
const CHANNEL_LEN: usize = 1024;

type Sender = async_std::sync::Sender<Arc<Vec<u8>>>;
type Receiver = async_std::sync::Receiver<Arc<Vec<u8>>>;

pub async fn accept_loop(
    listen_addr: SocketAddr,
    target_addr: SocketAddr,
    observer_addrs: Vec<SocketAddr>,
) -> io::Result<()> {
    let target_addr = Arc::new(target_addr);
    let observer_addrs = Arc::new(observer_addrs);

    let listener = TcpListener::bind(listen_addr).await?;
    let mut incoming = listener.incoming();
    while let Some(client_stream) = incoming.next().await {
        if let Ok(client_stream) = client_stream {
            let target_addr = target_addr.clone();
            let observer_addrs = observer_addrs.clone();
            task::spawn(async move {
                let _ = proxy_loop(client_stream, target_addr, observer_addrs).await;
            });
        }
    }

    Ok(())
}

async fn proxy_loop(
    client_stream: TcpStream,
    target_addr: Arc<SocketAddr>,
    observer_addrs: Arc<Vec<SocketAddr>>,
) -> io::Result<()> {
    let target_stream = TcpStream::connect(*target_addr).await?;
    let target_stream = Arc::new(target_stream);
    task::spawn(async move {
        let mut channels: Vec<Sender> = Vec::with_capacity(observer_addrs.len() + 1);

        for observer_addr in observer_addrs.iter() {
            let (client_sender, observer_receiver) = channel(CHANNEL_LEN);
            channels.push(client_sender);
            let observer_addr = *observer_addr;
            task::spawn(async move {
                let _ = observer_loop(observer_addr, observer_receiver).await;
            });
        }

        let client_stream = Arc::new(client_stream);
        let (client_reader, client_writer) = (client_stream.clone(), client_stream);

        let (client_sender, target_receiver) = channel(CHANNEL_LEN);
        channels.push(client_sender);
        task::spawn(async move {
            let _ = target_loop(target_stream, target_receiver, client_writer).await;
        });
        task::spawn(async move {
            let _ = client_read_loop(client_reader, channels).await;
        });
    });
    Ok(())
}

async fn target_loop(
    target_stream: Arc<TcpStream>,
    client_broadcast_receiver: Receiver,
    client_stream: Arc<TcpStream>,
) -> io::Result<()> {
    let (target_reader, target_writer) = (target_stream.clone(), target_stream);
    task::spawn(async move {
        let _ = target_read_loop(target_reader, client_stream).await;
    });
    let mut target_writer = &*target_writer;
    while let Some(data) = client_broadcast_receiver.recv().await {
        target_writer.write_all(&data).await?;
    }
    target_writer.shutdown(Shutdown::Write)?;
    Ok(())
}

async fn observer_loop(
    observer_addr: SocketAddr,
    client_broadcast_receiver: Receiver,
) -> io::Result<()> {
    let mut observer_stream = TcpStream::connect(observer_addr).await?;
    while let Some(data) = client_broadcast_receiver.recv().await {
        observer_stream.write_all(&data).await?;
    }
    Ok(())
}

async fn client_read_loop(client_stream: Arc<TcpStream>, txs: Vec<Sender>) -> io::Result<()> {
    let mut client_stream = &*client_stream;
    let mut buf = vec![0u8; BUFFER_LEN];
    loop {
        let n = client_stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        let data = Arc::new(buf[..n].to_vec());
        for tx in txs.iter() {
            let tx = tx.clone();
            let data = data.clone();
            task::spawn(async move {
                tx.send(data.clone()).await;
            });
        }
    }
    Ok(())
}

async fn target_read_loop(
    target_stream: Arc<TcpStream>,
    client_stream: Arc<TcpStream>,
) -> io::Result<()> {
    let mut client_stream = &*client_stream;
    let mut target_stream = &*target_stream;
    let mut buf = vec![0u8; BUFFER_LEN];
    loop {
        let n = target_stream.read(&mut buf).await?;
        if n == 0 {
            client_stream.shutdown(Shutdown::Write)?;
            break;
        }
        client_stream.write_all(&buf[..n]).await?;
    }
    Ok(())
}
