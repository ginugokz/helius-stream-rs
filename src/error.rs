use thiserror::Error;

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("connection failed: {0}")]
    Connect(String),

    #[error("subscribe failed: {0}")]
    Subscribe(String),

    #[error("invalid base58 pubkey '{0}': {1}")]
    InvalidPubkey(String, String),

    #[error("websocket send failed: {0}")]
    Send(String),

    #[error("websocket receive failed: {0}")]
    Recv(String),
}

impl From<tungstenite::Error> for StreamError {
    fn from(e: tungstenite::Error) -> Self {
        StreamError::Recv(e.to_string())
    }
}
