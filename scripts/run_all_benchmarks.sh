set -e
for chain in g1 gtest gdev
do
  cargo build --release --no-default-features --features runtime-benchmarks,$chain
  target/release/duniter benchmark storage --dev --mul=2 --weight-path=./runtime/$chain/src/weights/ --state-version=1 --database=paritydb --disable-pov-recorder --batch-size=100
  target/release/duniter benchmark overhead --chain=dev --wasm-execution=compiled --weight-path=./runtime/$chain/src/weights/ --warmup=10 --repeat=100
  target/release/duniter benchmark pallet --genesis-builder=spec-genesis --steps=50 --repeat=20 --pallet="*" --extrinsic="*" --wasm-execution=compiled --heap-pages=4096 --header=./file_header.txt --output=./runtime/$chain/src/weights/
done
