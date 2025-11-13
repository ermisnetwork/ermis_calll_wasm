use std::{ str::FromStr, time::Duration };

use anyhow::Result;
use iroh::{
    Endpoint,
    EndpointAddr,
    RelayMap,
    RelayMode,
    RelayUrl,
    Watcher,
    endpoint::{ Builder, Connection, ConnectionType },
};
use n0_future::{StreamExt, task};
use flume::{ Receiver, Sender };

use tokio_util::{ bytes::Bytes, codec::{ FramedRead, FramedWrite, LengthDelimitedCodec } };

use base64::{ Engine, prelude::BASE64_STANDARD };
use futures::{ FutureExt, SinkExt, select };

const ALPN: &[u8] = b"ermis-call";

#[derive(Debug, Clone)]
pub struct ErmisCallEndpoint {
    endpoint: Endpoint,
    cur_connection: Option<Connection>,
    local_sender: Sender<Bytes>,
    local_receiver: Receiver<Bytes>,
    remote_sender: Sender<Bytes>,
    remote_receiver: Receiver<Bytes>,
}

impl ErmisCallEndpoint {
    pub async fn new(relay_urls: &[&str]) -> Result<Self> {
        let endpoint = Builder::empty(
            RelayMode::Custom(
                RelayMap::from_iter(relay_urls.iter().map(|url| RelayUrl::from_str(url).unwrap()))
            )
        )
            .alpns(vec![ALPN.to_vec()])
            .bind().await?;
        endpoint.online().await;
        let (local_sender, remote_receiver) = flume::unbounded();
        let (remote_sender, local_receiver) = flume::unbounded();
        Ok(Self {
            endpoint,
            cur_connection: None,
            local_sender,
            local_receiver,
            remote_sender,
            remote_receiver,
        })
    }

    //  pub fn new(relay_urls: &[&str], secret_key: Option<&[u8; 32]>) -> Result<Self> {
    //     // let runtime = Runtime::new()?;
    //     let runtime = tokio::runtime::Builder::new_multi_thread()
    //         .worker_threads(2)
    //         .enable_all()
    //         .build()?;
    //     let _ = runtime.enter();
    //     let secret_key = if let Some(key) = secret_key {
    //         SecretKey::from_bytes(key)
    //     } else {
    //         SecretKey::generate(&mut rand::rng())
    //     };
    //     let res: Result<Endpoint> = runtime.block_on(async move {
    //         let iroh_endpoint = Builder::empty(RelayMode::Custom(RelayMap::from_iter(
    //             relay_urls
    //                 .iter()
    //                 .map(|url| RelayUrl::from_str(url).unwrap()),
    //         )))
    //         .secret_key(secret_key)
    //         .alpns(vec![b"ermis-call".to_vec()])
    //         .bind()
    //         .await?;
    //         iroh_endpoint.online().await;
    //         println!("{:?}", iroh_endpoint.addr());
    //         Ok(iroh_endpoint)
    //     });
    //     let (local_sender, remote_receiver) = flume::unbounded();
    //     let (remote_sender, local_receiver) = flume::unbounded();
    //     Ok(Self {
    //         iroh_endpoint: res?,
    //         cur_connection: None,
    //         local_sender,
    //         local_receiver,
    //         remote_sender,
    //         remote_receiver,
    //         runtime,
    //     })
    // }

    pub fn connection_type(&self) -> Option<ConnectionType> {
        if let Some(conn) = self.cur_connection.as_ref() {
            let c = self.endpoint.conn_type(conn.remote_id().unwrap());
            return Some(c.unwrap().get());
        } else {
            None
        }
    }

    pub fn round_trip_time(&self) -> Option<Duration> {
        if let Some(conn) = self.cur_connection.as_ref() { Some(conn.rtt()) } else { None }
    }

    pub fn get_local_endpoint_addr(&self) -> Result<String> {
        let addr_bytes = bitcode::serialize(&self.endpoint.addr())?;
        let addr_str = base64::prelude::BASE64_STANDARD.encode(addr_bytes);
        Ok(addr_str)
    }

    pub async fn connect(&mut self, addr: &str) -> Result<()> {
        let endpoint = self.endpoint.clone();
        let addr_bytes = BASE64_STANDARD.decode(addr)?;
        let addr: EndpointAddr = bitcode::deserialize(&addr_bytes)?;
        println!("connecting to {:?}", addr);
        let conn = endpoint.connect(addr, ALPN).await?;

        println!("connected to {:?}", conn.remote_id());

        // let ( send, recv) = conn.open_bi().await?;
        // println!("opened bidi stream");
        // let mut framed_write = FramedWrite::new(send, LengthDelimitedCodec::new());
        // framed_write.send(Bytes::from("hello from client")).await?;

        // send.write_all(b"test send message").await?;
        self.cur_connection = Some(conn);



        // let send_task =task::spawn(async move {
        //     let mut framed_reader = FramedRead::new(recv, LengthDelimitedCodec::new());
        //     loop {
        //         select! {
        //     msg = framed_reader.next().fuse() => match msg {
        //         Some(Ok(msg)) => {
        //             println!("Received message: {:?}", msg);
        //     } 
        //         Some(Err(e)) => {
        //             println!("Error receiving message: {}", e);
        //             break;
        //         }
        //         None => {
        //             println!("Receiver closed");
        //             break;
        //         }
        //     }}
        //     }
        //     // while let Some(msg) = reader.next().await {
        //     //     match msg {
        //     //         Ok(msg) => {
        //     //             println!("Received message: {:?}", msg);
        //     //         }
        //     //         Err(e) => {
        //     //             println!("Error receiving message: {}", e);
        //     //             break;
        //     //         }
        //     //     }
        //     // }
        // });

        // send_task.await?;

        Ok(())
    }

