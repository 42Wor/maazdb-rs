
# MaazDB-RS 🦀

**The Official Rust SDK for MaazDB**

[🌐 Official Website](https://maazdb.vercel.app/)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Rust](https://img.shields.io/badge/rust-1.70%2B-brightgreen.svg)
![Security](https://img.shields.io/badge/security-TLS_1.3-green)

`maazdb-rs` is a high-performance Rust client library for interacting with the MaazDB engine. It implements the custom MaazDB binary protocol over a secure TLS 1.3 socket, allowing Rust applications to communicate with your database safely and efficiently with zero-cost abstractions.

## 📦 Installation

Add `maazdb-rs` to your `Cargo.toml`:

```toml
[dependencies]
maazdb-rs = "0.1.0"
```

Or via cargo:
```bash
cargo add maazdb-rs
```

## 🛠 Quickstart

Ensure your **MaazDB Server** is running on `127.0.0.1:8888`.

```rust
use maazdb_rs::{MaazDB, Result};

fn main() -> Result<()> {
    // 1. Establish a Secure Connection
    let mut db = MaazDB::connect("127.0.0.1", 8888, "admin", "admin")?;
    println!("✓ Connected to MaazDB via TLS 1.3");

    // 2. Execute SQL Commands
    db.query("CREATE DATABASE store_prod;")?;
    db.query("USE store_prod;")?;
    db.query("CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT);")?;

    // 3. Insert Data
    db.query("INSERT INTO users (name) VALUES ('Maaz');")?;
    
    // 4. Fetch Results
    let results = db.query("SELECT * FROM users;")?;
    println!("--- Query Results ---\n{}", results);

    Ok(())
}
```

## ✨ Features
- **Zero-Cost Abstractions:** High-performance binary protocol handling.
- **Memory Safe:** Built with 100% safe Rust.
- **Secure:** Powered by `rustls` for modern TLS 1.3 support.
- **Synchronous & Asynchronous:** Supports both blocking and `tokio`-based workflows.

## 📄 License
Distributed under the MIT License.

---
*Created with ❤️ for the Rust ecosystem.*
