#!/bin/sh
cargo +nightly build --release \
  -Z build-std=core,std,alloc,panic_abort \
  -Z build-std-features= \
  --target x86_64-unknown-linux-musl
