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

`plebpool` is a pleb-friendly Bitcoin mining pool leveraging:
- Lightning integration via [LDK](https://lightningdevkit.org/)
- StratumV2 integration via [SRI](https://stratumprotocol.org/)

# instructions

`plebpool` takes a `.toml` configuration file as input.

there is a sample `config.toml` on the root of the repository, which the `plebpool` binary will use as default.

assuming you cloned the repo, you can start `plebpool` via:

```
$ cargo run
```

# roadmap

## Pool
- [ ] SV1 Translator Endpoint
- [ ] SV2 Endpoint
- [ ] Mining Channel Manager
- [ ] Mining Channel Difficulty Manager
- [ ] Job Declaration Channel Manager
- [ ] Job Declaration Protocol
- [ ] Share Accounting
- [ ] ?

## LN
- [ ] Peer Manager
- [ ] Lightning Channel Manager
- [ ] ?

## Pool + LN
- [ ] Coinbase Manager
- [ ] BOLT12 Payout Manager
- [ ] ?

# LN development

`plebpool` uses a Nix flake to streamline the setup of the LN development environment. The `flake.nix` file provides all necessary configurations and dependencies. Here’s an overview of what it does:

1. Dependencies: The flake pulls in dependencies from `NixOS/nixpkgs`, `rustshop/flakebox`, and `numtide/flake-utils`.
2. Development Shell: It sets up a development shell using `flakebox` utilities, including essential tools like `clightning` and `bitcoind`. 
3. Environment Configuration (`shellHook`):
 - Creates required directories for the project and Bitcoin/Lightning setups.
 - Defines convenient aliases for Bitcoin and Lightning CLI commands: `btc`, `ln1`, `ln2`
 - Ensures the Bitcoin daemon (`bitcoind`) is running and initializes a test wallet.
 - Starts two Lightning Network nodes (`ln1` and `ln2`) if they are not already active, facilitating a ready-to-use development environment.
   - `ln1 address`: `127.0.0.1:19846`
   - `ln2 address`: `127.0.0.1:15352`


To use this flake, simply run:
```
$ nix develop
```

This will launch the development shell with all the configurations and tools set up as described.

You can interact with the nodes via:
```
$ ln1 getinfo
$ ln2 getinfo
```

And write the `node_id` fields into `config.toml`

# SV2 development

`stratum-message-generator.sh` is used to automate [SRI Message Generator](https://github.com/stratum-mining/stratum/tree/main/utils/message-generator) tests.