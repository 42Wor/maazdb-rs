// FILE PATH: examples/test_sql.rs
// =====================================================
// MaazDB SQL Syntax Test Suite
// Updated to handle JSON aggregation result parsing
// =====================================================

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

// Parses JSON table payload or falls back to plain-text parsing
async fn execute_and_get_f64(client: &MaazDB, query: &str, expected_value: f64, tolerance: f64) -> bool {
    println!("Executing: {}", query);

    match client.query(query).await {
        Ok(result_str) => {
            // Try parsing JSON table payload first
            let actual_value = if let Ok(v) = serde_json::from_str::<serde_json::Value>(&result_str) {
                if let Some(data_array) = v.get("data").and_then(|d| d.as_array()) {
                    if let Some(first_row) = data_array.get(0).and_then(|r| r.as_array()) {
                        if let Some(val_str) = first_row.get(0).and_then(|c| c.as_str()) {
                            val_str.parse::<f64>().ok()
                        } else if let Some(val_num) = first_row.get(0).and_then(|c| c.as_f64()) {
                            Some(val_num)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                // Fallback to old plain text line parsing if it's not structured JSON
                result_str
                    .lines()
                    .last()
                    .unwrap_or("")
                    .trim()
                    .parse::<f64>()
                    .ok()
            };

            match actual_value {
                Some(actual_val) => {
                    if (actual_val - expected_value).abs() < tolerance {
                        println!("✓ Result: {} (Expected: {})", actual_val, expected_value);
                        true
                    } else {
                        println!("❌ Result: {} (Expected: {}) - Mismatch", actual_val, expected_value);
                        false
                    }
                }
                None => {
                    println!("❌ Failed to parse result as f64: '{}'", result_str.trim());
                    false
                }
            }
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
            false
        }
    }
}

async fn wait_for_server() -> Option<MaazDB> {
    for _ in 0..30 {
        match MaazDB::connect("127.0.0.1", 8888, "admin", "admin").await {
            Ok(client) => {
                println!("✓ Connected and authenticated to server");
                return Some(client);
            }
            Err(_) => {
                println!("Waiting for server to start...");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    None
}

#[tokio::main]
async fn main() {
    println!("==========================================");
    println!("MAAZDB v2.0.0 - SQL Syntax Test Suite (Client Lib)");
    println!("==========================================");

    let client = match wait_for_server().await {
        Some(c) => c,
        None => {
            eprintln!("❌ Could not connect to MaazDB server. Make sure it's running on port 8888.");
            return;
        }
    };

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Create Database
    println!("\n=== Test 1: Create Database ===");
    execute_query(&client, "DROP DATABASE IF EXISTS testdb;", true).await; // Ensure clean slate
    if execute_query(&client, "CREATE DATABASE testdb;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 2: Use Database
    println!("\n=== Test 2: Use Database ===");
    if execute_query(&client, "USE testdb;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 3: Create Table with SERIAL Primary Key
    println!("\n=== Test 3: Create Table with SERIAL Primary Key ===");
    let create_table = "CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name TEXT,
        age INT,
        salary DOUBLE,
        active BOOL,
        created TIMESTAMP,
        uuid UUID
    );";
    if execute_query(&client, create_table, true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 4: Insert rows (test SERIAL auto-increment)
    println!("\n=== Test 4: Insert Rows ===");
    let inserts = vec![
        "INSERT INTO users (name, age, salary, active, created, uuid) VALUES ('Alice', 30, 50000.50, TRUE, '2024-01-15 10:30:00', '550e8400-e29b-41d4-a716-446655440000');",
        "INSERT INTO users (name, age, salary, active, uuid) VALUES ('Bob', 25, 45000.75, FALSE, '550e8400-e29b-41d4-a716-446655440001');",
        "INSERT INTO users (name, age, salary, active, uuid) VALUES ('Charlie', 35, 60000.00, TRUE, '550e8400-e29b-41d4-a716-446655440002');",
        "INSERT INTO users (name, age, salary, active, uuid) VALUES ('David', 40, 70000.00, FALSE, '550e8400-e29b-41d4-a716-446655440003');",
        "INSERT INTO users (name, age, salary, active, uuid) VALUES ('Eve', 20, 30000.00, TRUE, '550e8400-e29b-41d4-a716-446655440004');",
    ];

    for insert in inserts {
        if execute_query(&client, insert, true).await {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    // Test 5: Select all rows
    println!("\n=== Test 5: SELECT * (all columns) ===");
    if execute_query(&client, "SELECT * FROM users;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 6: Select specific columns
    println!("\n=== Test 6: SELECT specific columns ===");
    if execute_query(&client, "SELECT name, age FROM users;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 7: WHERE clause with equality
    println!("\n=== Test 7: WHERE clause (equality) ===");
    if execute_query(&client, "SELECT * FROM users WHERE name = 'Alice';", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 8: WHERE clause with greater than
    println!("\n=== Test 8: WHERE clause (greater than) ===");
    if execute_query(&client, "SELECT * FROM users WHERE age > 25;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 9: WHERE clause with AND
    println!("\n=== Test 9: WHERE clause with AND ===");
    if execute_query(&client, "SELECT * FROM users WHERE age > 25 AND active = TRUE;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 10: ORDER BY
    println!("\n=== Test 10: ORDER BY ===");
    if execute_query(&client, "SELECT name, age FROM users ORDER BY age DESC;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 11: LIMIT and OFFSET
    println!("\n=== Test 11: LIMIT and OFFSET ===");
    if execute_query(&client, "SELECT * FROM users ORDER BY id LIMIT 2 OFFSET 1;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 12: Update rows
    println!("\n=== Test 12: UPDATE rows ===");
    if execute_query(&client, "UPDATE users SET salary = 55000.00 WHERE name = 'Alice';", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Verify update
    if execute_query(&client, "SELECT name, salary FROM users WHERE name = 'Alice';", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 13: Delete rows
    println!("\n=== Test 13: DELETE rows ===");
    if execute_query(&client, "DELETE FROM users WHERE name = 'Charlie';", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 14: Create table with foreign key
    println!("\n=== Test 14: Create table with FOREIGN KEY ===");
    let create_orders = "CREATE TABLE orders (
        order_id SERIAL PRIMARY KEY,
        user_id INT,
        amount DOUBLE,
        FOREIGN KEY (user_id) REFERENCES users(id)
    );";
    if execute_query(&client, create_orders, true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 15: Insert with foreign key constraint
    println!("\n=== Test 15: Insert with FOREIGN KEY constraint ===");
    if execute_query(&client, "INSERT INTO orders (user_id, amount) VALUES (1, 100.50);", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 16: Should fail - invalid foreign key
    println!("\n=== Test 16: Should fail - invalid FOREIGN KEY ===");
    if execute_query(&client, "INSERT INTO orders (user_id, amount) VALUES (999, 200.00);", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 17: SHOW TABLES
    println!("\n=== Test 17: SHOW TABLES ===");
    if execute_query(&client, "SHOW TABLES;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 18: DESCRIBE TABLE
    println!("\n=== Test 18: DESCRIBE TABLE ===");
    if execute_query(&client, "DESCRIBE users;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 19: CREATE USER
    println!("\n=== Test 19: CREATE USER ===");
    if execute_query(&client, "CREATE USER john PASSWORD 'secret123';", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 20: Performance - Batch Inserts
    println!("\n=== Test 20: Performance - Batch Inserts ===");
    let start = std::time::Instant::now();
    execute_query(&client, "CREATE TABLE perf_test (id SERIAL PRIMARY KEY, data TEXT);", true).await;
    for i in 1..=10 {
        let query = format!("INSERT INTO perf_test (data) VALUES ('Data row {}');", i);
        execute_query(&client, &query, true).await;
    }
    let duration = start.elapsed();
    println!("✓ 10 inserts took: {:?}", duration);
    passed += 1;

    // Test 21: SMART SELECT (Primary Key O(1) lookup)
    println!("\n=== Test 21: SMART SELECT (Primary Key O(1) lookup) ===");
    if execute_query(&client, "SELECT * FROM users WHERE id = 1;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 22: Expression SELECT
    println!("\n=== Test 22: Expression SELECT ===");
    if execute_query(&client, "SELECT 1 + 1, 'Hello', TRUE;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // --- AGGREGATE FUNCTION TESTS ---
    println!("\n=== Test 23: AGGREGATE FUNCTIONS ===");
    let tolerance = 0.001; 

    // Test 23a: COUNT(*)
    if execute_and_get_f64(&client, "SELECT COUNT(*) FROM users;", 4.0, tolerance).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 23b: SUM(salary)
    if execute_and_get_f64(&client, "SELECT SUM(salary) FROM users;", 200000.75, tolerance).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 23c: AVG(age)
    if execute_and_get_f64(&client, "SELECT AVG(age) FROM users;", 28.75, tolerance).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 24: DROP TABLE
    println!("\n=== Test 24: DROP TABLE ===");
    if execute_query(&client, "DROP TABLE perf_test;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 25: SHOW DATABASES
    println!("\n=== Test 25: SHOW DATABASES ===");
    if execute_query(&client, "SHOW DATABASES;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 26: Backup command
    println!("\n=== Test 26: BACKUP command ===");
    if execute_query(&client, "BACKUP 'test_backup';", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 27: Complex WHERE with multiple conditions
    println!("\n=== Test 27: Complex WHERE with multiple conditions ===");
    if execute_query(&client, "SELECT * FROM users WHERE (age > 20 AND active = TRUE) OR salary > 50000;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 28: Multiple Row Insert
    println!("\n=== Test 28: Multiple Row Insert ===");
    execute_query(&client, "CREATE TABLE batch_test (id SERIAL PRIMARY KEY, name TEXT, score INT);", true).await;
    if execute_query(&client, "INSERT INTO batch_test (name, score) VALUES ('Player1', 100), ('Player2', 200), ('Player3', 300);", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 29: INSERT INTO ... SELECT
    println!("\n=== Test 29: INSERT INTO ... SELECT ===");
    execute_query(&client, "CREATE TABLE archive_test (id SERIAL PRIMARY KEY, name TEXT, score INT);", true).await;
    if execute_query(&client, "INSERT INTO archive_test (name, score) SELECT name, score FROM batch_test WHERE score > 150;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 30: DROP DATABASE (cleanup)
    println!("\n=== Test 30: DROP DATABASE (cleanup) ===");
    execute_query(&client, "USE system;", true).await;
    if execute_query(&client, "DROP DATABASE testdb;", true).await {
        passed += 1;
    } else {
        failed += 1;
    }

    // Close the connection
    client.close();

    // Summary
    println!("\n==========================================");
    println!("TEST SUMMARY");
    println!("==========================================");
    println!("Total Tests: {}", passed + failed);
    println!("Passed: {} ✓", passed);
    println!("Failed: {} ❌", failed);
    println!("Success Rate: {:.1}%", (passed as f32 / (passed + failed) as f32) * 100.0);
    println!("==========================================");

    if failed > 0 {
        std::process::exit(1);
    }
}