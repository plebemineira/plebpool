# Architecture

`plebpool` has two submodules:
- `ln`: leveraging LDK to integrate LN functionality
- `pool`: leveraging SRI to integrate SV2 functionality

## Config

each submodule defines its own configuration `struct`:
- `LnConfig`
- `PoolConfig`

these `struct` are populated with values coming from a [`config.toml`](../config.toml), which is provided as CLI input.

## Services

`main.rs` is responsible for waiting on these two `tokio::task::JoinHandle` handles:
- `LnService::serve()`
- `PoolService::serve()`

All `plebpool` logic is handled asyncrhonously inside these two main tasks.