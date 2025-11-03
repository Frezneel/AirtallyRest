// Generate bcrypt password hash
// Run with: cargo run --example generate_password

use bcrypt::{hash, verify, DEFAULT_COST};

fn main() {
    let password = "AirTally2025!";

    println!("Testing password: {}", password);
    println!("Default bcrypt cost: {}", DEFAULT_COST);
    println!();

    // Test existing hash from migration
    let existing_hash = "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LjEYDCjYqKlwXxwXe";
    println!("Testing existing hash from migration:");
    println!("{}", existing_hash);

    match verify(password, existing_hash) {
        Ok(valid) => {
            if valid {
                println!("✅ Existing hash is VALID for password '{}'", password);
            } else {
                println!("❌ Existing hash is INVALID for password '{}'", password);
                println!("   The hash in the database does NOT match the password!");
            }
        }
        Err(e) => {
            println!("❌ Error verifying existing hash: {}", e);
        }
    }

    println!();
    println!("Generating NEW hash with cost {}...", DEFAULT_COST);

    // Generate new hash
    match hash(password, DEFAULT_COST) {
        Ok(new_hash) => {
            println!("✅ New hash generated:");
            println!("{}", new_hash);
            println!();

            // Verify the new hash
            match verify(password, &new_hash) {
                Ok(valid) => {
                    if valid {
                        println!("✅ New hash verified successfully!");
                        println!();
                        println!("===========================================");
                        println!("SQL to update user password:");
                        println!("===========================================");
                        println!("UPDATE users SET password_hash = '{}' WHERE username = 'superuser';", new_hash);
                        println!("===========================================");
                    } else {
                        println!("❌ New hash verification failed!");
                    }
                }
                Err(e) => {
                    println!("❌ Error verifying new hash: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Error generating new hash: {}", e);
        }
    }
}
