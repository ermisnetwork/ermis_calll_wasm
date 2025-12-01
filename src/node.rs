use std::{ str::FromStr, time::Duration };

use anyhow::Result;
use iroh::{
    Endpoint,
    NodeAddr,
    RelayMap,
    RelayMode,
    RelayUrl,
    SecretKey,
    endpoint::{  Connection, ConnectionType },
};
use iroh_quinn_proto::VarInt;
use n0_future::StreamExt;
use flume::{ Receiver, Sender };

use tokio_util::{ bytes::Bytes, codec::{ FramedRead, FramedWrite, LengthDelimitedCodec } };

use base64::{ Engine, prelude::BASE64_STANDARD };
use futures::{ FutureExt, SinkExt, select };
use wasm_bindgen_futures::spawn_local;
use raptorq::{Decoder, Encoder, EncodingPacket, ObjectTransmissionInformation};

const ALPN: &[u8] = b"ermis-call";

pub struct StreamHandle {
    pub sender: Sender<Bytes>,
    pub receiver: Receiver<Bytes>,
}

#[derive(Debug, Clone)]
pub struct ErmisCallEndpoint {
    pub endpoint: Endpoint,
    pub cur_connection: Option<Connection>,
    pub local_sender: Sender<Bytes>,
    pub local_receiver: Receiver<Bytes>,
    pub remote_sender: Sender<Bytes>,
    pub remote_receiver: Receiver<Bytes>,
    pub local_datagram_sender: Sender<Bytes>,
    pub local_datagram_receiver: Receiver<Bytes>,
    pub remote_datagram_sender: Sender<Bytes>,
    pub remote_datagram_receiver: Receiver<Bytes>,
}

impl ErmisCallEndpoint {
    pub async fn new(relay_urls: &[&str], secret_key: Option<&[u8; 32]>) -> Result<Self> {
        let secret_key = if let Some(key) = secret_key {
            SecretKey::from_bytes(key)
        } else {
            let mut rng = rand::rngs::OsRng;
            SecretKey::generate(&mut rng)
        };
        let endpoint = Endpoint::builder().relay_mode(RelayMode::Custom(RelayMap::from_iter(
                relay_urls
                    .iter()
                    .map(|url| RelayUrl::from_str(url).unwrap()),
            )))
            .secret_key(secret_key)
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await?;
        let (local_sender, remote_receiver) = flume::unbounded();
        let (remote_sender, local_receiver) = flume::unbounded();
        let (remote_datagram_sender, local_datagram_receiver) = flume::unbounded();
        let (local_datagram_sender, remote_datagram_receiver) = flume::unbounded();
        Ok(Self {
            endpoint,
            cur_connection: None,
            local_sender,
            local_receiver,
            remote_sender,
            remote_receiver,
            local_datagram_sender,
            local_datagram_receiver,
            remote_datagram_sender,
            remote_datagram_receiver,
        })
    }

   



    pub fn connection_type(&self) -> Option<ConnectionType> {
        if let Some(conn) = self.cur_connection.as_ref() {
            let c = self.endpoint.conn_type(conn.remote_node_id().unwrap());
            return Some(c.unwrap().get().unwrap());
        } else {
            None
        }
    }

    pub fn round_trip_time(&self) -> Option<Duration> {
        if let Some(conn) = self.cur_connection.as_ref() { Some(conn.rtt()) } else { None }
    }

    pub async fn get_local_endpoint_addr(&self) -> Result<String> {
        let addr_bytes = bitcode::serialize(&self.endpoint.node_addr().await?)?;
        let addr_str = base64::prelude::BASE64_STANDARD.encode(addr_bytes);
        Ok(addr_str)
    }

    pub async fn connect(&mut self, addr: &str) -> Result<()> {
        let endpoint = self.endpoint.clone();
        let addr_bytes = BASE64_STANDARD.decode(addr)?;
        let addr: NodeAddr = bitcode::deserialize(&addr_bytes)?;
        println!("connecting to {:?}", addr);
        let conn = endpoint.connect(addr, ALPN).await?;
         let remote_datagram_sender = self.remote_datagram_sender.clone();
        let remote_datagram_receiver = self.remote_datagram_receiver.clone();
        let remote_sender = self.remote_sender.clone();
        let conn_clone = conn.clone();
        spawn_local(async move {
            let mut decoder = None;
            loop {
                tokio::select! {
                    Ok(data) = conn_clone.read_datagram().fuse() => {
                       let _ = remote_datagram_sender.send(data.clone());
                        if data.len() == 12 {
                            let transmission_info =
                                ObjectTransmissionInformation::deserialize(&data[..12].try_into().unwrap());
                            decoder = Some(Decoder::new(transmission_info));
                            continue;
                        }
                        if let Some(dcd) = &mut decoder {
                            let encoding_packet = EncodingPacket::deserialize(&data);
                            dcd.add_new_packet(encoding_packet);
                            if let Some(res) = dcd.get_result() {
                                let _ = remote_sender.send(res.into());
                                decoder = None;
                            }
                        }
                    },
                    Ok(data) = remote_datagram_receiver.recv_async().fuse() => {
                        let _ = conn_clone.send_datagram(data);
                    }
                }
            }
        });

        println!("connected to {:?}", conn.remote_node_id()?);

        self.cur_connection = Some(conn);

        Ok(())
    }

     pub fn close(&mut self) -> Option<()> {
        self.cur_connection.take()?.close(VarInt::from_u32(0), &[0]);
        Some(())
    }


