// FILE PATH: src/bin/cli.rs

use std::io::{self, Write};
use std::fs;
use maazdb_rs::MaazDB;
use comfy_table::{Table, presets, Attribute, Cell, CellAlignment};
use colored::*;
use serde_json::Value;

/// Tries to parse a JSON string and print a pretty ASCII table.
/// Returns true if it was a table, false if it was just a string.
fn print_pretty_table(json_response: &str) -> bool {
    let parsed: Result<Value, _> = serde_json::from_str(json_response);

    match parsed {
        Ok(v) => {
            // Check if it has "headers" and "data" (Standard MaazDB Table format)
            if let (Some(headers), Some(data)) = (v["headers"].as_array(), v["data"].as_array()) {
                let mut table = Table::new();
                table
                    .load_preset(presets::UTF8_FULL)
                    .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

                // Add Headers
                let header_row: Vec<Cell> = headers.iter()
                    .map(|h| Cell::new(h.as_str().unwrap_or("")).add_attribute(Attribute::Bold).fg(comfy_table::Color::Cyan))
                    .collect();
                table.set_header(header_row);

                // Add Rows
                for row in data {
                    if let Some(row_arr) = row.as_array() {
                        let row_cells: Vec<Cell> = row_arr.iter()
                            .map(|val| Cell::new(val.as_str().unwrap_or("")))
                            .collect();
                        table.add_row(row_cells);
                    }
                }

                // Print the table
                if data.is_empty() {
                    println!("{}", "Empty set".yellow());
                } else {
                    println!("{}", table);
                    let row_count = data.len();
                    println!("{} {} in set", row_count.to_string().bold(), if row_count == 1 { "row" } else { "rows" });
                }
                return true;
            }
            false
        },
        Err(_) => false,
    }
}

/// Helper to send a single query using the SDK and print the response
fn execute_query(db: &mut MaazDB, query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() { return true; }
    
    // Filter out SQL comments
    let query_without_comments = query.lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    
    if query_without_comments.is_empty() { return true; }
    
    // Execute via SDK
    match db.query(&query_without_comments) {
        Ok(response) => {
            if !response.is_empty() {
                // 1. Try to print as a pretty table (SELECT, SHOW, etc.)
                if !print_pretty_table(&response) {
                    // 2. If not a table, print as a success message (INSERT, UPDATE, etc.)
                    // We assume non-JSON responses are success messages
                    println!("{}", response.green());
                }
            }
            true
        },
        Err(e) => {
            eprintln!("{} {}", "ERROR:".red().bold(), e);
            true 
        }
    }
}

fn main() {
    // Clear screen for a fresh start (optional, works on most terminals)
    print!("\x1B[2J\x1B[1;1H");

    println!("{}", "--------------------------------------------------".bright_blue());
    println!("  {} v12.5", "MaazDB CLI".bold().cyan());
    println!("  Powered by maazdb-rs");
    println!("  Type 'help' for commands or 'exit' to quit.");
    println!("{}", "--------------------------------------------------".bright_blue());

    let host = "127.0.0.1";
    let port = 8888;
    let user = "admin";
    let pass = "admin";

    print!("Connecting to {}:{} as {}... ", host, port, user);
    io::stdout().flush().unwrap();

    match MaazDB::connect(host, port, user, pass) {
        Ok(mut db) => {
            println!("{}", "Success!".green().bold());
            println!("✓ Connected via TLS 1.3\n");
            
            let mut query_buffer = String::new();

            loop {
                // Fancy prompt
                if query_buffer.is_empty() {
                    print!("{}", "maazdb> ".bold().bright_green());
                } else {
                    print!("{}", "    -> ".bold().green());
                }
                io::stdout().flush().unwrap();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() {
                    break;
                }
                let trimmed = input.trim();

                if trimmed.eq_ignore_ascii_case("exit") { 
                    println!("Bye!");
                    break; 
                }
                if trimmed.is_empty() { continue; }
                if trimmed.starts_with("--") { continue; }

                query_buffer.push_str(trimmed);
                query_buffer.push(' ');

                if trimmed.ends_with(';') {
                    let final_query = query_buffer.trim().trim_end_matches(';').to_string();
                    query_buffer.clear();

                    // SOURCE COMMAND LOGIC
                    if final_query.to_uppercase().starts_with("SOURCE") {
                        let path_part = final_query.splitn(2, ' ').collect::<Vec<_>>();
                        if path_part.len() < 2 {
                            eprintln!("{}", "Usage: SOURCE 'path/to/file.sql';".yellow());
                            continue;
                        }
                        let path_str = path_part[1].trim().trim_matches('\'').trim_matches('\"');
                        
                        println!("{} {}", "Reading script:".blue(), path_str);
                        
                        match fs::read_to_string(path_str) {
                            Ok(content) => {
                                for cmd in content.split(';') {
                                    let mut clean_cmd = String::new();
                                    for line in cmd.lines() {
                                        let line_trimmed = line.trim();
                                        if !line_trimmed.is_empty() && !line_trimmed.starts_with("--") {
                                            clean_cmd.push_str(line_trimmed);
                                            clean_cmd.push(' ');
                                        }
                                    }
                                    
                                    if !clean_cmd.trim().is_empty() {
                                        // Print the query being run in a subtle color
                                        println!("{}", format!("Running: {}", clean_cmd.trim()).truecolor(100, 100, 100));
                                        execute_query(&mut db, &clean_cmd);
                                    }
                                }
                                println!("{}", "Script execution finished.".blue());
                            },
                            Err(e) => eprintln!("{} {}", "Failed to read file:".red(), e),
                        }
                    } else {
                        execute_query(&mut db, &final_query);
                    }
                }
            }
            db.close();
        },
        Err(e) => eprintln!("\n{} {}", "Connection Failed:".red().bold(), e),
    }
}