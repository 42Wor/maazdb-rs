// FILE PATH: src/examples/simple_test.rs

// Import the client from the main library
use maazdb_rs::MaazDB;
use std::thread;
use std::time::Duration;

fn execute_query(client: &mut MaazDB, query: &str, expected_success: bool) -> bool {
    println!("Executing: {}", query);

    match client.query(query) {
        Ok(msg) => {
            if msg.is_empty() {
                println!("✓ Success: (No results)");
            } else {
                println!("✓ Success / Result Set:\n{}", msg.trim());
            }
            expected_success
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
            !expected_success
        }
    }
}

fn wait_for_server() -> Option<MaazDB> {
    println!("Attempting to connect to MaazDB Secure Server...");
    for i in 0..10 {
        // Connects using TLS automatically via client.rs
        match MaazDB::connect("127.0.0.1", 8888, "admin", "admin") {
            Ok(client) => {
                println!("✓ Connected and authenticated securely.");
                return Some(client);
            }
            Err(e) => {
                println!("Attempt {}: Failed ({:?})", i+1, e);
                thread::sleep(Duration::from_millis(1000));
            }
        }
    }
    None
}


fn main() {
    println!("==========================================");
    println!("MAAZDB v11.7.1 - Simple SQL Test (Client Lib)");
    println!("==========================================");

    // Wait for server to start and connect/authenticate
    let mut client = match wait_for_server() {
        Some(c) => c,
        None => {
            eprintln!("❌ Could not connect to MaazDB server.");
            return;
        }
    };

    let mut passed = 0;
    let mut total = 0;

    // Test 1: Create Database (Login is handled by MaazDB::connect)
    println!("\n=== Test 1: Create Database ===");
    total += 1;
    if execute_query(&mut client, "CREATE DATABASE testdb;", true) {
        passed += 1;
    }

    // Test 2: Use Database
    println!("\n=== Test 2: Use Database ===");
    total += 1;
    if execute_query(&mut client, "USE testdb;", true) {
        passed += 1;
    }

    // Test 3: Create Simple Table
    println!("\n=== Test 3: Create Simple Table ===");
    total += 1;
    if execute_query(
        &mut client,
        "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT, age INT);",
        true,
    ) {
        passed += 1;
    }

    // Test 4: Insert Rows
    println!("\n=== Test 4: Insert Rows ===");
    let inserts = vec![
        "INSERT INTO users (name, age) VALUES ('Alice', 30);",
        "INSERT INTO users (name, age) VALUES ('Bob', 25);",
        "INSERT INTO users (name, age) VALUES ('Charlie', 35);",
    ];

    for insert in inserts {
        total += 1;
        if execute_query(&mut client, insert, true) {
            passed += 1;
        }
    }

    // Test 5: Select all rows
    println!("\n=== Test 5: SELECT * ===");
    total += 1;
    if execute_query(&mut client, "SELECT * FROM users;", true) {
        passed += 1;
    }

    // Test 6: Select with WHERE
    println!("\n=== Test 6: SELECT with WHERE ===");
    total += 1;
    if execute_query(&mut client, "SELECT name FROM users WHERE age > 25;", true) {
        passed += 1;
    }

    // Test 7: Update
    println!("\n=== Test 7: UPDATE ===");
    total += 1;
    if execute_query(
        &mut client,
        "UPDATE users SET age = 31 WHERE name = 'Alice';",
        true,
    ) {
        passed += 1;
    }

    // Test 8: Verify Update
    println!("\n=== Test 8: Verify Update ===");
    total += 1;
    if execute_query(&mut client, "SELECT age FROM users WHERE name = 'Alice';", true) {
        passed += 1;
    }

    // Test 9: Delete
    println!("\n=== Test 9: DELETE ===");
    total += 1;
    if execute_query(
        &mut client,
        "DELETE FROM users WHERE name = 'Charlie';",
        true,
    ) {
        passed += 1;
    }

    // Test 10: Verify Delete
    println!("\n=== Test 10: Verify Delete ===");
    total += 1;
    if execute_query(&mut client, "SELECT COUNT(*) FROM users;", true) {
        passed += 1;
    }

    // Test 11: SHOW TABLES
    println!("\n=== Test 11: SHOW TABLES ===");
    total += 1;
    if execute_query(&mut client, "SHOW TABLES;", true) {
        passed += 1;
    }

    // Test 12: DESCRIBE TABLE
    println!("\n=== Test 12: DESCRIBE TABLE ===");
    total += 1;
    if execute_query(&mut client, "DESCRIBE users;", true) {
        passed += 1;
    }

    // Test 13: DROP TABLE
    println!("\n=== Test 13: DROP TABLE ===");
    total += 1;
    if execute_query(&mut client, "DROP TABLE users;", true) {
        passed += 1;
    }

    // Test 14: DROP DATABASE
    println!("\n=== Test 14: DROP DATABASE ===");
    total += 1;
    if execute_query(&mut client, "DROP DATABASE testdb;", true) {
        passed += 1;
    }

    // Close the connection
    client.close();

    // Summary
    println!("\n==========================================");
    println!("TEST SUMMARY");
    println!("==========================================");
    println!("Total Tests: {}", total);
    println!("Passed: {} ✓", passed);
    println!("Failed: {} ❌", total - passed);
    println!(
        "Success Rate: {:.1}%",
        (passed as f32 / total as f32) * 100.0
    );
    println!("==========================================");

    if passed < total {
        std::process::exit(1);
    }
}