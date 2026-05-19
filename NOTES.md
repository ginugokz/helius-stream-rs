# Notas de extração — para ginug

Este documento não vai para o repo público. Serve para tu te lembrares do que mudou em relação ao prometheus9 original e onde tomar decisões antes de publicar.

## O que foi extraído

Ficheiros-fonte de `C:\Users\ginug\Desktop\VALIDADOS\prometheus9_FINAL_20260517\prometheus_clean\mev_tip_engine\src\`:

- `helius_stream.rs` → `src/stream.rs` (decoupled de pool_state)
- `grpc_streamer.rs` → `src/health.rs` (StreamHealth, ReconnectPolicy) + `src/types.rs` (AccountUpdate, StreamConfig, StreamState)

## Mudanças face ao original

1. **Subscrições deixaram de ser hardcoded.** O `subscribe_all()` original assumia `POOL_VAULTS` + `ORCA_SOL_USDC_WHIRLPOOL`. Agora a API exige que o consumidor chame `subscribe_account()` explicitamente. Mais geral, mais reutilizável.

2. **`bs58` agora é dependência externa.** O prometheus9 usa `mev_tip_engine::bs58`. A crate usa `bs58 = "0.5"` (standard, mais auditada). Resultado equivalente, menos código que tens de manter.

3. **`base64` agora é dependência externa.** Igual ao acima — usa `base64 = "0.22"` em vez do decoder manual em `helius_stream.rs`.

4. **`eprintln!` substituído por `log::debug!` e `log::warn!`.** Convenção de biblioteca — o consumidor escolhe o backend (env_logger, tracing-log, slog). Não imprime para stderr por omissão.

5. **Erros tipados.** O original usa `Box<dyn std::error::Error>`. Agora há `StreamError` em `src/error.rs` com variantes específicas.

6. **Endpoint configurável.** O original hardcoded `wss://mainnet.helius-rpc.com/`. Agora `StreamConfig::mainnet()` / `::devnet()` / `::custom()`.

7. **API key construída via método em vez de format inline.** `StreamConfig::ws_url()` lida com o caso de o endpoint custom já trazer query string.

## O que NÃO foi extraído (deliberadamente)

- `subscribe_vault()` específico — substituído por `subscribe_account()` genérico.
- `subscription_target_count()` estático — agora `subscription_count()` retorna o real em runtime.
- Todos os imports cruzados com `mev_tip_engine::pool_state` — removidos.
- Modo paper trading, circuit breaker no sentido do prometheus9, e outros módulos.

Se algum dia quiseres uma versão "batteries-included" com pools Solana conhecidos pré-configurados, isso é um crate separado (`helius-stream-presets` ou similar). Mantém a base limpa.

## Antes de `cargo publish`

| Verificação | Comando | Notas |
|---|---|---|
| Compila | `cargo build` | |
| Testes passam | `cargo test` | health.rs tem 6 testes copiados do prometheus9 |
| Lint | `cargo clippy -- -D warnings` | Pode haver warnings menores a corrigir |
| Doc compila | `cargo doc --no-deps` | Verifica o doctest em lib.rs |
| Exemplo corre | `HELIUS_API_KEY=xxx cargo run --example basic_stream` | Sanity check com API real |
| Nome livre | `cargo search helius-stream` | Se ocupado, ver alternativas em baixo |
| GitHub repo público | criar `ginug/helius-stream-rs` | URL no Cargo.toml já aponta para lá |

## Alternativas de nome (se "helius-stream" estiver ocupado)

- `helius-rpc-stream`
- `helio-stream` (afastamento do nome da marca)
- `solana-ws-stream`
- `solana-account-stream`

Notar: usar "helius" no nome levanta risco trademark menor mas real. A Helius Labs até hoje (que eu saiba em 2026-05) não fez enforcement contra crates community, mas se quiseres ser conservador, escolhe uma das três sem "helius".

## Próximos passos para monetização da crate

1. Publicar v0.1.0 em crates.io
2. Push do repo público com badges funcionais
3. Adicionar GitHub Sponsors no repo
4. Mencionar a crate na thread Twitter do prometheus9 (a que está em `NEW/twitter_thread_prometheus9.md`) como produto derivado — Tweet 6 ou novo tweet final
5. Quando alguém abrir issue/PR, oferecer consultoria paga para custom protocols (Drift, Kamino, Marinade, etc.) — €80/h, factura via Stripe/Wise

## O que NÃO depende de mim

- Nome da crate final (decide tu)
- `[authors]` no Cargo.toml — coloquei "ginug", confirma se queres nome próprio
- URL do repo — assumi `github.com/ginug/helius-stream-rs`, ajusta se quiseres outro org
- Versão inicial — `0.1.0`. Alguns devs preferem começar em `0.0.1`. Indiferente para crates.io.
