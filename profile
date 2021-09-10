#!/bin/bash


./make-patterns 10000 > data.txt
sudo cargo flamegraph --bin logmine-rs -- ./data.txt
open flamegraph.svg
