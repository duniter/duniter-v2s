cargo build --release --features runtime-benchmarks
target/release/duniter benchmark storage --chain=dev --mul=2 --weight-path=./runtime/common/src/weights/ --state-version=1
target/release/duniter benchmark overhead --chain=dev --wasm-execution=compiled --weight-path=./runtime/common/src/weights/ --warmup=10 --repeat=100
target/release/duniter benchmark pallet --chain=dev --steps=50 --repeat=20 --pallet="*" --extrinsic="*" --wasm-execution=compiled --heap-pages=4096 --header=./file_header.txt --output=./runtime/common/src/weights/

