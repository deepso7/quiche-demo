# quic-example

A minimal QUIC hello-world built with [`quiche`](https://crates.io/crates/quiche), split into separate server and client binaries for readability.

## What this does

- Starts a QUIC server on `127.0.0.1:5555`.
- Starts a QUIC client on `127.0.0.1:4444`.
- Client sends `hello from client` on stream `0`.
- Server replies with `hello from quiche server`.

## Project layout

- `src/bin/server.rs` - QUIC server binary.
- `src/bin/client.rs` - QUIC client binary.
- `src/certs.rs` - helper functions to generate/ensure local cert files.
- `src/lib.rs` - shared constants and quiche config builders.

## Prerequisites

- Rust toolchain (stable)
- `cmake` (required by quiche's default vendored BoringSSL build)

On macOS:

```bash
brew install cmake
```

## Run

In terminal 1:

```bash
cargo run --bin server
```

In terminal 2:

```bash
cargo run --bin client
```

Expected output:

- Server prints `Server received: hello from client`
- Client prints `Client received: hello from quiche server`

## Certificates

- Certificates are auto-created on first run under `certs/`.
- Generated files:
  - `certs/server.crt`
  - `certs/server.key`
- The client uses strict verification (`verify_peer(true)`) and trusts `certs/server.crt` as its CA file.

`certs/` is ignored by git.

## Quick checks

```bash
cargo check --bins
```
