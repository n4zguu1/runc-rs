# runc-rs

An OCI-compatible container runtime written in Rust.

## Workspace layout

- `crates/libcontainer` — container runtime implementation.
- `crates/libcli` — command-line interface and command handling.
- `crates/runc-rs` — executable entry point; delegates to `libcli`.

## Project documentation

- [Project brief](docs/project.md)
- [Architecture](docs/architecture.md)
- [Milestones](docs/milestones.md)
- [Architecture decision records](docs/ADRs/README.md)

## Build and run

```sh
cargo build
cargo run -p runc-rs
```
