use bytes::Bytes;
use flume::{Sender, TrySendError};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::js_sys::{self, Uint8Array};
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
    new_gop_notifier: Option<Sender<()>>,
}

#[wasm_bindgen]
impl ErmisCall {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            new_gop_notifier: None,
        }
    }

    #[wasm_bindgen]
    pub async fn spawn(
        &mut self,
        relay_urls: JsValue,
        secret_key: Option<Vec<u8>>,
    ) -> Result<(), JsValue> {
        let urls: Vec<String> = match serde_wasm_bindgen::from_value(relay_urls)
            .map_err(|e| JsValue::from_str(&format!("Invalid relay URLs: {}", e))) {
            Ok(urls) => urls,
            Err(e) => {
                console_log!("Invalid relay URLs: {:?}", e);
                return Err(e)},
            };

        let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();

        let array: Option<[u8; 32]> = secret_key
            .as_deref()
            .and_then(|v| v.try_into().ok());

        let endpoint = match ErmisCallEndpoint::new(&url_refs, array.as_ref()).await
            .map_err(|e| JsValue::from_str(&format!("Failed to spawn: {}", e))) {
            Ok(ep) => ep,
            Err(e) => {
                console_log!("Failed to spawn: {:?}", e);
                return Err(e)},
            };

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


    #[wasm_bindgen(js_name = sendControlFrame)]
    pub fn send_control_frame(&self, data: Vec<u8>) -> Result<(), JsValue> {
        let sender = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;
            endpoint.local_control_sender.clone()
        };

        sender
            .send(data.into())
            .map_err(|e| JsValue::from_str(&format!("Failed to send control frame: {}", e)))
    }
    #[wasm_bindgen(js_name = sendAudioFrame)]
    pub fn send_audio_frame(&self, data: Vec<u8>) -> Result<(), JsValue> {
        self.send_audio_frame_inner(data.into())
    }

    #[wasm_bindgen(js_name = sendFrame)]
    pub fn send_frame(&self, data: Vec<u8>) -> Result<(), JsValue> {
        self.send_frame_inner(data.into())
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
    pub fn recv(&self) -> Result<Uint8Array, JsValue> {
        let recv = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;
            endpoint.local_receiver.clone()
        };  

        let bytes = recv
            .recv()
            .map_err(|e| JsValue::from_str(&format!("Failed to receive: {}", e)))?;

        Ok(bytes.as_ref().into())
    }



    #[wasm_bindgen(js_name = asyncRecv)]
    pub async fn async_recv(&self) -> Result<Uint8Array, JsValue> {

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
        Ok(bytes.as_ref().into())
    }


    #[wasm_bindgen(js_name = beginWithGop)]
    pub fn begin_with_gop(&self, data: Vec<u8>) -> Result<(), JsValue> {
        let mut endpoint = self.inner.lock();
        let ep = endpoint
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

        ep.begin_with_gop(data)
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


    #[wasm_bindgen(js_name = networkChange)]
    pub fn network_change(&self) {
        let mut endpoint = self.inner.lock();
        if let Some(ep) = endpoint.as_mut() {
            ep.network_change();
        }
    }


}

impl ErmisCall {
   fn send_frame_inner(&self, data: Bytes) -> Result<(), JsValue> {
        let sender = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;
            endpoint.local_sender.clone()
        };
        let remote_receiver = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;
            endpoint.remote_receiver.clone()
        };
        if let Err(TrySendError::Full(f)) = sender.try_send(data) {
            let _ = remote_receiver.try_recv();
            self.send_frame_inner(f)?;
        }
        Ok(())
    }

   fn send_audio_frame_inner(&self, data: Bytes) -> Result<(), JsValue> {
        let sender = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;
            endpoint.local_audio_sender.clone()
        };
        let remote_receiver = {
            let inner = self.inner.lock();
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized or local receiver not available"))?;
            endpoint.remote_audio_receiver.clone()
        };
        if let Err(TrySendError::Full(f)) = sender.try_send(data) {
            let _ = remote_receiver.try_recv();
            self.send_audio_frame_inner(f)?;
        }
        Ok(())
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