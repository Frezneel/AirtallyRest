use bcrypt::{hash, verify, DEFAULT_COST};

fn main() {
    let password = "AirTally2025!";
    let existing_hash = "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LjEYDCjYqKlwXxwXe";

    // Test verify with existing hash
    match verify(password, existing_hash) {
        Ok(valid) => {
            println!("Existing hash valid: {}", valid);
            if valid {
                println!("✅ Password 'AirTally2025!' matches the migration hash!");
            } else {
                println!("❌ Password does NOT match!");
            }
        }
        Err(e) => {
            println!("❌ Error verifying: {}", e);
        }
    }

    // Generate new hash for comparison
    match hash(password, DEFAULT_COST) {
        Ok(new_hash) => {
            println!("\nNew hash generated:");
            println!("{}", new_hash);

            // Verify the new hash works
            match verify(password, &new_hash) {
                Ok(valid) => println!("New hash verification: {}", valid),
                Err(e) => println!("Error with new hash: {}", e),
            }
        }
        Err(e) => {
            println!("Error generating hash: {}", e);
        }
    }
}
