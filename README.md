# telnet-did-demo

The demo of Decentralized Identifiers (DIDs) v1.0

## Installation

------------

### Pre-requisites

You'll need to install:

- [Telnet Client](https://webhostinggeeks.com/howto/how-to-install-telnet-on-windows-macos-linux/) - Client UI
- [Rust](https://www.rust-lang.org/tools/install) - Programming Language

> **_NOTE:_** We use port `3456` to connect to the server

### Init local env

Run server or Delivery Service

```bash
$ cargo run -p telnet
```

Connect to Delivery Service

```bash
$ telnet 127.0.0.1 3456
```
