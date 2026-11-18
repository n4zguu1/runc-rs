# runc-rs

An [OCI-compatible](https://github.com/opencontainers/runtime-spec/blob/main/spec.md) container runtime written in Rust.

## Workspace layout

- `crates/libcontainer` — container runtime implementation.
- `crates/libcli` — command-line interface and command handling.
- `crates/runc-rs` — executable entry point; delegates to `libcli`.

## Build and run

```sh
cargo build
cargo run -p runc-rs
```
