# Configuration UX - Configuration Wizard Flow

**Goal**: Interactive step-by-step configuration setup

---

## Interactive Wizard Flow

### Step 1: Welcome (Welcome)

**Implementation**:
```rust
async fn step_welcome(&mut self) -> Result<(), ConfigError> {
    println!("\n🧠 Welcome to Memory CLI Configuration!");
    println!("=====================================");
    println!("This wizard will help you set up Memory CLI in just a few steps.");
    println!("You can always re-run this wizard or edit config file later.");
    println!();
    Ok(())
}
```

### Step 2: Database Type Selection (DatabaseType)

**Implementation**:
```rust
async fn step_database_type(&mut self) -> Result<(), ConfigError> {
    println!("\n📊 Step 1 of 3: Database Type");
    println!("------------------------------");
    println!("Choose how you want to store your memory data:");
    println!();
    println!("[1] 📁 Local Database (SQLite)");
    println!("    • Good for: Development, personal use, small teams");
    println!("    • Benefits: Fast, private, no external dependencies");
    println!("    • Storage: Local file on your computer");
    println!();
    println!("[2] ☁️  Cloud Database (Turso)");
    println!("    • Good for: Production, collaboration, backups");
    println!("    • Benefits: Accessible from anywhere, automatic backups");
    println!("    • Storage: Cloud-hosted database");
    println!();
    println!("[3] 🧠 Memory Only (Temporary)");
    println!("    • Good for: Testing, learning, temporary data");
    println!("    • Benefits: Instant setup, no storage needed");
    println!("    • Storage: In-memory (lost when program closes)");
    
    let choice = self.ui.prompt_choice(1, 3)?;
    
    match choice {
        1 => {
            self.config.database.turso_url = None;
            self.config.database.turso_token = None;
            self.config.database.redb_path = Some("./data/memory.redb".to_string());
            println!("✅ Local database selected");
        }
        2 => {
            self.config.database.turso_url = Some("https://your-db.turso.io".to_string());
            self.config.database.redb_path = Some("./data/memory.cache.redb".to_string());
            println!("✅ Cloud database selected");
            println!("💡 You'll need to configure your Turso database URL and token");
        }
        3 => {
            self.config.database.turso_url = None;
            self.config.database.turso_token = None;
            self.config.database.redb_path = None;
            println!("✅ Memory-only mode selected");
        }
        _ => unreachable!(),
    }
    
    Ok(())
}
```

### Step 3: Performance Level (PerformanceLevel)

**Implementation**:
```rust
async fn step_performance_level(&mut self) -> Result<(), ConfigError> {
    println!("\n⚡ Step 2 of 3: Performance Level");
    println!("---------------------------------");
    println!("Choose performance level that matches your needs:");
    println!();
    
    // Adjust options based on database type
    if self.config.database.turso_url.is_some() {
        // Cloud database
        println!("[1] 🐌 Minimal");
        println!("    • Memory: < 100MB");
        println!("    • Episodes: < 1,000");
        println!("    • Good for: Light usage, small projects");
        println!();
        println!("[2] 🏃 Standard");
        println!("    • Memory: < 500MB");
        println!("    • Episodes: < 10,000");
        println!("    • Good for: Most use cases");
        println!();
        println!("[3] 🚀 High");
        println!("    • Memory: < 2GB");
        println!("    • Episodes: < 100,000");
        println!("    • Good for: Heavy usage, large projects");
    } else {
        // Local or memory-only
        println!("[1] 🐌 Minimal");
        println!("    • Memory: < 50MB");
        println!("    • Episodes: < 100");
        println!("    • Good for: Testing, quick experiments");
        println!();
        println!("[2] 🏃 Standard");
        println!("    • Memory: < 200MB");
        println!("    • Episodes: < 1,000");
        println!("    • Good for: Development, personal projects");
        println!();
        println!("[3] 🚀 High");
        println!("    • Memory: < 1GB");
        println!("    • Episodes: < 10,000");
        println!("    • Good for: Production use, large datasets");
    }
    
    let choice = self.ui.prompt_choice(1, 3)?;
    
    let (max_cache, pool_size) = match choice {
        1 => (100, 2),
        2 => (1000, 10),
        3 => (10000, 20),
        _ => unreachable!(),
    };
    
    self.config.storage.max_episodes_cache = max_cache;
    self.config.storage.pool_size = pool_size;
    
    let level_name = match choice {
        1 => "Minimal",
        2 => "Standard", 
        3 => "High",
        _ => unreachable!(),
    };
    
    println!("✅ {} performance level selected", level_name);
    Ok(())
}
```

---

*Status: Wizard Flow Designed*
*Next: CLI Integration*
