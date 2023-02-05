# Wasm Example

## How to launch on your environment?

You can also launch the demo server on your machine using [trunk](https://github.com/thedodd/trunk).

Run the following commands in this directory:
```
# Installs wasm target of Rust compiler.
rustup target add wasm32-unknown-unknown

# Installs trunk
cargo install trunk
cargo install wasm-bindgen-cli

# Downloads and extracts the model file
wget https://github.com/daac-tools/vibrato/releases/download/v0.4.0/ipadic-mecab-2_7_0-small.tar.xz
tar xf ./ipadic-mecab-2_7_0-small.tar.xz
mv ./ipadic-mecab-2_7_0-small/system.dic.zst ./src/

# Builds and launches the server
# Note: We recommend using --release flag to reduce loading time.
trunk serve --release
```

For ARM Mac, you may need to install binaryen to build wasm-opt.
```
brew install binaryen
```
