#!/usr/bin/env bash

set -e

cargo build --release
cargo run --features user-doc -- user-doc --output-dir target/docs --overwrite --renku-cli target/release/renku-cli docs/
