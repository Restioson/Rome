#!/bin/sh
echo "Warning - this will take a long time (10min+)!"
python3 fetch_data.py
cargo run --bin rome_preprocessor --release --features "preprocess"
