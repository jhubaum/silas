#!/bin/bash

cargo build --release &&
scp -r target/release/silas theme "$1:~"
