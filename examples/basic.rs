// FILE PATH: maazdb-rs/examples/basic.rs  cargo run --example basic
use maazdb_rs::MaazDB;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("--- MaazDB Official Rust SDK Example ---");

    // 1. Connect (TLS is automatic)
    let mut db = match MaazDB::connect("127.0.0.1", 8888, "admin", "admin") {
        Ok(client) => {
            println!("✓ Secure TLS 1.3 connection established");
            client
        }
        Err(e) => {
            eprintln!("❌ Connection failed: {:?}", e);
            return Err("Check if MaazDB Server is running!".into());
        }
    };

    // Helper to print server responses
    let run = |client: &mut MaazDB, sql: &str| -> Result<(), Box<dyn Error>> {
        println!("Executing: {}", sql);
        let resp = client.query(sql)?;
        println!("Server: {}", resp.trim());
        Ok(())
    };

    // 2. Setup Database
    run(&mut db, "CREATE DATABASE demo_db;")?;
    run(&mut db, "USE demo_db;")?;
    
    // 3. Create Table       
    run(&mut db, "CREATE TABLE products (id SERIAL PRIMARY KEY, name TEXT, price INT);")?;

    // 4. Insert Data
    run(&mut db, "INSERT INTO products (name, price) VALUES ('Laptop', 1200);")?;
    run(&mut db, "INSERT INTO products (name, price) VALUES ('Smartphone', 800);")?;

    // 5. Query
    println!("\n--- Fetching Data ---");
    let results = db.query("SELECT * FROM products;")?;
    println!("{}", results);

    // 6. Cleanup
    println!("--- Cleaning Up ---");
    run(&mut db, "DROP TABLE products;")?;
    run(&mut db, "DROP DATABASE demo_db;")?;

    db.close();
    println!("\nDone.");
    Ok(())
}