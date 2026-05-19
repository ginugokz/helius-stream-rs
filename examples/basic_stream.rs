//! Minimal example: subscribe to the USDC mint and print updates.
//!
//! Run with (PowerShell):
//!     $env:HELIUS_API_KEY="your_key"; cargo run --example basic_stream
//! Or (cmd.exe / bash):
//!     HELIUS_API_KEY=your_key cargo run --example basic_stream

use helius_stream::{HeliusStream, StreamConfig};

const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // No logger init — keeps example dependency-free.
    // The crate uses `log` facade; to see internal logs, add env_logger
    // (or any log backend) to your binary's Cargo.toml and init here.

    let api_key = std::env::var("HELIUS_API_KEY")
        .map_err(|_| "set HELIUS_API_KEY in environment")?;

    let config = StreamConfig::mainnet(api_key);
    let mut stream = HeliusStream::connect(config)?;

    println!("[INIT] subscribing to USDC mint");
    stream.subscribe_account_b58(USDC_MINT)?;

    let mut received = 0;
    while let Some(update) = stream.next_update() {
        received += 1;
        println!(
            "[UPDATE] slot={} lamports={} data_bytes={} safe={} gap_rate={:.3}",
            update.slot,
            update.lamports,
            update.data.len(),
            stream.is_safe_for_simulation(),
            stream.health().gap_rate(),
        );
        if received >= 10 {
            println!("[DONE] received 10 updates, exiting");
            break;
        }
    }
    Ok(())
}
