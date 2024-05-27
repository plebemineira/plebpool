# plebpool

<h1 align="center">
  <br>
  <img width="100" src="dwarf.png">
  <br>
plebpool
<br>
</h1>

<p align="center">
⛏️ plebs be hashin ⚡
</p>

# intro

`plebpool` is a Bitcoin mining pool for the plebs leveraging:
- Lightning integration via [LDK](https://lightningdevkit.org/)
- StratumV2 integration via [SRI](https://stratumprotocol.org/)

# instructions

`plebpool` takes a `.toml` configuration file as input.

there is a sample `config.toml` on the root of the repository, which the `plebpool` binary will use as default.

assuming you cloned the repo, you can start `plebpool` via:

```
$ cargo build --release
$ target/release/plebpool
```

# roadmap

## Pool
- [ ] Channel Difficulty Management
- [ ] Channel Factory
- [ ] JDS
- [ ] Share Accounting
- [ ] ?

## LN
- [ ] BOLT12
- [ ] ?

## Pool + LN
- [ ] Coinbase Management
- [ ] ?