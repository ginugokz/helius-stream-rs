use std::collections::HashMap;
use std::net::TcpStream;

use base64::Engine;
use log::{debug, warn};
use serde::Deserialize;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

use crate::error::StreamError;
use crate::health::{ReconnectPolicy, StreamHealth};
use crate::types::{AccountUpdate, Pubkey, StreamConfig, StreamState};

/// A resilient Helius WebSocket client.
///
/// Subscribes to Solana account updates and tracks health (gaps, staleness,
/// reconnect attempts). Synchronous; if you need async, wrap calls in
/// `tokio::task::spawn_blocking`.
pub struct HeliusStream {
    ws: WebSocket<MaybeTlsStream<TcpStream>>,
    config: StreamConfig,
    state: StreamState,
    health: StreamHealth,
    reconnect: ReconnectPolicy,
    /// Maps server-assigned subscription_id → pubkey.
    subs: HashMap<u64, Pubkey>,
    /// Subscribe requests sent but not yet acknowledged.
    pending_subs: Vec<(u64, Pubkey)>,
    next_request_id: u64,
}

#[derive(Debug, Deserialize)]
struct WsNotification {
    method: Option<String>,
    params: Option<WsParams>,
}

#[derive(Debug, Deserialize)]
struct WsParams {
    result: Option<WsResult>,
    subscription: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct WsResult {
    context: Option<WsContext>,
    value: Option<WsAccountValue>,
}

#[derive(Debug, Deserialize)]
struct WsContext {
    slot: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WsAccountValue {
    lamports: u64,
    data: serde_json::Value,
}

impl HeliusStream {
    /// Open a WebSocket connection to the configured endpoint.
    pub fn connect(config: StreamConfig) -> Result<Self, StreamError> {
        let url = config.ws_url();
        let (ws, _) = connect(&url).map_err(|e| StreamError::Connect(e.to_string()))?;
        Ok(Self {
            ws,
            config,
            state: StreamState::Connected { since: std::time::SystemTime::now() },
            health: StreamHealth::new(),
            reconnect: ReconnectPolicy::new(),
            subs: HashMap::new(),
            pending_subs: Vec::new(),
            next_request_id: 1,
        })
    }

    /// Subscribe to account updates for a 32-byte pubkey.
    pub fn subscribe_account(&mut self, pubkey: Pubkey) -> Result<(), StreamError> {
        if self.subs.values().any(|pk| *pk == pubkey)
            || self.pending_subs.iter().any(|(_, pk)| *pk == pubkey)
        {
            return Ok(());
        }
        let addr = bs58::encode(pubkey).into_string();
        let id = self.next_request_id;
        self.next_request_id += 1;
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "accountSubscribe",
            "params": [&addr, {"encoding": "base64", "commitment": "processed"}]
        });
        self.ws
            .send(Message::Text(req.to_string()))
            .map_err(|e| StreamError::Send(e.to_string()))?;
        self.pending_subs.push((id, pubkey));
        debug!(target: "helius_stream", "subscribed account {}", &addr[..addr.len().min(8)]);
        Ok(())
    }

    /// Convenience: subscribe by base58 string.
    pub fn subscribe_account_b58(&mut self, addr: &str) -> Result<Pubkey, StreamError> {
        let decoded = bs58::decode(addr)
            .into_vec()
            .map_err(|e| StreamError::InvalidPubkey(addr.into(), e.to_string()))?;
        if decoded.len() != 32 {
            return Err(StreamError::InvalidPubkey(
                addr.into(),
                format!("expected 32 bytes, got {}", decoded.len()),
            ));
        }
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&decoded);
        self.subscribe_account(pk)?;
        Ok(pk)
    }

    /// Block until the next account update arrives, or return None on close.
    pub fn next_update(&mut self) -> Option<AccountUpdate> {
        loop {
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let val: serde_json::Value = match serde_json::from_str(&text) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    // subscribe-ack: {"result": <sub_id>, "id": <req_id>}
                    if let (Some(sub_id), Some(req_id)) = (
                        val.get("result").and_then(|v| v.as_u64()),
                        val.get("id").and_then(|v| v.as_u64()),
                    ) {
                        if let Some(pos) = self.pending_subs.iter().position(|(id, _)| *id == req_id) {
                            let (_, pk) = self.pending_subs.remove(pos);
                            self.subs.insert(sub_id, pk);
                            debug!(target: "helius_stream", "subscribe ack req_id={} sub_id={}", req_id, sub_id);
                        }
                        continue;
                    }
                    // accountNotification
                    let notif: WsNotification = match serde_json::from_value(val) {
                        Ok(n) => n,
                        Err(_) => continue,
                    };
                    if notif.method.as_deref() != Some("accountNotification") {
                        continue;
                    }
                    let params = match notif.params {
                        Some(p) => p,
                        None => continue,
                    };
                    let sub_id = params.subscription.unwrap_or(0);
                    let pubkey = self.subs.get(&sub_id).copied().unwrap_or([0u8; 32]);
                    let result = match params.result {
                        Some(r) => r,
                        None => continue,
                    };
                    let slot = result.context.map(|c| c.slot).unwrap_or(0);
                    let (lamports, data) = match result.value {
                        Some(v) => {
                            let lam = v.lamports;
                            let raw = v
                                .data
                                .as_array()
                                .and_then(|a| a.first())
                                .and_then(|s| s.as_str())
                                .unwrap_or("");
                            (lam, decode_base64(raw))
                        }
                        None => (0, vec![]),
                    };

                    let gap = self.health.record_update(slot);
                    self.state = if gap {
                        let gap_slots = slot.saturating_sub(self.health.last_slot());
                        StreamState::Degraded { gap_slots }
                    } else {
                        StreamState::Connected { since: std::time::SystemTime::now() }
                    };

                    return Some(AccountUpdate {
                        pubkey,
                        lamports,
                        data,
                        slot,
                        write_version: slot,
                    });
                }
                Ok(Message::Ping(data)) => {
                    let _ = self.ws.send(Message::Pong(data));
                }
                Ok(Message::Close(_)) => {
                    warn!(target: "helius_stream", "connection closed by server");
                    self.state = StreamState::Failed { reason: "server close".into() };
                    return None;
                }
                Err(e) => {
                    warn!(target: "helius_stream", "ws error: {e}");
                    self.state = StreamState::Failed { reason: e.to_string() };
                    return None;
                }
                _ => {}
            }
        }
    }

    /// Re-check freshness and update internal state if the stream went stale.
    pub fn check_staleness(&mut self) {
        if self.health.is_stale(self.config.staleness_threshold) {
            let ms = self.health.last_update().elapsed().as_millis() as u64;
            self.state = StreamState::Stale { stale_for_ms: ms };
        }
    }

    /// True if state is `Connected` or mildly `Degraded` (≤2 slot gap).
    /// Use this as a circuit breaker before acting on data.
    pub fn is_safe_for_simulation(&self) -> bool {
        match &self.state {
            StreamState::Connected { .. } => true,
            StreamState::Degraded { gap_slots } => *gap_slots <= 2,
            _ => false,
        }
    }

    pub fn state(&self) -> &StreamState { &self.state }
    pub fn health(&self) -> &StreamHealth { &self.health }
    pub fn reconnect_policy_mut(&mut self) -> &mut ReconnectPolicy { &mut self.reconnect }
    pub fn subscription_count(&self) -> usize { self.subs.len() }
    pub fn pending_count(&self) -> usize { self.pending_subs.len() }
}

fn decode_base64(s: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .unwrap_or_default()
}
