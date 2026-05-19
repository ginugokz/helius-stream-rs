use std::time::Duration;

/// A Solana account public key — 32 raw bytes.
pub type Pubkey = [u8; 32];

/// An account update received from the stream.
#[derive(Debug, Clone)]
pub struct AccountUpdate {
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub slot: u64,
    pub write_version: u64,
}

/// Configuration for a single stream connection.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub endpoint: String,
    pub api_key: String,
    pub staleness_threshold: Duration,
}

impl StreamConfig {
    /// Default mainnet endpoint.
    pub fn mainnet(api_key: impl Into<String>) -> Self {
        Self {
            endpoint: "wss://mainnet.helius-rpc.com/".into(),
            api_key: api_key.into(),
            staleness_threshold: Duration::from_millis(500),
        }
    }

    /// Default devnet endpoint.
    pub fn devnet(api_key: impl Into<String>) -> Self {
        Self {
            endpoint: "wss://devnet.helius-rpc.com/".into(),
            api_key: api_key.into(),
            staleness_threshold: Duration::from_millis(500),
        }
    }

    /// Custom endpoint (e.g., an enterprise / dedicated node URL).
    pub fn custom(endpoint: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            staleness_threshold: Duration::from_millis(500),
        }
    }

    /// How long without an update before the stream is considered stale.
    /// Default: 500ms.
    pub fn staleness_threshold(mut self, d: Duration) -> Self {
        self.staleness_threshold = d;
        self
    }

    pub(crate) fn ws_url(&self) -> String {
        let sep = if self.endpoint.contains('?') { '&' } else { '?' };
        format!("{}{}api-key={}", self.endpoint, sep, self.api_key)
    }
}

/// Current operational state of the stream.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { since: std::time::SystemTime },
    Degraded { gap_slots: u64 },
    Stale { stale_for_ms: u64 },
    Failed { reason: String },
}
