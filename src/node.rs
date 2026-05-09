use std::{str::FromStr, time::Duration};

use anyhow::{Result, anyhow};
use flume::{Receiver, Sender};
use iroh::{
    Endpoint, NodeAddr, RelayMap, RelayMode, RelayUrl, SecretKey,
    endpoint::{Connection, ConnectionType},
};
use iroh_quinn_proto::VarInt;
// use n0_future::StreamExt;
use futures::{StreamExt as _, sink::SinkExt};

use tokio_util::{
    bytes::Bytes,
    codec::{FramedRead, FramedWrite, LengthDelimitedCodec},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use wasm_bindgen_futures::spawn_local;

const ALPN: &[u8] = b"ermis-call";
const STREAM_KIND_AUDIO: u8 = 0x01;
const STREAM_KIND_VIDEO: u8 = 0x02;

fn spawn_uni_stream_receiver(conn: Connection, remote_frame_channel_sender: Sender<Bytes>) {
    spawn_local(async move {
        loop {
            match conn.accept_uni().await {
                Ok(mut stream) => {
                    let cl = remote_frame_channel_sender.clone();
                    spawn_local(async move {
                        let mut stream_kind = [0u8; 1];
                        if let Err(e) = stream.read_exact(&mut stream_kind).await {
                            println!("error reading stream kind: {}", e);
                            return;
                        }

                        match stream_kind[0] {
                            STREAM_KIND_AUDIO | STREAM_KIND_VIDEO => {
                                let mut frame_receiver =
                                    FramedRead::new(stream, LengthDelimitedCodec::new());
                                while let Some(frame) = frame_receiver.next().await {
                                    match frame {
                                        Ok(frame) => {
                                            if let Err(e) = cl.send(frame.into()) {
                                                println!("error sending frame to channel: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            println!("error reading frame from uni stream: {}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                            kind => {
                                println!("unknown uni stream kind: {}", kind);
                            }
                        }
                    });
                }
                Err(e) => {
                    println!("error accepting uni stream: {}", e);
                    break;
                }
            }
        }
    });
}

fn spawn_audio_stream_sender(conn: Connection, remote_audio_receiver: Receiver<Bytes>) {
    spawn_local(async move {
        match conn.open_uni().await {
            Ok(mut stream) => {
                if let Err(e) = stream.write_all(&[STREAM_KIND_AUDIO]).await {
                    println!("error writing audio stream kind: {}", e);
                    return;
                }

                let mut audio_sender = FramedWrite::new(stream, LengthDelimitedCodec::new());
                while let Ok(audio_frame) = remote_audio_receiver.recv_async().await {
                    if let Err(e) = audio_sender.send(audio_frame).await {
                        println!("error sending audio frame: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("error opening uni stream for audio: {}", e);
            }
        }
    });
}

#[derive(Clone)]
pub struct ErmisCallEndpoint {
    pub endpoint: Endpoint,
    pub cur_connection: Option<Connection>,
    pub local_sender: Sender<Bytes>,
    pub local_receiver: Receiver<Bytes>,
    pub remote_sender: Sender<Bytes>,
    pub remote_receiver: Receiver<Bytes>,
    pub local_audio_sender: Sender<Bytes>,
    pub remote_audio_receiver: Receiver<Bytes>,
    pub local_control_sender: Sender<Bytes>,
    pub remote_control_receiver: Receiver<Bytes>,
    pub new_gop_notifier: Sender<()>,
    pub lost_packets: u64,
    pub sent_packets: u64,
}

impl ErmisCallEndpoint {
    pub async fn new(relay_urls: &[&str], secret_key: Option<&[u8; 32]>) -> Result<Self> {
        let secret_key = if let Some(key) = secret_key {
            SecretKey::from_bytes(key)
        } else {
            let mut rng = rand::rngs::OsRng;
            SecretKey::generate(&mut rng)
        };
        let endpoint = Endpoint::builder()
            .relay_mode(RelayMode::Custom(RelayMap::from_iter(
                relay_urls
                    .iter()
                    .map(|url| RelayUrl::from_str(url).unwrap()),
            )))
            .secret_key(secret_key)
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await?;
        let (local_sender, remote_receiver) = flume::bounded(60);
        let (remote_sender, local_receiver) = flume::bounded(60);
        let (local_control_sender, remote_control_receiver) = flume::bounded(60);
        let (local_audio_sender, remote_audio_receiver) = flume::bounded(60);
        let (new_gop_notifier, _) = flume::bounded(1);
        Ok(Self {
            endpoint,
            cur_connection: None,
            local_sender,
            local_receiver,
            remote_sender,
            remote_receiver,
            local_control_sender,
            remote_control_receiver,
            local_audio_sender,
            remote_audio_receiver,
            new_gop_notifier,
            lost_packets: 0,
            sent_packets: 0,
        })
    }

    pub async fn close_endpoint(&self) -> Result<()> {
        self.endpoint.close().await;
        Ok(())
    }

    pub async fn get_local_endpoint_addr(&self) -> Result<String> {
        let addr_bytes = bitcode::serialize(&self.endpoint.node_addr().await?)?;
        let addr_str = base64::prelude::BASE64_STANDARD.encode(addr_bytes);
        Ok(addr_str)
    }

    pub fn close_connection(&mut self) -> Option<()> {
        self.cur_connection.take()?.close(VarInt::from_u32(0), &[0]);
        Some(())
    }

    pub fn network_change(&mut self) {
        let ep = self.endpoint.clone();
        spawn_local(async move {
            ep.network_change().await;
        });
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
        if let Some(conn) = self.cur_connection.as_ref() {
            Some(conn.rtt())
        } else {
            None
        }
    }

    pub fn get_current_connection(&self) -> Option<Connection> {
        self.cur_connection.clone()
    }

    pub async fn connect(&mut self, addr: &str) -> Result<()> {
        let endpoint = self.endpoint.clone();
        let addr_bytes = BASE64_STANDARD.decode(addr)?;
        let addr: NodeAddr = bitcode::deserialize(&addr_bytes)?;
        println!("connecting to {:?}", addr);
        let conn = endpoint.connect(addr, ALPN).await?;
        let (control_send_stream, control_recv_stream) = conn.open_bi().await?;
        let mut control_sender = FramedWrite::new(control_send_stream, LengthDelimitedCodec::new());
        let mut control_receiver =
            FramedRead::new(control_recv_stream, LengthDelimitedCodec::new());
        let remote_control_channel_receiver = self.remote_control_receiver.clone();
        let remote_frame_channel_sender = self.remote_sender.clone();
        let remote_audio_receiver = self.remote_audio_receiver.clone();

        spawn_uni_stream_receiver(conn.clone(), remote_frame_channel_sender.clone());
        spawn_audio_stream_sender(conn.clone(), remote_audio_receiver);

        spawn_local(async move {
            loop {
                tokio::select! {
                    // send control frames to local client
                    Some(Ok(control_frame)) = control_receiver.next() => {
                       if let Err(e) = remote_frame_channel_sender.send(control_frame.freeze()) {
                           println!("error sending control frame: {}", e);
                        }
                    }
                    // send control frames to remote client
                    Ok(control_frame) = remote_control_channel_receiver.recv_async() => {
                        println!("sending control frame");
                        if let Err(e) = control_sender.send(control_frame).await {
                            println!("error sending control frame: {}", e);
                        }
                    }
                }
            }
        });

        println!("connected to {:?}", conn.remote_node_id()?);

        self.cur_connection = Some(conn);

        Ok(())
    }

    pub async fn accept_connection(&mut self) -> Result<()> {
        let endpoint = self.endpoint.clone();
        if let Some(incoming) = endpoint.accept().await {
            let conn = incoming.accept()?.await?;
            let (control_send_stream, control_recv_stream) = conn.accept_bi().await?;
            let mut control_sender =
                FramedWrite::new(control_send_stream, LengthDelimitedCodec::new());
            let mut control_receiver =
                FramedRead::new(control_recv_stream, LengthDelimitedCodec::new());
            let remote_control_channel_receiver = self.remote_control_receiver.clone();
            let remote_frame_channel_sender = self.remote_sender.clone();
            let remote_audio_receiver = self.remote_audio_receiver.clone();

            spawn_uni_stream_receiver(conn.clone(), remote_frame_channel_sender.clone());
            spawn_audio_stream_sender(conn.clone(), remote_audio_receiver);

            spawn_local(async move {
                loop {
                    tokio::select! {
                        Some(Ok(control_frame)) = control_receiver.next() => {
                           if let Err(e) = remote_frame_channel_sender.send(control_frame.freeze()) {
                               println!("error sending control frame: {}", e);
                           }
                        }
                        Ok(control_frame) = remote_control_channel_receiver.recv_async() => {
                            if let Err(e) = control_sender.send(control_frame).await {
                                println!("error sending control frame: {}", e);
                            }
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

    pub fn begin_with_gop(&mut self, data: Vec<u8>) -> Result<()> {
        let conn = self
            .cur_connection
            .as_ref()
            .ok_or(anyhow!("no existing quic connection"))?
            .clone();
        let key_frame = data.into();
        let remote_frame_receiver = self.remote_receiver.clone();
        let _ = self.new_gop_notifier.send(());
        let (new_gop_notifier, new_gop_watcher) = flume::bounded(1);
        self.new_gop_notifier = new_gop_notifier;
        spawn_local(async move {
            if let Ok(mut stream) = conn.open_uni().await {
                if let Err(e) = stream.write_all(&[STREAM_KIND_VIDEO]).await {
                    println!("error writing video stream kind: {}", e);
                    return;
                }
                let mut frame_sender = FramedWrite::new(stream, LengthDelimitedCodec::new());
                if let Err(e) = frame_sender.send(key_frame).await {
                    println!("error sending key frame: {}", e);
                }
                if let Ok(next_frame) = remote_frame_receiver.recv_async().await {
                    if let Err(e) = frame_sender.send(next_frame).await {
                        println!("error sending next frame: {}", e);
                    }
                }
                loop {
                    tokio::select! {
                        Ok(()) = new_gop_watcher.recv_async() => {
                            if let Err(e) =  frame_sender.into_inner().reset(12u8.into()) {
                                println!("error resetting stream for new GOP: {}", e);
                            }
                            break;
                        }
                        Ok(frame) = remote_frame_receiver.recv_async() => {
                            if let Err(e) = frame_sender.send(frame).await {
                                println!("error sending frame: {}", e);
                            }
                        }
                    }
                }
            } else {
                println!("error opening uni stream for key frame");
            }
        });
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

    pub fn cur_packet_loss(&mut self) -> Option<f64> {
        if let Some(conn) = &self.cur_connection {
            let lost_packets_since_last_check = conn.stats().path.lost_packets - self.lost_packets;
            self.lost_packets = conn.stats().path.lost_packets;
            let sent_packets_since_last_check = conn.stats().path.sent_packets - self.sent_packets;
            self.sent_packets = conn.stats().path.sent_packets;
            let loss = lost_packets_since_last_check as f64 / sent_packets_since_last_check as f64;
            Some(loss)
        } else {
            None
        }
    }
}
