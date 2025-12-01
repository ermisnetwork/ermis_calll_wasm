use bytes::Bytes;
use parking_lot::Mutex;
use rand::Rng;
use raptorq::{Decoder, Encoder, EncodingPacket, ObjectTransmissionInformation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

use crate::node::ErmisCallEndpoint;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()));
}

#[wasm_bindgen]
pub struct ErmisCall {
    inner: Arc<Mutex<Option<ErmisCallEndpoint>>>,
}

#[wasm_bindgen]
impl ErmisCall {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    #[wasm_bindgen]
    pub async fn spawn(
        &self,
        relay_urls: JsValue,
        secret_key: Option<Vec<u8>>,
    ) -> Result<(), JsValue> {
        let urls: Vec<String> = serde_wasm_bindgen::from_value(relay_urls)
            .map_err(|e| JsValue::from_str(&format!("Invalid relay URLs: {}", e)))?;

        let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();

        // let array: [u8; 32] = secret_key.as_deref().map_or_else(
        //     || {
        //         let mut rng = rand::thread_rng();
        //         let mut random_bytes = [0u8; 32];
        //         let _ = rng.try_fill(&mut random_bytes);
        //         random_bytes
        //     },
        //     |v| {
        //         v.try_into()
        //             .map_err(|_| JsValue::from_str("Invalid length"))
        //             .unwrap()
        //     },
        // );

        let array: [u8; 32] = secret_key
            .as_deref()
            .and_then(|v| v.try_into().ok()) 
            .unwrap_or_else(|| {
                let mut rng = rand::thread_rng();
                let mut random_bytes = [0u8; 32];
                let _ = rng.try_fill(&mut random_bytes);
                random_bytes
            });

        let endpoint = ErmisCallEndpoint::new(&url_refs, Some(&array))
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to spawn: {}", e)))?;

        let mut inner = self.inner.lock();
        *inner = Some(endpoint);

        Ok(())
    }

    #[wasm_bindgen(js_name = getLocalEndpointAddr)]
    pub async fn get_local_endpoint_addr(&self) -> Result<String, JsValue> {
        let endpoint = self.inner.lock();

        endpoint
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
            .get_local_endpoint_addr()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to get address: {}", e)))
    }

