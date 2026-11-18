# Dev

```bash
# Dev
cargo r --bin game_name
# Hot reload
cargo r --bin game_name --features hot_reload
# Release
cargo b --release --bin game_name

```

Enable features/targets in .vscode/settings.json to test features.

`"rust-analyzer.cargo.target": "aarch64-linux-android"`

# Windows

Need to install vs studio stuff for Rust.

# Android Build

Follow instructions from [egui demo](https://github.com/emilk/egui/tree/main/examples/hello_android).

(Might need to add support for 32-bit.)

For release, need to setup the keystore.

```bash
# Keystore
keytool -genkey -v -keystore fvnrs-release.keystore -alias fvnrs-release -keyalg RSA -keysize 2048 -validity 10000

source .env.android

# Set these env vars:
export CARGO_APK_RELEASE_KEYSTORE=/root/fvnrs-release.keystore
export CARGO_APK_RELEASE_KEYSTORE_PASSWORD=""

# 64-bit
cargo apk build --release --manifest-path game/app/Cargo.toml --lib --target aarch64-linux-android

# 32-bit
cargo apk build --release --manifest-path game/app/Cargo.toml --lib --target armv7-linux-androideabi
```

# Linux Build

Use WSL and build/run with cargo normally.

# MacOS Build

Build normally with cargo on a Mac, make sure to build for Intel and Apple silicon.
```bash
# Apple silicon
cargo build --release -p app --target aarch64-apple-darwin
# Intel
cargo build --release -p app --target x86_64-apple-darwin

# Combine binaries
lipo -create -output game_name target/aarch64-apple-darwin/release/game_name target/x86_64-apple-darwin/release/game_name
```