    pub fn get_current_connection(&self) -> Option<Connection> {
        self.cur_connection.clone()
    }

    pub async fn accept_connection(&mut self) -> Result<()> {
        let endpoint = self.endpoint.clone();
        if let Some(incoming) = endpoint.accept().await {
            let conn = incoming.accept()?.await?;
            let remote_datagram_sender = self.remote_datagram_sender.clone();
        let remote_datagram_receiver = self.remote_datagram_receiver.clone();
        let remote_sender = self.remote_sender.clone();
        let conn_clone = conn.clone();
            spawn_local(async move {
            let mut decoder = None;
            loop {
                tokio::select! {
                    Ok(data) = conn_clone.read_datagram().fuse() => {
                        let _ = remote_datagram_sender.send(data.clone());
                        if data.len() == 12 {
                            let transmission_info =
                                ObjectTransmissionInformation::deserialize(&data[..12].try_into().unwrap());
                            decoder = Some(Decoder::new(transmission_info));
                            continue;
                        }
                        if let Some(dcd) = &mut decoder {
                            let encoding_packet = EncodingPacket::deserialize(&data);
                            dcd.add_new_packet(encoding_packet);
                            if let Some(res) = dcd.get_result() {
                                let _ = remote_sender.send(res.into());
                                decoder = None;
                            }
                        }
                    },
                    Ok(data) = remote_datagram_receiver.recv_async().fuse() => {
                        let _ = conn_clone.send_datagram(data);
                    }
                }
            }
        });
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
        let remote_sender = self.remote_sender.clone();
        let remote_receiver = self.remote_receiver.clone();
        let conn = conn.clone();

        wasm_bindgen_futures::spawn_local(async move {
            // tokio::spawn(async move {
            println!("accepted bidi stream");
            let (send_stream, recv_stream) = conn.accept_bi().await.unwrap();
            let mut sender = FramedWrite::new(send_stream, LengthDelimitedCodec::new());
            let mut receiver = FramedRead::new(recv_stream, LengthDelimitedCodec::new());

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

        let remote_sender = self.remote_sender.clone();
        let remote_receiver = self.remote_receiver.clone();
        let conn = conn.clone();

        wasm_bindgen_futures::spawn_local(async move {
            println!("opened bidi stream");

            let (send_stream, recv_stream) = conn.open_bi().await.unwrap();
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

    pub async fn async_send_raptorq(&self, data: &[u8]) -> Result<()> {
        let mtu = self
            .cur_connection
            .as_ref()
            .unwrap()
            .max_datagram_size()
            .unwrap();
        let repair_packets_per_block = (data.len() as f64 / (mtu - 100) as f64) * 0.1;
        let encoder = Encoder::with_defaults(data, mtu as u16 - 100);
        self.local_datagram_sender
            .send(Bytes::copy_from_slice(&encoder.get_config().serialize()))?;
        for encoded_packet in encoder.get_encoded_packets(repair_packets_per_block.ceil() as u32) {
            self.local_datagram_sender
                .send_async(encoded_packet.serialize().into())
                .await?;
        }
        Ok(())
    }

    pub fn send_raptorq(&self, data: &[u8]) -> Result<()> {
        let mtu = self
            .cur_connection
            .as_ref()
            .unwrap()
            .max_datagram_size()
            .unwrap();
        let repair_packets_per_block = (data.len() as f64 / (mtu - 100) as f64) * 0.1;
        println!("{}", repair_packets_per_block.ceil() as u32);
        let encoder = Encoder::with_defaults(data, mtu as u16 - 100);
        self.local_datagram_sender
            .send(Bytes::copy_from_slice(&encoder.get_config().serialize()))?;
        for encoded_packet in encoder.get_encoded_packets(repair_packets_per_block.ceil() as u32) {
            self.local_datagram_sender
                .send(encoded_packet.serialize().into())?;
        }
        Ok(())
    }

    pub async fn async_recv_raptorq(&self) -> Result<Bytes> {
        let mut decoder = None;
        while let Ok(datagram) = self.local_datagram_receiver.recv_async().await {
            if datagram.len() == 12 {
                let transmission_info =
                    ObjectTransmissionInformation::deserialize(&datagram[..12].try_into().unwrap());
                decoder = Some(Decoder::new(transmission_info));
                continue;
            }
            if let Some(decoder) = &mut decoder {
                let encoding_packet = EncodingPacket::deserialize(&datagram);
                decoder.add_new_packet(encoding_packet);
                if let Some(res) = decoder.get_result() {
                    return Ok(res.into());
                }
            }
        }
        Err(anyhow::anyhow!("No data received"))
    }

    pub fn recv_raptorq(&self) -> Result<Bytes> {
        let mut decoder = None;
        while let Ok(datagram) = self.local_datagram_receiver.recv() {
            if datagram.len() == 12 {
                let transmission_info =
                    ObjectTransmissionInformation::deserialize(&datagram[..12].try_into().unwrap());
                decoder = Some(Decoder::new(transmission_info));
                continue;
            }
            if let Some(decoder) = &mut decoder {
                let encoding_packet = EncodingPacket::deserialize(&datagram);
                decoder.add_new_packet(encoding_packet);
                if let Some(res) = decoder.get_result() {
                    return Ok(res.into());
                }
            }
        }
        Err(anyhow::anyhow!("No data received"))
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
