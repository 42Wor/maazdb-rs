// FILE PATH: maazdb-rs/examples/basic.rs
// Run with: cargo run --example basic

use maazdb_rs::MaazDB;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- MaazDB Official Rust SDK v2.0.0 Example ---");

    // 1. Connect (TLS and HMAC-SHA256 Handshake are automatic)
    let db = match MaazDB::connect("127.0.0.1", 8888, "admin", "admin").await {
        Ok(client) => {
            println!("✓ Secure TLS 1.3 connection established");
            client
        }
        Err(e) => {
            eprintln!("❌ Connection failed: {:?}", e);
            return Err("Check if MaazDB Server is running!".into());
        }
    };

    // 2. Setup Database
    println!("Executing: CREATE DATABASE demo_db;");
    let _ = db.query("CREATE DATABASE demo_db;").await;
    
    println!("Executing: USE demo_db;");
    db.query("USE demo_db;").await?;
    
    println!("Executing: CREATE TABLE products...");
    db.query("CREATE TABLE products (id SERIAL PRIMARY KEY, name TEXT, price INT);").await?;

    // 3. Insert Data (We can now do this concurrently!)
    println!("Inserting data concurrently...");
    let db_clone1 = db.clone();
    let db_clone2 = db.clone();

    let task1 = tokio::spawn(async move {
        db_clone1.query("INSERT INTO products (name, price) VALUES ('Laptop', 1200);").await
    });

    let task2 = tokio::spawn(async move {
        db_clone2.query("INSERT INTO products (name, price) VALUES ('Smartphone', 800);").await
    });

    // Wait for both inserts to finish
    let _ = tokio::try_join!(task1, task2)?;

    // 4. Query
    println!("\n--- Fetching Data ---");
    let results = db.query("SELECT * FROM products;").await?;
    println!("{}", results);

    // 5. Cleanup
    println!("--- Cleaning Up ---");
    db.query("DROP TABLE products;").await?;
    db.query("DROP DATABASE demo_db;").await?;

    println!("\nDone.");
    Ok(())
}