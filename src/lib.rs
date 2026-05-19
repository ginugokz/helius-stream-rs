//! # helius-stream
//!
//! A resilient WebSocket client for Helius RPC on Solana.
//!
//! Built for production: gap detection, reconnect backoff with exponential
//! jitter, circuit-breaker-friendly state machine, and zero panic surface.
//!
//! Extracted from `prometheus9`, an MEV engine that ran 15h on Solana mainnet
//! against Raydium AMM v4 and Orca Whirlpool. The original engine subscribed
//! to specific vault accounts; this crate exposes the same machinery as a
//! general-purpose primitive.
//!
//! ## Quick start
//!
//! ```no_run
//! use helius_stream::{HeliusStream, StreamConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = StreamConfig::mainnet(std::env::var("HELIUS_API_KEY")?);
//! let mut stream = HeliusStream::connect(config)?;
//!
//! // USDC mint as an example
//! stream.subscribe_account_b58("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
//!
//! while let Some(update) = stream.next_update() {
//!     if stream.is_safe_for_simulation() {
//!         println!("slot={} lamports={} bytes={}",
//!             update.slot, update.lamports, update.data.len());
//!     }
//! }
//! # Ok(()) }
//! ```
//!
//! ## Why not just use `solana-client`?
//!
//! `solana-client` pulls in 200+ transitive dependencies. This crate has 8.
//! For workloads that only need account subscriptions, that matters.

mod error;
mod health;
mod stream;
mod types;

pub use error::StreamError;
pub use health::{ReconnectPolicy, StreamHealth};
pub use stream::HeliusStream;
pub use types::{AccountUpdate, Pubkey, StreamConfig, StreamState};
