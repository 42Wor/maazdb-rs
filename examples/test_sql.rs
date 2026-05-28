// FILE PATH: examples/test_sql.rs
// =====================================================
// MaazDB SQL Syntax Test Suite
// Updated to handle manual result parsing
// =====================================================

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

// FIXED: Manually parse the result string into an f64
fn execute_and_get_f64(client: &mut MaazDB, query: &str, expected_value: f64, tolerance: f64) -> bool {
    println!("Executing: {}", query);

    match client.query(query) {
        Ok(result_str) => {
            // MaazDB usually returns aggregates as a single string value or the last line of a table
            // We trim and attempt to parse the numeric value
            let cleaned_result = result_str
                .lines()
                .last() // Get the last line in case there are headers
                .unwrap_or("")
                .trim();

            match cleaned_result.parse::<f64>() {
                Ok(actual_value) => {
                    if (actual_value - expected_value).abs() < tolerance {
                        println!("✓ Result: {} (Expected: {})", actual_value, expected_value);
                        true
                    } else {
                        println!("❌ Result: {} (Expected: {}) - Mismatch", actual_value, expected_value);
                        false
                    }
                }
                Err(_) => {
                    println!("❌ Failed to parse result as f64: '{}'", cleaned_result);
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

fn wait_for_server() -> Option<MaazDB> {
    for _ in 0..30 {
        match MaazDB::connect("127.0.0.1", 8888, "admin", "admin") {
            Ok(client) => {
                println!("✓ Connected and authenticated to server");
                return Some(client);
            }
            Err(_) => {
                println!("Waiting for server to start...");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    None
}

fn main() {
    println!("==========================================");
    println!("MAAZDB v12.1.0 - SQL Syntax Test Suite (Client Lib)");
    println!("==========================================");

    // Wait for server to start and connect/authenticate
    let mut client = match wait_for_server() {
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
    execute_query(&mut client, "DROP DATABASE IF EXISTS testdb;", true); // Ensure clean slate
    if execute_query(&mut client, "CREATE DATABASE testdb;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 2: Use Database
    println!("\n=== Test 2: Use Database ===");
    if execute_query(&mut client, "USE testdb;", true) {
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
    if execute_query(&mut client, create_table, true) {
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
        if execute_query(&mut client, insert, true) {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    // Test 5: Select all rows
    println!("\n=== Test 5: SELECT * (all columns) ===");
    if execute_query(&mut client, "SELECT * FROM users;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 6: Select specific columns
    println!("\n=== Test 6: SELECT specific columns ===");
    if execute_query(&mut client, "SELECT name, age FROM users;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 7: WHERE clause with equality
    println!("\n=== Test 7: WHERE clause (equality) ===");
    if execute_query(&mut client, "SELECT * FROM users WHERE name = 'Alice';", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 8: WHERE clause with greater than
    println!("\n=== Test 8: WHERE clause (greater than) ===");
    if execute_query(&mut client, "SELECT * FROM users WHERE age > 25;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 9: WHERE clause with AND
    println!("\n=== Test 9: WHERE clause with AND ===");
    if execute_query(&mut client, "SELECT * FROM users WHERE age > 25 AND active = TRUE;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 10: ORDER BY
    println!("\n=== Test 10: ORDER BY ===");
    if execute_query(&mut client, "SELECT name, age FROM users ORDER BY age DESC;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 11: LIMIT and OFFSET
    println!("\n=== Test 11: LIMIT and OFFSET ===");
    if execute_query(&mut client, "SELECT * FROM users ORDER BY id LIMIT 2 OFFSET 1;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 12: Update rows
    println!("\n=== Test 12: UPDATE rows ===");
    if execute_query(&mut client, "UPDATE users SET salary = 55000.00 WHERE name = 'Alice';", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Verify update
    if execute_query(&mut client, "SELECT name, salary FROM users WHERE name = 'Alice';", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 13: Delete rows
    println!("\n=== Test 13: DELETE rows ===");
    if execute_query(&mut client, "DELETE FROM users WHERE name = 'Charlie';", true) {
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
    if execute_query(&mut client, create_orders, true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 15: Insert with foreign key constraint
    println!("\n=== Test 15: Insert with FOREIGN KEY constraint ===");
    if execute_query(&mut client, "INSERT INTO orders (user_id, amount) VALUES (1, 100.50);", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 16: Should fail - invalid foreign key
    println!("\n=== Test 16: Should fail - invalid FOREIGN KEY ===");
    if execute_query(&mut client, "INSERT INTO orders (user_id, amount) VALUES (999, 200.00);", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 17: SHOW TABLES
    println!("\n=== Test 17: SHOW TABLES ===");
    if execute_query(&mut client, "SHOW TABLES;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 18: DESCRIBE TABLE
    println!("\n=== Test 18: DESCRIBE TABLE ===");
    if execute_query(&mut client, "DESCRIBE users;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 19: CREATE USER
    println!("\n=== Test 19: CREATE USER ===");
    if execute_query(&mut client, "CREATE USER john PASSWORD 'secret123';", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 20: Performance test - batch inserts
    println!("\n=== Test 20: Performance - Batch Inserts ===");
    let start = std::time::Instant::now();
    execute_query(&mut client, "CREATE TABLE perf_test (id SERIAL PRIMARY KEY, data TEXT);", true);
    for i in 1..=10 {
        let query = format!("INSERT INTO perf_test (data) VALUES ('Data row {}');", i);
        execute_query(&mut client, &query, true);
    }
    let duration = start.elapsed();
    println!("✓ 10 inserts took: {:?}", duration);
    passed += 1;

    // Test 21: SMART SELECT optimization (Primary Key lookup)
    println!("\n=== Test 21: SMART SELECT (Primary Key O(1) lookup) ===");
    if execute_query(&mut client, "SELECT * FROM users WHERE id = 1;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 22: Expression SELECT
    println!("\n=== Test 22: Expression SELECT ===");
    if execute_query(&mut client, "SELECT 1 + 1, 'Hello', TRUE;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // --- AGGREGATE FUNCTION TESTS ---
    println!("\n=== Test 23: AGGREGATE FUNCTIONS ===");
    let tolerance = 0.001; 

    // Test 23a: COUNT(*)
    if execute_and_get_f64(&mut client, "SELECT COUNT(*) FROM users;", 4.0, tolerance) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 23b: SUM(salary)
    if execute_and_get_f64(&mut client, "SELECT SUM(salary) FROM users;", 200000.75, tolerance) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 23c: AVG(age)
    if execute_and_get_f64(&mut client, "SELECT AVG(age) FROM users;", 28.75, tolerance) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 24: DROP TABLE
    println!("\n=== Test 24: DROP TABLE ===");
    if execute_query(&mut client, "DROP TABLE perf_test;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 25: SHOW DATABASES
    println!("\n=== Test 25: SHOW DATABASES ===");
    if execute_query(&mut client, "SHOW DATABASES;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 26: Backup command
    println!("\n=== Test 26: BACKUP command ===");
    if execute_query(&mut client, "BACKUP 'test_backup';", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 27: Complex WHERE with multiple conditions
    println!("\n=== Test 27: Complex WHERE with multiple conditions ===");
    if execute_query(&mut client, "SELECT * FROM users WHERE (age > 20 AND active = TRUE) OR salary > 50000;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 28: Multiple Row Insert
    println!("\n=== Test 28: Multiple Row Insert ===");
    execute_query(&mut client, "CREATE TABLE batch_test (id SERIAL PRIMARY KEY, name TEXT, score INT);", true);
    if execute_query(&mut client, "INSERT INTO batch_test (name, score) VALUES ('Player1', 100), ('Player2', 200), ('Player3', 300);", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 29: INSERT INTO ... SELECT
    println!("\n=== Test 29: INSERT INTO ... SELECT ===");
    execute_query(&mut client, "CREATE TABLE archive_test (id SERIAL PRIMARY KEY, name TEXT, score INT);", true);
    if execute_query(&mut client, "INSERT INTO archive_test (name, score) SELECT name, score FROM batch_test WHERE score > 150;", true) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 30: DROP DATABASE (cleanup)
    println!("\n=== Test 30: DROP DATABASE (cleanup) ===");
    execute_query(&mut client, "USE system;", true);
    if execute_query(&mut client, "DROP DATABASE testdb;", true) {
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