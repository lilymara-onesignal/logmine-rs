#!/bin/bash

cargo run --example make-patterns --release -- 100000 > data.txt
cargo flamegraph --root --bin logmine-rs -- data.txt
open flamegraph.svg
