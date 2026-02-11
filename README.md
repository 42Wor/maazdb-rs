

# MaazDB-RS 🦀

**The Official Rust SDK for MaazDB**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)

`maazdb-rs` is a high-performance, asynchronous-capable Rust off driver for **MaazDB**. Built on top of `tokio` and `rustls`, it provides full TLS 1.3 encryption and a type-safe interface for interacting with your MaazDB instances.



## 📦 Installation

Add `maazdb-rs` to your `Cargo.toml` dependencies.


### from crates
```toml
[dependencies]
maazdb-rs = "0.1.0"
```
### Local Development (Using path)
```toml
[dependencies]
maazdb-rs = { path = "../maazdb-rs" }
tokio = { version = "1", features = ["full"] }
```

### From GitHub
```toml
[dependencies]
maazdb-rs = { git = "https://github.com/42Wor/maazdb-rs" }
```
### from crates
```toml
[dependencies]
maazdb-rs = "0.1.0"
```
## 🛠 Quickstart

Make sure your **MaazDB Server** is running on `127.0.0.1:8888` before running your code.

```rust
use maazdb_rs::MaazDB;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Establish Secure Session
    let mut db = MaazDB::connect("127.0.0.1", 8888, "admin", "admin")?;
    println!("✓ Connected to MaazDB Cluster via TLS 1.3");

    // 2. Data Definition
    db.query("CREATE DATABASE store_prod;")?;
    db.query("USE store_prod;")?;
    db.query("CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT, email TEXT);")?;

    // 3. High-Speed Ingestion
    db.query("INSERT INTO users (name, email) VALUES ('Maaz', 'dev@maazdb.io');")?;
    
    // 4. Advanced Selection
    let results = db.query("SELECT * FROM users WHERE name = 'Maaz';")?;
    println!("--- Query Results ---\n{}", results);

    // 5. Safe Teardown
    db.close();
    Ok(())
}
```


## 🧪 Testing

To run the internal integration tests:

```bash
cargo run --example basic
```

## 📄 License

MaazDB-RS is distributed under the MIT License. See `LICENSE` for more information.

---
*Created with ❤️ by Maaz for the Rust ecosystem.*