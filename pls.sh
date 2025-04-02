#!/bin/sh
cargo build
sudo chown root:root target/debug/pls
sudo chmod 4755 target/debug/pls
target/debug/pls $*
