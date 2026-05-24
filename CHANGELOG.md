# Changelog

All notable changes to `helius-stream` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-05-23

### Documentation

- Minor README clarifications in the `Performance` section.
- Added this `CHANGELOG.md`.

## [0.1.0] - 2026-05-19

### Added

- Initial release.
- `HeliusStream` — synchronous WebSocket client for `accountSubscribe` over
  Helius mainnet/devnet endpoints.
- `StreamHealth` — slot-delta gap detection, staleness tracking.
- `ReconnectPolicy` — exponential backoff with configurable cap.
- `StreamState` — typed state machine (`Connected` / `Degraded` / `Stale` /
  `Failed`) for circuit-breaker logic.
- `StreamError` — all errors surface here; no panics in normal paths.
- `examples/basic_stream.rs` — runnable example subscribing to the USDC mint.

[0.1.1]: https://github.com/ginugokz/helius-stream-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ginugokz/helius-stream-rs/releases/tag/v0.1.0
