#!/bin/sh

MG_MANIFEST_PATH=stratum/utils/message-generator/Cargo.toml

RUST_LOG=info cargo run --manifest-path=$MG_MANIFEST_PATH -- stratum-message-generator/downstream-setup-connection.json