    #[wasm_bindgen]
    pub async fn connect(&self, addr: &str) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.lock();
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone()
        };
        endpoint
            .connect(addr)
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to connect: {}", e)))?;

        self.inner.lock().replace(endpoint);

        console_log!("Connected to peer");
        Ok(())
    }

    pub fn close(&self) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.lock();
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone()
        };
        endpoint
            .close()
            .ok_or_else(|| JsValue::from_str("No active connection to close"))?;

        console_log!("Connection closed");
        Ok(())
    }

    #[wasm_bindgen(js_name = acceptConnection)]
    pub async fn accept_connection(&self) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.lock();
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone()
        };
        endpoint
            .accept_connection()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to accept connection: {}", e)))?;

        let conn = endpoint.get_current_connection();
        if let Some(c) = conn {
            console_log!("Accepted connection from {:?}", c.remote_node_id());
            self.inner.lock().replace(endpoint);
        } else {
            console_log!("No connection found after acceptance");
        }

        console_log!("Connection accepted");
        Ok(())
    }

    #[wasm_bindgen(js_name = acceptBidiStream)]
    pub async fn accept_bidi_stream(&self) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.lock();
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone()
        };
        endpoint
            .accept_bidi_stream()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to accept bidi stream: {}", e)))?;

        console_log!("Bidi stream accepted");
        Ok(())
    }

    #[wasm_bindgen(js_name = openBidiStream)]
    pub async fn open_bidi_stream(&self) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.lock();
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone()
        };
        endpoint
            .open_bidi_stream()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to open bidi stream: {}", e)))?;

        console_log!("Bidi stream opened");
        Ok(())
    }

    #[wasm_bindgen(js_name = asyncSend)]
    pub async fn async_send(&self, data: &[u8]) -> Result<(), JsValue> {
        let sender = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            endpoint.local_sender.clone()
        };

        sender
            .send_async(Bytes::copy_from_slice(data))
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to async send: {}", e)))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = asyncRecv)]
    pub async fn async_recv(&self) -> Result<Vec<u8>, JsValue> {
        let recv = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            endpoint.local_receiver.clone()
        };

        let bytes = recv
            .recv_async()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to async receive: {}", e)))?;

        Ok(bytes.to_vec())
    }

    #[wasm_bindgen(js_name = connectionType)]
    pub fn connection_type(&self) -> Option<String> {
        let endpoint = self.inner.lock();
        let endpoint = endpoint.as_ref()?;

        endpoint.connection_type().map(|ct| format!("{:?}", ct))
    }

    #[wasm_bindgen(js_name = roundTripTime)]
    pub fn round_trip_time(&self) -> Option<f64> {
        let endpoint = self.inner.lock();
        let endpoint = endpoint.as_ref()?;
        endpoint.round_trip_time().map(|d| d.as_secs_f64() * 1000.0)
    }

    #[wasm_bindgen(js_name = currentPacketLoss)]
    pub fn current_packet_loss(&self) -> Option<f64> {
        let endpoint = self.inner.lock();
        let endpoint = endpoint.as_ref()?;
        endpoint.cur_packet_loss()
    }

    #[wasm_bindgen(js_name = sendRaptorQ)]
    pub fn send_raptorq(&self, data: &[u8]) -> Result<(), JsValue> {
        let local_datagram_sender = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            endpoint.local_datagram_sender.clone()
        };
        let mtu = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            let conn = endpoint
                .cur_connection
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Connection not established"))?;

            conn.max_datagram_size().ok_or_else(|| {
                JsValue::from_str("Datagram not available (max_datagram_size = None)")
            })?
        };
        let repair_packets_per_block = (data.len() as f64 / (mtu - 100) as f64) * 0.1;
        let encoder = Encoder::with_defaults(data, mtu as u16 - 100);
        let _ = local_datagram_sender
            .send(Bytes::copy_from_slice(&encoder.get_config().serialize()))
            .map_err(|e| JsValue::from_str(&format!("Failed to send raptorq: {}", e)));
        for encoded_packet in encoder.get_encoded_packets(repair_packets_per_block.ceil() as u32) {
            let _ = local_datagram_sender
                .send(encoded_packet.serialize().into())
                .map_err(|e| JsValue::from_str(&format!("Failed to async send: {}", e)));
        }
        Ok(())
    }
    #[wasm_bindgen(js_name = asyncSendRaptorQ)]
    pub async fn async_send_raptorq(&self, data: &[u8]) -> Result<(), JsValue> {
        let local_datagram_sender = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            endpoint.local_datagram_sender.clone()
        };
        //  let mtu = self.inner.lock().as_ref().ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
        //             .cur_connection
        //             .as_ref()
        //             .unwrap()
        //             .max_datagram_size()
        //             .unwrap();

        let mtu = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            let conn = endpoint
                .cur_connection
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Connection not established"))?;

            conn.max_datagram_size().ok_or_else(|| {
                JsValue::from_str("Datagram not available (max_datagram_size = None)")
            })?
        };
        let repair_packets_per_block = (data.len() as f64 / (mtu - 100) as f64) * 0.1;
        let encoder = Encoder::with_defaults(data, mtu as u16 - 100);
        let _ = local_datagram_sender
            .send_async(Bytes::copy_from_slice(&encoder.get_config().serialize()))
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to send raptorq: {}", e)));
        for encoded_packet in encoder.get_encoded_packets(repair_packets_per_block.ceil() as u32) {
            let _ = local_datagram_sender
                .send_async(encoded_packet.serialize().into())
                .await
                .map_err(|e| JsValue::from_str(&format!("Failed to async send: {}", e)));
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = asyncRecvRaptorQ)]
    pub async fn async_recv_raptorq(&self) -> Result<Vec<u8>, JsValue> {
        let local_datagram_receiver = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            endpoint.local_datagram_receiver.clone()
        };
        let mut decoder = None;

        while let Ok(datagram) = local_datagram_receiver.recv_async().await {
            if datagram.len() == 12 {
                let slice: &[u8; 12] = match datagram[..12].try_into() {
                    Ok(s) => s,
                    Err(_) => {
                        return Err("Invalid RaptorQ header: expected 12 bytes".into());
                    }
                };
                let transmission_info = ObjectTransmissionInformation::deserialize(slice);
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
        Err(JsValue::from_str("No data received"))
    }

    #[wasm_bindgen(js_name = recvRaptorQ)]
    pub fn recv_raptorq(&self) -> Result<Vec<u8>, JsValue> {
        let local_datagram_receiver = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            endpoint.local_datagram_receiver.clone()
        };
        let mut decoder = None;

        while let Ok(datagram) = local_datagram_receiver.recv() {
            if datagram.len() == 12 {
                let slice: &[u8; 12] = match datagram[..12].try_into() {
                    Ok(s) => s,
                    Err(_) => {
                        return Err("Invalid RaptorQ header: expected 12 bytes".into());
                    }
                };
                let transmission_info = ObjectTransmissionInformation::deserialize(slice);
                decoder = Some(Decoder::new(transmission_info));
                continue;
            }
            if let Some(decoder) = &mut decoder {
                let encoding_packet = EncodingPacket::deserialize(&datagram);
                decoder.add_new_packet(encoding_packet);
                if let Some(res) = decoder.get_result() {
                    return Ok(res);
                }
            }
        }
        Err(JsValue::from_str("No data received"))
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct ConnectionStats {
    connection_type: Option<String>,
    pub rtt_ms: Option<f64>,
    pub packet_loss: Option<f64>,
}

#[wasm_bindgen]
impl ConnectionStats {
    #[wasm_bindgen(constructor)]
    pub fn new(
        connection_type: Option<String>,
        rtt_ms: Option<f64>,
        packet_loss: Option<f64>,
    ) -> ConnectionStats {
        ConnectionStats {
            connection_type,
            rtt_ms,
            packet_loss,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn connection_type(&self) -> Option<String> {
        self.connection_type.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_connection_type(&mut self, value: Option<String>) {
        self.connection_type = value;
    }
}

#[wasm_bindgen]
impl ErmisCall {
    #[wasm_bindgen(js_name = getStats)]
    pub async fn get_stats(&self) -> Result<JsValue, JsValue> {
        let endpoint = self.inner.lock();
        let endpoint = endpoint
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

        let stats = ConnectionStats::new(
            endpoint.connection_type().map(|ct| format!("{:?}", ct)),
            endpoint.round_trip_time().map(|d| d.as_secs_f64() * 1000.0),
            endpoint.cur_packet_loss(),
        );

        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats: {}", e)))
    }
}
