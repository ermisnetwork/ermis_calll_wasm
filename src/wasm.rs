
use bytes::Bytes;
use wasm_bindgen::prelude::*;
use serde::{ Deserialize, Serialize };
use std::cell::RefCell;
use std::rc::Rc;

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
    inner: Rc<RefCell<Option<ErmisCallEndpoint>>>,
}

#[wasm_bindgen]
impl ErmisCall {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            inner: Rc::new(RefCell::new(None)),
        }
    }

    #[wasm_bindgen]
    pub async fn spawn(&self, relay_urls: JsValue) -> Result<(), JsValue> {
        let urls: Vec<String> = serde_wasm_bindgen
            ::from_value(relay_urls)
            .map_err(|e| JsValue::from_str(&format!("Invalid relay URLs: {}", e)))?;

        let url_refs: Vec<&str> = urls
            .iter()
            .map(|s| s.as_str())
            .collect();

        let endpoint = ErmisCallEndpoint::new(&url_refs).await.map_err(|e|
            JsValue::from_str(&format!("Failed to spawn: {}", e))
        )?;

        *self.inner.borrow_mut() = Some(endpoint);
        console_log!("ErmisCall endpoint spawned successfully");
        Ok(())
    }



    #[wasm_bindgen(js_name = getLocalEndpointAddr)]
    pub fn get_local_endpoint_addr(&self) -> Result<String, JsValue> {
        let inner = self.inner.borrow();
        let endpoint = inner.as_ref().ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

        endpoint
            .get_local_endpoint_addr()
            .map_err(|e| JsValue::from_str(&format!("Failed to get address: {}", e)))
    }

    #[wasm_bindgen]
    // pub async fn connect(&self, addr: &str) -> Result<(), JsValue> {
    //     // let mut inner = self.inner.borrow_mut();
    //     // let endpoint = inner.as_mut()
    //     //     .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
    //     let mut endpoint = {
    //         let mut inner = self.inner.borrow_mut();
    //         inner
    //             .as_mut()
    //             .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
    //             .clone() // Cần ErmisCallEndpoint implement Clone
    //     };

    //     endpoint
    //         .connect(addr).await
    //         .map_err(|e| JsValue::from_str(&format!("Failed to connect: {}", e)))?;

    //     console_log!("current connection after connected: {:?}", endpoint.get_current_connection());

    //     console_log!("Connected to peer");
    //     Ok(())
    // }

     pub async fn connect(&self, addr: &str) -> Result<(), JsValue> {
        let inner = self.inner.clone();
        let addr = addr.to_string();
        // let endpoint;
        // {
        //     let mut endpoint = inner.borrow_mut();
        //      endpoint = endpoint.as_mut()
        //         .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
            
        // }

        inner.borrow_mut()
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
            .connect(&addr)
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to connect: {}", e)))?;

        console_log!("Connected to peer");
        Ok(())
    }

    #[wasm_bindgen(js_name = acceptConnection)]
    pub async fn accept_connection(&self) -> Result<(), JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.borrow_mut();
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone() // Cần ErmisCallEndpoint implement Clone
        };
        endpoint
            .accept_connection().await
            .map_err(|e| JsValue::from_str(&format!("Failed to accept connection: {}", e)))?;

        console_log!("Connection accepted");
        Ok(())
    }

    #[wasm_bindgen(js_name = acceptBidiStream)]
    pub async fn accept_bidi_stream(&self) -> Result<(), JsValue> {
        let inner = self.inner.clone();

        inner.borrow_mut()
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
            .accept_bidi_stream().await
            .map_err(|e| JsValue::from_str(&format!("Failed to accept bidi stream: {}", e)))?;

        console_log!("Bidi stream accepted");
        Ok(())
    }

    #[wasm_bindgen(js_name = openBidiStream)]
    pub async fn open_bidi_stream(&self) -> Result<(), JsValue> {
        let inner = self.inner.clone();

        inner.borrow_mut()
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
            .open_bidi_stream()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to open bidi stream: {}", e)))?;

        console_log!("Bidi stream opened");
        Ok(())
    }


    #[wasm_bindgen]
    pub fn send(&self, data: &[u8]) -> Result<(), JsValue> {

        let endpoint = self.inner.borrow();
        let endpoint = endpoint
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;
        let sender = endpoint.local_sender.clone();
        sender
            .send(Bytes::copy_from_slice(data))
            .map_err(|e| JsValue::from_str(&format!("Failed to send: {}", e)))
    }

    #[wasm_bindgen(js_name = asyncSend)]
    pub async fn async_send(&self, data: &[u8]) -> Result<(), JsValue> {

        let mut endpoint = {
            let mut inner = self.inner.borrow_mut();
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone() 
        };

        endpoint
            .async_send(data).await
            .map_err(|e| JsValue::from_str(&format!("Failed to async send: {}", e)))
    }

    #[wasm_bindgen]
    pub fn recv(&self) -> Result<Vec<u8>, JsValue> {

        let mut endpoint = {
            let mut inner = self.inner.borrow_mut();
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone() 
        };

        let bytes = endpoint
            .recv()
            .map_err(|e| JsValue::from_str(&format!("Failed to recv: {}", e)))?;

        Ok(bytes.to_vec())
    }

    #[wasm_bindgen(js_name = asyncRecv)]
    pub async fn async_recv(&self) -> Result<Vec<u8>, JsValue> {
        let mut endpoint = {
            let mut inner = self.inner.borrow_mut();
            inner
                .as_mut()
                .ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?
                .clone() 
        };

        let bytes = endpoint
            .async_recv().await
            .map_err(|e| JsValue::from_str(&format!("Failed to async recv: {}", e)))?;

        Ok(bytes.to_vec())
    }

    #[wasm_bindgen(js_name = connectionType)]
    pub fn connection_type(&self) -> Option<String> {
        let inner = self.inner.borrow();
        let endpoint = inner.as_ref()?;

        endpoint.connection_type().map(|ct| format!("{:?}", ct))
    }

    #[wasm_bindgen(js_name = roundTripTime)]
    pub fn round_trip_time(&self) -> Option<f64> {
        let inner = self.inner.borrow();
        let endpoint = inner.as_ref()?;

        endpoint.round_trip_time().map(|d| d.as_secs_f64() * 1000.0)
    }

    #[wasm_bindgen(js_name = currentPacketLoss)]
    pub fn current_packet_loss(&self) -> Option<f64> {
        let inner = self.inner.borrow();
        let endpoint = inner.as_ref()?;

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
    pub fn get_stats(&self) -> Result<JsValue, JsValue> {
        let inner = self.inner.borrow();
        let endpoint = inner.as_ref().ok_or_else(|| JsValue::from_str("Endpoint not initialized"))?;

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


