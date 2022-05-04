#!/bin/sh

PKG_CONFIG_ALLOW_CROSS=1 \
RUSTFLAGS='-C link-arg=-lgcc_eh -C link-arg=-lgcc -C target-feature=+crt-static' \
cargo +nightly build --release \
  -Z build-std=core,std,alloc,panic_abort \
  -Z build-std-features= \
  --target x86_64-unknown-linux-musl
