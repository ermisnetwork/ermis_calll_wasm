use bytes::Bytes;
use flume::{Receiver, Sender};
use parking_lot::Mutex;
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
    local_sender: Option<Sender<Bytes>>,
    local_receiver: Option<Receiver<Bytes>>,
    local_control_sender: Option<Sender<Bytes>>,
    new_gop_notifier: Option<Sender<()>>,
}

#[wasm_bindgen]
impl ErmisCall {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            local_sender: None,
            local_receiver: None,
            local_control_sender: None,
            new_gop_notifier: None,
        }
    }

    #[wasm_bindgen]
    pub async fn spawn(
        &mut self,
        relay_urls: JsValue,
        secret_key: Option<Vec<u8>>,
    ) -> Result<(), JsValue> {
        let urls: Vec<String> = serde_wasm_bindgen::from_value(relay_urls)
            .map_err(|e| JsValue::from_str(&format!("Invalid relay URLs: {}", e)))?;

        let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();

        let array: Option<[u8; 32]> = secret_key
            .as_deref()
            .and_then(|v| v.try_into().ok());

        let endpoint = ErmisCallEndpoint::new(&url_refs, array.as_ref()).await
            .map_err(|e| JsValue::from_str(&format!("Failed to spawn: {}", e)))?;

   
        self.local_sender = Some(endpoint.local_sender.clone());
        self.local_receiver = Some(endpoint.local_receiver.clone());
        self.local_control_sender = Some(endpoint.local_control_sender.clone());
        self.new_gop_notifier = Some(endpoint.new_gop_notifier.clone());

        let mut inner = self.inner.lock();
        *inner = Some(endpoint);

        Ok(())
    }

    #[wasm_bindgen(js_name = getLocalEndpointAddr)]
    pub async  fn get_local_endpoint_addr(&self) -> Result<String, JsValue> {
        let endpoint = self.inner.lock();

        endpoint
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
            .get_local_endpoint_addr().await
            .map_err(|e| JsValue::from_str(&format!("Failed to get address: {}", e)))
    }

    #[wasm_bindgen]
    pub async fn connect(&self, addr: &str) -> Result<(), JsValue> {
        let mut endpoint = self.inner.lock();
        let ep = endpoint
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

        ep.connect(addr).await
            .map_err(|e| JsValue::from_str(&format!("Failed to connect: {}", e)))?;

        console_log!("Connected to peer");
        Ok(())
    }

    #[wasm_bindgen(js_name = closeConnection)]
    pub fn close_connection(&self) -> Result<(), JsValue> {
        let mut endpoint = self.inner.lock();
        let ep = endpoint
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

        ep.close_connection()
            .ok_or_else(|| JsValue::from_str("No active connection to close"))?;

        console_log!("Connection closed");
        Ok(())
    }

    #[wasm_bindgen(js_name = acceptConnection)]
    pub async fn accept_connection(&self) -> Result<(), JsValue> {
        let mut endpoint = self.inner.lock();
        let ep = endpoint
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

        ep.accept_connection().await
            .map_err(|e| JsValue::from_str(&format!("Failed to accept connection: {}", e)))?;

        console_log!("Connection accepted");
        Ok(())
    }

    // ============================================
    // FAST PATH - NO LOCK - ZERO COPY
    // ============================================

    #[wasm_bindgen(js_name = sendControlFrame)]
    pub fn send_control_frame(&self, data: &[u8]) -> Result<(), JsValue> {
        let sender = self.local_control_sender
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local control sender not available"))?;

        sender
            .send(Bytes::copy_from_slice(data))
            .map_err(|e| JsValue::from_str(&format!("Failed to send control frame: {}", e)))
    }

    #[wasm_bindgen(js_name = sendDeltaFrame)]
    pub fn send_delta_frame(&self, data: &[u8]) -> Result<(), JsValue> {
        let sender = self.local_sender
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local sender not available"))?;

        sender
            .send(Bytes::copy_from_slice(data))
            .map_err(|e| JsValue::from_str(&format!("Failed to send delta frame: {}", e)))
    }

    #[wasm_bindgen(js_name = sendAudioFrame)]
    pub fn send_audio_frame(&self, data: &[u8]) -> Result<(), JsValue> {
        let sender = self.local_sender
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local sender not available"))?;

        sender
            .send(Bytes::copy_from_slice(data))
            .map_err(|e| JsValue::from_str(&format!("Failed to send audio frame: {}", e)))
    }

    #[wasm_bindgen(js_name = notifyNewGop)]
    pub fn notify_new_gop(&self) -> Result<(), JsValue> {
        let notifier = self.new_gop_notifier
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized or new GOP notifier not available"))?;

        notifier
            .send(())
            .map_err(|e| JsValue::from_str(&format!("Failed to notify new GOP: {}", e)))
    }

    #[wasm_bindgen]
    pub fn recv(&self) -> Result<Vec<u8>, JsValue> {
        let receiver = self.local_receiver
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;

        let bytes = receiver
            .recv()
            .map_err(|e| JsValue::from_str(&format!("Failed to receive: {}", e)))?;

        Ok(bytes.to_vec())
    }

    // #[wasm_bindgen(js_name = tryRecv)]
    // pub fn try_recv(&self) -> Result<Option<Vec<u8>>, JsValue> {
    //     let receiver = self.local_receiver
    //         .as_ref()
    //         .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;

    //     match receiver.try_recv() {
    //         Ok(bytes) => Ok(Some(bytes.to_vec())),
    //         Err(flume::TryRecvError::Empty) => Ok(None),
    //         Err(e) => Err(JsValue::from_str(&format!("Failed to receive: {}", e))),
    //     }
    // }

    #[wasm_bindgen(js_name = asyncRecv)]
    pub async fn async_recv(&self) -> Result<Vec<u8>, JsValue> {
        // let receiver = self.local_receiver
        //     .as_ref()
        //     .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;

        let recv = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;
            endpoint.local_receiver.clone()
        };
        let bytes = recv
            .recv_async()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to receive: {}", e)))?; 
        Ok(bytes.to_vec())
    }


    #[wasm_bindgen(js_name = sendKeyFrame)]
    pub fn send_key_frame(&self, data: &[u8]) -> Result<(), JsValue> {
        let mut endpoint = self.inner.lock();
        let ep = endpoint
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

        ep.send_key_frame(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to send key frame: {}", e)))?;
        console_log!("Key frame sent from wasm");
        Ok(())
    }

    #[wasm_bindgen(js_name = connectionType)]
    pub fn connection_type(&self) -> Option<String> {
        let endpoint = self.inner.lock();
        let ep = endpoint.as_ref()?;

        ep.connection_type().map(|ct| format!("{:?}", ct))
    }

    #[wasm_bindgen(js_name = roundTripTime)]
    pub fn round_trip_time(&self) -> Option<f64> {
        let endpoint = self.inner.lock();
        let ep = endpoint.as_ref()?;
        ep.round_trip_time().map(|d| d.as_secs_f64() * 1000.0)
    }

    #[wasm_bindgen(js_name = currentPacketLoss)]
    pub fn current_packet_loss(&self) -> Option<f64> {
        let mut endpoint = self.inner.lock();
        let ep = endpoint.as_mut()?;
        ep.cur_packet_loss()
    }

    // #[wasm_bindgen(js_name = isConnected)]
    // pub fn is_connected(&self) -> bool {
    //     let endpoint = self.inner.lock();
    //     if let Some(ep) = endpoint.as_ref() {
    //         ep.is_connected()
    //     } else {
    //         false
    //     }
    // }

    #[wasm_bindgen(js_name = networkChange)]
    pub fn network_change(&self) {
        let mut endpoint = self.inner.lock();
        if let Some(ep) = endpoint.as_mut() {
            ep.network_change();
        }
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
    pub fn get_stats(&self) -> Result<JsValue, JsValue> {
        let mut endpoint = self.inner.lock();
        let ep = endpoint
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

        let stats = ConnectionStats::new(
            ep.connection_type().map(|ct| format!("{:?}", ct)),
            ep.round_trip_time().map(|d| d.as_secs_f64() * 1000.0),
            ep.cur_packet_loss(),
        );

        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats: {}", e)))
    }
}