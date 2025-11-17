use bytes::Bytes;
use tokio::sync::Mutex;
use wasm_bindgen::prelude::*;
use serde::{ Deserialize, Serialize };
use std::sync::Arc;

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
        secret_key: Option<Vec<u8>>
    ) -> Result<(), JsValue> {
        let urls: Vec<String> = serde_wasm_bindgen
            ::from_value(relay_urls)
            .map_err(|e| JsValue::from_str(&format!("Invalid relay URLs: {}", e)))?;

        let url_refs: Vec<&str> = urls
            .iter()
            .map(|s| s.as_str())
            .collect();
        // let secret_key_ref = secret_key.as_deref().map(|v| {
        //     let mut arr = [0u8; 32];
        //     arr.copy_from_slice(&v[0..32]);
        //     arr
        // });
        let array = secret_key
            .as_deref()
            .map(|v|
                v
                    .try_into()
                    .map_err(|_| JsValue::from_str("Invalid length"))
                    .unwrap()
            )
            .unwrap_or(&[0u8; 32]);

        let endpoint = ErmisCallEndpoint::new(&url_refs, Some(array)).await.map_err(|e|
            JsValue::from_str(&format!("Failed to spawn: {}", e))
        )?;

        let mut inner = self.inner.lock().await;
        *inner = Some(endpoint);

        Ok(())
    }

    #[wasm_bindgen(js_name = getLocalEndpointAddr)]
    pub async fn get_local_endpoint_addr(&self) -> Result<String, JsValue> {
        let endpoint = self.inner.lock().await;

        endpoint
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
            .get_local_endpoint_addr().await
            .map_err(|e| JsValue::from_str(&format!("Failed to get address: {}", e)))
    }

    #[wasm_bindgen]
    pub async fn connect(&self, addr: &str) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.lock().await;
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone()
        };
        endpoint
            .connect(addr).await
            .map_err(|e| JsValue::from_str(&format!("Failed to connect: {}", e)))?;

        self.inner.lock().await.replace(endpoint);

        console_log!("Connected to peer");
        Ok(())
    }

    #[wasm_bindgen(js_name = acceptConnection)]
    pub async fn accept_connection(&self) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.lock().await;
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone()
        };
        endpoint
            .accept_connection().await
            .map_err(|e| JsValue::from_str(&format!("Failed to accept connection: {}", e)))?;

        let conn = endpoint.get_current_connection();
        if let Some(c) = conn {
            console_log!("Accepted connection from {:?}", c.remote_node_id());
            self.inner.lock().await.replace(endpoint);
        } else {
            console_log!("No connection found after acceptance");
        }

        console_log!("Connection accepted");
        Ok(())
    }

    #[wasm_bindgen(js_name = acceptBidiStream)]
    pub async fn accept_bidi_stream(&self) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.lock().await;
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone()
        };
        endpoint
            .accept_bidi_stream().await
            .map_err(|e| JsValue::from_str(&format!("Failed to accept bidi stream: {}", e)))?;

        console_log!("Bidi stream accepted");
        Ok(())
    }

    #[wasm_bindgen(js_name = openBidiStream)]
    pub async fn open_bidi_stream(&self) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.lock().await;
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone()
        };
        endpoint
            .open_bidi_stream().await
            .map_err(|e| JsValue::from_str(&format!("Failed to open bidi stream: {}", e)))?;

        console_log!("Bidi stream opened");
        Ok(())
    }

    #[wasm_bindgen(js_name = asyncSend)]
    pub async fn async_send(&self, data: &[u8]) -> Result<(), JsValue> {
        let sender = {
            let inner = self.inner.lock().await;
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            endpoint.local_sender.clone()
        };

        sender
            .send_async(Bytes::copy_from_slice(data)).await
            .map_err(|e| JsValue::from_str(&format!("Failed to async send: {}", e)))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = asyncRecv)]
    pub async fn async_recv(&self) -> Result<Vec<u8>, JsValue> {
        let recv = {
            let inner = self.inner.lock().await;
            let endpoint = inner
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            endpoint.local_receiver.clone()
        };

        let bytes = recv
            .recv_async().await
            .map_err(|e| JsValue::from_str(&format!("Failed to async receive: {}", e)))?;

        Ok(bytes.to_vec())
    }

    #[wasm_bindgen(js_name = connectionType)]
    pub async fn connection_type(&self) -> Option<String> {
        let endpoint = self.inner.lock().await;
        let endpoint = endpoint.as_ref()?;

        endpoint.connection_type().map(|ct| format!("{:?}", ct))
    }

    #[wasm_bindgen(js_name = roundTripTime)]
    pub async fn round_trip_time(&self) -> Option<f64> {
        let endpoint = self.inner.lock().await;
        let endpoint = endpoint.as_ref()?;
        endpoint.round_trip_time().map(|d| d.as_secs_f64() * 1000.0)
    }

    #[wasm_bindgen(js_name = currentPacketLoss)]
    pub async fn current_packet_loss(&self) -> Option<f64> {
        let endpoint = self.inner.lock().await;
        let endpoint = endpoint.as_ref()?;
        endpoint.cur_packet_loss()
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
        packet_loss: Option<f64>
    ) -> ConnectionStats {
        ConnectionStats { connection_type, rtt_ms, packet_loss }
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
        let endpoint = self.inner.lock().await;
        let endpoint = endpoint
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

        let stats = ConnectionStats::new(
            endpoint.connection_type().map(|ct| format!("{:?}", ct)),
            endpoint.round_trip_time().map(|d| d.as_secs_f64() * 1000.0),
            endpoint.cur_packet_loss()
        );

        serde_wasm_bindgen
            ::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats: {}", e)))
    }
}
