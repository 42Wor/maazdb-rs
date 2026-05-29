// FILE PATH: examples/simple_test.rs

use maazdb_rs::MaazDB;
use std::time::Duration;

async fn execute_query(client: &MaazDB, query: &str, expected_success: bool) -> bool {
    println!("Executing: {}", query);

    match client.query(query).await {
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

async fn wait_for_server() -> Option<MaazDB> {
    println!("Attempting to connect to MaazDB Secure Server...");
    for i in 0..10 {
        match MaazDB::connect("127.0.0.1", 8888, "admin", "admin").await {
            Ok(client) => {
                println!("✓ Connected and authenticated securely.");
                return Some(client);
            }
            Err(e) => {
                println!("Attempt {}: Failed ({:?})", i+1, e);
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }
    None
}

#[tokio::main]
async fn main() {
    println!("==========================================");
    println!("MAAZDB v2.0.0 - Simple SQL Test (Client Lib)");
    println!("==========================================");

    // Wait for server to start and connect/authenticate
    let client = match wait_for_server().await {
        Some(c) => c,
        None => {
            eprintln!("❌ Could not connect to MaazDB server.");
            return;
        }
    };

    let mut passed = 0;
    let mut total = 0;

    // Test 1: Create Database
    println!("\n=== Test 1: Create Database ===");
    total += 1;
    if execute_query(&client, "CREATE DATABASE testdb;", true).await {
        passed += 1;
    }

    // Test 2: Use Database
    println!("\n=== Test 2: Use Database ===");
    total += 1;
    if execute_query(&client, "USE testdb;", true).await {
        passed += 1;
    }

    // Test 3: Create Simple Table
    println!("\n=== Test 3: Create Simple Table ===");
    total += 1;
    if execute_query(
        &client,
        "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT, age INT);",
        true,
    ).await {
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
        if execute_query(&client, insert, true).await {
            passed += 1;
        }
    }

    // Test 5: Select all rows
    println!("\n=== Test 5: SELECT * ===");
    total += 1;
    if execute_query(&client, "SELECT * FROM users;", true).await {
        passed += 1;
    }

    // Test 6: Select with WHERE
    println!("\n=== Test 6: SELECT with WHERE ===");
    total += 1;
    if execute_query(&client, "SELECT name FROM users WHERE age > 25;", true).await {
        passed += 1;
    }

    // Test 7: Update
    println!("\n=== Test 7: UPDATE ===");
    total += 1;
    if execute_query(
        &client,
        "UPDATE users SET age = 31 WHERE name = 'Alice';",
        true,
    ).await {
        passed += 1;
    }

    // Test 8: Verify Update
    println!("\n=== Test 8: Verify Update ===");
    total += 1;
    if execute_query(&client, "SELECT age FROM users WHERE name = 'Alice';", true).await {
        passed += 1;
    }

    // Test 9: Delete
    println!("\n=== Test 9: DELETE ===");
    total += 1;
    if execute_query(
        &client,
        "DELETE FROM users WHERE name = 'Charlie';",
        true,
    ).await {
        passed += 1;
    }

    // Test 10: Verify Delete
    println!("\n=== Test 10: Verify Delete ===");
    total += 1;
    if execute_query(&client, "SELECT COUNT(*) FROM users;", true).await {
        passed += 1;
    }

    // Test 11: SHOW TABLES
    println!("\n=== Test 11: SHOW TABLES ===");
    total += 1;
    if execute_query(&client, "SHOW TABLES;", true).await {
        passed += 1;
    }

    // Test 12: DESCRIBE TABLE
    println!("\n=== Test 12: DESCRIBE TABLE ===");
    total += 1;
    if execute_query(&client, "DESCRIBE users;", true).await {
        passed += 1;
    }

    // Test 13: DROP TABLE
    println!("\n=== Test 13: DROP TABLE ===");
    total += 1;
    if execute_query(&client, "DROP TABLE users;", true).await {
        passed += 1;
    }

    // Test 14: DROP DATABASE
    println!("\n=== Test 14: DROP DATABASE ===");
    total += 1;
    if execute_query(&client, "DROP DATABASE testdb;", true).await {
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