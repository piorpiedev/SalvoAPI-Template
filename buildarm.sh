#!/bin/bash
CARGO_TARGET_DIR=target/cross cross build --target aarch64-unknown-linux-gnu --release
