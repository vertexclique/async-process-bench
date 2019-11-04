export RUST_BACKTRACE=1

echo "Async-std benchmark started"
cd asyncstd-bench && cargo run && cd -
