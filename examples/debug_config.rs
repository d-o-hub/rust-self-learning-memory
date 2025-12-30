use std::env;

fn main() {
    println!("=== Configuration Debug ===");

    // Check environment variables
    println!("Environment variables:");
    println!("MEMORY_DATA_DIR: {:?}", env::var("MEMORY_DATA_DIR").ok());
    println!("MEMORY_CACHE_DIR: {:?}", env::var("MEMORY_CACHE_DIR").ok());
    println!("REDB_PATH: {:?}", env::var("REDB_PATH").ok());
    println!("LOCAL_DATABASE_URL: {:?}", env::var("LOCAL_DATABASE_URL").ok());
    println!("TURSO_URL: {:?}", env::var("TURSO_URL").ok());
    println!("TURSO_TOKEN: {:?}", env::var("TURSO_TOKEN").ok());

    // Test smart defaults (simulated)
    println!("\nSmart defaults simulation:");
    let current_dir = env::current_dir().unwrap();
    println!("Current directory: {:?}", current_dir);

    // redb detection
    if let Ok(redb_path) = env::var("REDB_PATH") {
        println!("REDB_PATH from env: {}", redb_path);
    } else {
        let default_redb = current_dir.join("data").join("cache").join("memory.redb");
        println!("Default redb path: {:?}", default_redb);
    }

    // Check if files exist
    println!("\nFile existence check:");
    let files_to_check = [
        "/workspaces/feat-phase3/data/memory.db",
        "/workspaces/feat-phase3/data/memory.redb",
        "/workspaces/feat-phase3/data/cache.redb",
        "/workspaces/feat-phase3/memory-cli/data/memory.redb",
        "/workspaces/feat-phase3/memory-cli.toml",
    ];

    for file in &files_to_check {
        let path = std::path::Path::new(file);
        if path.exists() {
            let metadata = std::fs::metadata(file).unwrap();
            println!("✓ {} ({} bytes)", file, metadata.len());
        } else {
            println!("✗ {} (missing)", file);
        }
    }
}