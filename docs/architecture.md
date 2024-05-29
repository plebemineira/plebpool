# Architecture

## Tower

`plebpool` leverages the [`tower`](https://crates.io/crates/tower) crate and its ecosystem.

[`tower-test`](https://crates.io/crates/tower-test) crate is used to assert Towers work as expected.

`main.rs` is responsible for waiting on these two `tokio::task::JoinHandle` handles:
- `LnTower::serve()?`
- `PoolTower::serve()?`

All `plebpool` logic is handled asyncrhonously inside these two main tasks.

### LN Service Tower

```
struct LnChannelManagerService { ... }
struct LnPaymentService { ... }

impl tower::Service<LnChannelManagerRequest> for LnChannelManagerService { ... }
impl tower::Service<LnPaymentRequest> for LnPaymentService { ... }

struct LnTower {
  ln_channel_manager_service: LnChannelManagerService,
  ln_payment_service: LnPaymentService,
}
```

### Pool Service Towers

```
struct Sv1MiningChannelService { ... }
struct Sv2MiningChannelService { ... }
struct JobDeclaratorService { ... }
struct TemplateReceiverService { ... }

impl tower::Service<Sv1MiningChannelRequest> { ... }
impl tower::Service<Sv2MiningChannelRequest> { ... }
impl tower::Service<JobDeclarationRequest> { ... }
impl tower::Service<TemplateReceiverRequest> { ... }

struct PoolTower {
  sv1_mining_channels_service: Sv1MiningChannelService,
  sv2_mining_channels_service: Sv2MiningChannelService,
  job_declarator_service: JobDeclaratorService,
  template_receiver_service: TemplateReceiverService
}
```

Unit tests are based on:
- `tower-test`
- [SRI Message Generator](https://github.com/stratum-mining/stratum/tree/main/utils/message-generator).

## Config

each submodule defines its own configuration `struct`:
- `LnConfig`
- `PoolConfig`

these `struct` are populated with values coming from a [`config.toml`](../config.toml), which is provided as CLI input.

