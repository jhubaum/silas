#!/bin/bash

echo "$1:~"
cargo build --release &&
scp -r target/release/silas theme "$1:~"
