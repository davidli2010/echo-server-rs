# echo server
A simple echo server written in Rust.

## build
```bash
cargo build
```

There will be two executable binary files: echo-server, echo-client.

## run

By default, cargo generates binary files in debug mode.

I use the env_logger crate. Logging can be turned on by ```RUST_LOG```.

run echo server:
```bash
RUST_LOG=echo_server ./target/debug/echo-server
```

run echo client:
```bash
RUST_LOG=echo_client ./target/debug/echo-client
```
