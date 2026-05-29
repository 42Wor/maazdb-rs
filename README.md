# MaazDB-RS 🦀

**The Official Rust SDK for MaazDB**

[🌐 Official Website](https://maazdb.vercel.app/)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Rust](https://img.shields.io/badge/rust-1.75%2B-brightgreen.svg)
![Security](https://img.shields.io/badge/security-TLS_1.3-green)

`maazdb-rs` is a high-performance, asynchronous Rust client library for interacting with the MaazDB engine. It implements the multiplexed MaazDB Protocol v2.1 over a secure TLS 1.3 socket, allowing Rust applications to communicate with your database safely and concurrently.

## 📦 Installation

Add `maazdb-rs` and `tokio` (required for the async runtime) to your `Cargo.toml`:

```toml
[dependencies]
maazdb-rs = "2.0.0"
tokio = { version = "1", features = ["full"] }
```

Or via cargo:
```bash
cargo add maazdb-rs tokio --features tokio/full
```

## 🛠 Quickstart

Ensure your **MaazDB Server** is running and listening on `127.0.0.1:8888`.

```rust
use maazdb_rs::MaazDB;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Establish a Secure Connection (TLS 1.3 & HMAC Handshake are handled automatically)
    let db = MaazDB::connect("127.0.0.1", 8888, "admin", "admin").await?;
    println!("✓ Connected to MaazDB via Protocol v2.1");

    // 2. Execute SQL Commands
    db.query("CREATE DATABASE store_prod;").await?;
    db.query("USE store_prod;").await?;
    db.query("CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT);").await?;

    // 3. Insert Data
    db.query("INSERT INTO users (name) VALUES ('Maaz');").await?;
    
    // 4. Fetch Results
    let results = db.query("SELECT * FROM users;").await?;
    println!("--- Query Results ---\n{}", results);

    Ok(())
}
```

## ✨ Features
- **Asynchronous Multiplexing:** Dispatch multiple queries concurrently over a single TCP connection, eliminating head-of-line blocking and reducing network turnaround time.
- **Cryptographic Handshake:** Built on the Protocol v2.1 challenge-response flow utilizing HMAC-SHA256 signatures to prevent driver spoofing.
- **Connection Sharing:** Cheaply clone the `MaazDB` handle (`db.clone()`) to share a single connection safely across multiple threads or Tokio tasks without connection pool overhead.
- **Secure by Default:** Modern TLS 1.3 encryption provided via `rustls`.
- **Memory Safe:** Built with 100% safe Rust.

## 📄 License
Distributed under the MIT License.

---
*Created with ❤️ for the Rust ecosystem.*