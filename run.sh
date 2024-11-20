#!/bin/sh

cargo run --release --bin cli -- /opt/data/raw_data/BTCUSD.csv
cargo run --release --bin cli -- /opt/data/raw_data/audusd.csv
cargo run --release --bin cli -- /opt/data/raw_data/eurusd.csv 
cargo run --release --bin cli -- /opt/data/raw_data/gbpjpy.csv 
cargo run --release --bin cli -- /opt/data/raw_data/gbpusd.csv 
cargo run --release --bin cli -- /opt/data/raw_data/usdcad.csv 
cargo run --release --bin cli -- /opt/data/raw_data/usdchf.csv 
cargo run --release --bin cli -- /opt/data/raw_data/usdjpy.csv 
cargo run --release --bin cli -- /opt/data/raw_data/wtiusd.csv 
cargo run --release --bin cli -- /opt/data/raw_data/xagusd.csv
cargo run --release --bin cli -- /opt/data/raw_data/xauusd.csv