    pub fn get_current_connection(&self) -> Option<Connection> {
        self.cur_connection.clone()
    }

    pub async fn accept_connection(&mut self) -> Result<()> {
        let endpoint = self.endpoint.clone();
        if let Some(incoming) = endpoint.accept().await {
            let conn = incoming.accept()?.await?;
            self.cur_connection = Some(conn);
        } else {
            anyhow::bail!("cannot accept");
        }

        Ok(())
    }

    pub async fn accept_bidi_stream(&mut self) -> Result<()> {
        let cur_connection = self.cur_connection.clone();

        let Some(conn) = &cur_connection else {
            anyhow::bail!("Error accepting stream: No Connection established")
        };

        let (send_stream, recv_stream) = conn.accept_bi().await?;
        let mut sender = FramedWrite::new(send_stream, LengthDelimitedCodec::new());
        let mut receiver = FramedRead::new(recv_stream, LengthDelimitedCodec::new());

        let remote_sender = self.remote_sender.clone();
        let remote_receiver = self.remote_receiver.clone();

        // wasm_bindgen_futures::spawn_local(async move {
        tokio::spawn(async move {
            loop {
                select! {
            msg = receiver.next().fuse() => match msg {
                Some(Ok(msg)) => {
                    if let Err(e) = remote_sender.send_async(msg.freeze()).await {
                        println!("Error sending message: {}", e);
                        break;
                    }
                }
                Some(Err(e)) => {
                    println!("Error receiving message: {}", e);
                    break;
                }
                None => {
                    println!("Receiver closed");
                    break;
                }
            },
            msg = remote_receiver.recv_async().fuse() => match msg {
                Ok(msg) => {
                    if let Err(e) = sender.send(msg).await {
                        println!("Error sending message: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    println!("Error receiving message: {}", e);
                    break;
                }
            }
        }
            }
        });
        Ok(())
    }

    pub async fn open_bidi_stream(&mut self) -> Result<()> {
        let cur_connection = self.cur_connection.clone();

        let Some(conn) = &cur_connection else {
            anyhow::bail!("Error opening stream: No Connection established")
        };
        // let (send_stream, recv_stream) = conn.accept_bi().await?;
        // let (send_stream, recv_stream) = conn.open_bi().await?;
        // println!("opened bidi stream");

        // let mut sender = FramedWrite::new(send_stream, LengthDelimitedCodec::new());
        // let mut receiver = FramedRead::new(recv_stream, LengthDelimitedCodec::new());

        let remote_sender = self.remote_sender.clone();
        let remote_receiver = self.remote_receiver.clone();
        let conn = conn.clone();

        wasm_bindgen_futures::spawn_local(async move {
             let (send_stream, recv_stream) = conn.open_bi().await.unwrap();
        println!("opened bidi stream");

        let mut sender = FramedWrite::new(send_stream, LengthDelimitedCodec::new());
        let mut receiver = FramedRead::new(recv_stream, LengthDelimitedCodec::new());
       
            loop {
                select! {
            msg = receiver.next().fuse() => match msg {
                Some(Ok(msg)) => {
                    if let Err(e) = remote_sender.send_async(msg.freeze()).await {
                        println!("Error sending message: {}", e);
                        continue;
                    }
                }
                Some(Err(e)) => {
                    println!("Error receiving message: {}", e);
                    break;
                }
                None => {
                    println!("Receiver closed");
                    break;
                }
            },
            msg = remote_receiver.recv_async().fuse() => match msg {
                Ok(msg) => {
                    if let Err(e) = sender.send(msg).await {
                        println!("Error sending message: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    println!("Error receiving message: {}", e);
                    break;
                }
            }
        }
            }
        });
        Ok(())
    }

    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        self.local_sender.send(Bytes::copy_from_slice(data))?;
        Ok(())
    }

    pub async fn async_send(&mut self, data: &[u8]) -> Result<()> {
        self.local_sender.send_async(Bytes::copy_from_slice(data)).await?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<Bytes> {
        let bytes = self.local_receiver.recv()?;
        Ok(bytes)
    }

    pub async fn async_recv(&mut self) -> Result<Bytes> {
        let bytes = self.local_receiver.recv_async().await?;
        Ok(bytes)
    }

    pub fn cur_packet_loss(&self) -> Option<f64> {
        if let Some(conn) = &self.cur_connection {
            let loss =
                (conn.stats().path.lost_packets as f64) / (conn.stats().path.sent_packets as f64);
            Some(loss)
        } else {
            None
        }
    }
}
