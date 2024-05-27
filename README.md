# plebpool

⛏️ plebs be hashin ⚡

plebpool is a pool for the plebs
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

# nix develop

a [nix flake]() is provided with the following development setup:
- one `bitcoind` on `regtest` mode
- two `lightningd`

assuming you cloned the repo, and that nix flakes are enabled on your system, you can launch this development setup via:
```
$ nix develop
```

