#!/bin/bash

cargo run --example make-patterns --release -- 100000 | pv | cargo flamegraph --root --bin logmine-rs
open flamegraph.svg
