# Compile the runtime with srtool

When voting for a runtime upgrade, you should check that the proposed hash actually corresponds to the published code you reviewed. Otherwise, a malicious runtime upgrade could be advertised as a legitimate one.

```docker
docker run \
  -i \
  --rm \
  -e PACKAGE=gdev-runtime \
  -e RUNTIME_DIR=runtime/gdev \
  -v $PWD:/build \
  paritytech/srtool:1.60.0 build --app --json -cM
```

Then, the runtime wasm bytecode is generated in this location:

```
runtime/gdev/target/srtool/release/wbuild/gdev-runtime/gdev_runtime.compact.compressed.wasm
```

To compare it to last official :
- download it here : https://git.duniter.org/nodes/rust/duniter-v2s/-/releases
- compare `sha256sum` of it and of the one you've built
