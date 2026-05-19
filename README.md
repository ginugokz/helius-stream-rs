# helius-stream

[![Crates.io](https://img.shields.io/crates/v/helius-stream.svg)](https://crates.io/crates/helius-stream)
[![Docs.rs](https://docs.rs/helius-stream/badge.svg)](https://docs.rs/helius-stream)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

> Resilient Helius WebSocket client for Solana.
> Gap detection, reconnect backoff, circuit-breaker state.
> Extracted from a battle-tested MEV engine that ran 15h on Solana mainnet.

## Why

`solana-client` pulls 200+ transitive dependencies. For workloads that only need
account subscriptions, that's a heavy tax. `helius-stream` does one thing:
subscribe to accounts via Helius WebSocket and surface updates as a typed stream,
with built-in health tracking so you can wire a circuit breaker in 3 lines.

Dependency count: 8 direct, ~25 transitive.

## Quick start

```rust
use helius_stream::{HeliusStream, StreamConfig};

let config = StreamConfig::mainnet(std::env::var("HELIUS_API_KEY")?);
let mut stream = HeliusStream::connect(config)?;

// USDC mint
stream.subscribe_account_b58("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

while let Some(update) = stream.next_update() {
    if stream.is_safe_for_simulation() {
        println!("slot={} lamports={}", update.slot, update.lamports);
    }
}
```

Run the included example:

```bash
HELIUS_API_KEY=your_key cargo run --example basic_stream
```

## What you get

- **`HeliusStream`** — synchronous WebSocket client. Subscribe to accounts,
  receive `AccountUpdate` events.
- **`StreamHealth`** — slot-gap detection, staleness tracking, gap rate.
- **`ReconnectPolicy`** — exponential backoff with cap (default: 100ms → 10s).
- **`StreamState`** — typed state machine (`Connected`, `Degraded`, `Stale`,
  `Failed`) for circuit-breaker logic.
- **No panics in normal operation** — all errors via `StreamError`.

## What you don't get

- Async API. v0.1 is sync; wrap in `spawn_blocking` if you need tokio. Async
  feature flag planned for v0.2.
- Multi-stream orchestration. Open multiple `HeliusStream` instances yourself.
- Program subscriptions, log subscriptions, signature subscriptions. v0.1
  covers only `accountSubscribe`. PRs welcome.
- A Solana SDK. This is an RPC client, not a transaction builder.

## Provenance

Extracted from [prometheus9](https://github.com/ginugokz/prometheus9), a Rust MEV
engine that ran 15 hours on Solana mainnet monitoring cross-DEX SOL/USDC
arbitrage. The engine captured zero opportunities — the market was efficient
to within 29 bps. The infrastructure code, however, proved itself: gap detection,
auto-reconnect, and zero crashes across 15 hours. That infrastructure is this
crate.

## Performance

Single connection, single thread, on a €4.50/mo Hetzner CX22:

- ~1,500 updates/hour sustained
- ~50ms p50 from server send to `next_update()` return
- ~1,200 slot gaps observed over 15 hours (network variance, not crate bug)

## License

MIT.

## Contributing

Issues and PRs at [github.com/ginugokz/helius-stream-rs](https://github.com/ginugokz/helius-stream-rs).
For paid work (custom protocols, async port, audit support), see
[the author's contact](https://github.com/ginugokz).
