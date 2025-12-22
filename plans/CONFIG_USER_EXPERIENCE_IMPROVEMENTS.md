# Configuration User Experience Improvements Plan

**Goal**: Transform complex configuration into intuitive, guided experience

---

## Current User Experience Problems

### 1. Setup Complexity Issues

**Problem**: Users face overwhelming choices and complex setup
```
Current workflow:
1. Read documentation about configuration files
2. Choose between multiple configuration formats (TOML/YAML/JSON)
3. Manually configure database URLs, tokens, paths
4. Set performance parameters without guidance
5. Debug connectivity issues manually
6. Handle validation errors with poor error messages

Time to first use: 15-30 minutes for basic setup
```

**User Impact**:
- New users abandon during setup
- Existing users make configuration mistakes
- Support burden from configuration issues
- Poor first impression of the tool

### 2. Error Experience Issues

**Problem**: Poor error messages provide no actionable guidance
```
Current error:
"Either Turso URL or redb path must be configured"

Better error:
"No storage backend configured
💡 Fix: Choose one of these options:
  • Local development: Config::simple(DatabaseType::Local, PerformanceLevel::Standard)
  • Cloud setup: Configure database.turso_url and database.turso_token
  • Quick start: Run 'memory-cli config wizard'
"
```

### 3. Learning Curve Issues

**Problem**: No progressive disclosure of complexity
- Beginners see all options at once
- No recommended defaults for common use cases
- No guided path from simple to advanced configuration

---

## Proposed User Experience Design

### 1. Progressive Disclosure Architecture

#### 1.1 Three-Tier Configuration Model

```
TIER 1: Simple Mode (80% of users)
├─ One decision: Database type (Local/Cloud/Memory)
├─ One decision: Performance level (Minimal/Standard/High)
└─ Result: Optimal configuration in 1 call

TIER 2: Wizard Mode (15% of users)
├─ Step-by-step guided setup
├─ Contextual help and suggestions
├─ Real-time validation feedback
└─ Result: Custom configuration with guidance

TIER 3: Advanced Mode (5% of users)
├─ Manual configuration file editing
├─ Full control over all parameters
├─ Advanced validation rules
└─ Result: Fine-tuned configuration
```

#### 1.2 User Journey Mapping

```
NEW USER JOURNEY:
1. First run → Welcome message with options
2. Choose "Simple Setup" → Tier 1 (2 choices, 30 seconds)
3. Configuration created → Immediate validation
4. Ready to use → Success confirmation

EXISTING USER JOURNEY:
1. Run command → Check if new config available
2. If issues → Guided troubleshooting
3. If upgrade needed → Migration assistant
4. If advanced → Manual configuration

POWER USER JOURNEY:
1. Direct to advanced mode
2. Full configuration control
3. Validation with detailed feedback
4. Performance optimization suggestions
```

### 2. Simple Mode Design

#### 2.1 Decision Matrix

```rust
// Simple Mode Decision Matrix
pub enum SimpleSetup {
    // Local development scenarios
    LocalMinimal,    // 30 seconds to setup, < 100MB memory
    LocalStandard,   // 2 minutes to setup, < 1GB memory  
    LocalHigh,       // 5 minutes to setup, < 4GB memory
    
    // Cloud scenarios
    CloudStarter,    // Cloud database, balanced performance
    CloudProduction, // Cloud database, high performance
    
    // Testing scenarios
    MemoryOnly,      // Instant setup, testing only
}

impl SimpleSetup {
    pub fn for_use_case(use_case: &str) -> Self {
        match use_case {
            "development" => Self::LocalStandard,
            "testing" => Self::MemoryOnly,
            "production" => Self::CloudProduction,
            _ => Self::LocalStandard,
        }
    }
}
```

#### 2.2 One-Call Setup Experience

```rust
// CLI Integration for Simple Mode
#[derive(Clap)]
pub enum SimpleCommands {
    /// Quick setup for local development
    Dev {
        /// Performance level (minimal/standard/high)
        #[clap(short, long, default_value = "standard")]
        performance: PerformanceLevel,
        
        /// Skip validation and just create config
        #[clap(short, long)]
        dry_run: bool,
    },
    
    /// Quick setup for production
    Prod {
        /// Cloud provider (turso only for now)
        #[clap(short, long, default_value = "turso")]
        provider: String,
        
        /// Performance level
        #[clap(short, long, default_value = "high")]
        performance: PerformanceLevel,
    },
    
    /// Quick setup for testing
    Test {
        /// Include sample data
        #[clap(short, long)]
        with_sample_data: bool,
    },
}

impl SimpleCommands {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Dev { performance, dry_run } => {
                let config = Config::simple(DatabaseType::Local, *performance)?;
                
                if *dry_run {
                    println!("📋 Configuration preview:");
                    self.display_config_preview(&config)?;
                } else {
                    self.create_and_validate_config(config).await?;
                }
            }
            
            Self::Prod { provider, performance } => {
                if provider != "turso" {
                    return Err("Only 'turso' provider supported currently".into());
                }
                
                let config = Config::simple(DatabaseType::Cloud, *performance)?;
                self.create_and_validate_config(config).await?;
            }
            
            Self::Test { with_sample_data } => {
                let config = Config::simple(DatabaseType::Memory, PerformanceLevel::Minimal)?;
                self.create_and_validate_config(config).await?;
                
                if *with_sample_data {
                    println!("💡 Run 'memory-cli test-data' to load sample episodes");
                }
            }
        }
        
        Ok(())
    }
    
    fn display_config_preview(&self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧠 Memory CLI - Configuration Preview");
        println!("=====================================");
        
        println!("\n📊 Storage Configuration:");
        println!("  • Max Episodes: {}", config.storage.max_episodes_cache);
        println!("  • Cache TTL: {} seconds", config.storage.cache_ttl_seconds);
        println!("  • Pool Size: {}", config.storage.pool_size);
        
        println!("\n💾 Database:");
        if let Some(url) = &config.database.turso_url {
            println!("  • Turso: {}", url);
        }
        if let Some(path) = &config.database.redb_path {
            println!("  • Local: {}", path);
        }
        
        println!("\n⚡ Performance Level: {}", match self {
            Self::Dev { performance, .. } => format!("{:?}", performance),
            Self::Prod { performance, .. } => format!("{:?}", performance), 
            Self::Test { .. } => "Minimal".to_string(),
        });
        
        println!("\n✅ This configuration is optimized for:");
        println!("  • Fast startup and low memory usage");
        println!("  • Good performance for typical workloads");
        println!("  • Easy migration to higher performance levels");
        
        Ok(())
    }
    
    async fn create_and_validate_config(&self, config: Config) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Creating configuration...");
        
        // Validate configuration
        let validation = ConfigValidator::new().validate(&config)?;
        
        if !validation.is_valid {
            println!("❌ Configuration validation failed:");
            for error in &validation.errors {
                println!("  • [{}] {}", error.category, error.message);
                if let Some(suggestion) = &error.suggestion {
                    println!("    💡 {}", suggestion);
                }
            }
            return Err("Configuration validation failed".into());
        }
        
        if !validation.warnings.is_empty() {
            println!("⚠️  Configuration warnings:");
            for warning in &validation.warnings {
                println!("  • [{}] {}", warning.category, warning.message);
                if let Some(suggestion) = &warning.suggestion {
                    println!("    💡 {}", suggestion);
                }
            }
        }
        
        // Save configuration
        let filename = "memory-cli.toml";
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(filename, content)?;
        
        println!("✅ Configuration saved to {}", filename);
        println!("🚀 Run 'memory-cli --help' to get started!");
        
        Ok(())
    }
}
```

### 3. Configuration Wizard Design

#### 3.1 Interactive Wizard Flow

```rust
pub struct ConfigWizard {
    current_step: WizardStep,
    config: Config,
    ui: WizardUI,
}

#[derive(Debug, Clone)]
pub enum WizardStep {
    Welcome,
    DatabaseType,
    PerformanceLevel,
    Customize,
    Review,
    Save,
    Complete,
}

impl ConfigWizard {
    pub async fn run() -> Result<Config, ConfigError> {
        println!("🧠 Welcome to Memory CLI Configuration!");
        println!("=====================================");
        println!("This wizard will help you set up Memory CLI in just a few steps.");
        println!("You can always re-run this wizard or edit the config file later.");
        println!();
        
        let mut wizard = Self {
            current_step: WizardStep::Welcome,
            config: Config::default(),
            ui: WizardUI::new(),
        };
        
        // Run wizard steps
        wizard.run_step(WizardStep::DatabaseType).await?;
        wizard.run_step(WizardStep::PerformanceLevel).await?;
        
        if wizard.ui.prompt_yes_no("\nWould you like to customize advanced settings?")? {
            wizard.run_step(WizardStep::Customize).await?;
        }
        
        wizard.run_step(WizardStep::Review).await?;
        wizard.run_step(WizardStep::Save).await?;
        wizard.run_step(WizardStep::Complete).await?;
        
        Ok(wizard.config)
    }
    
    async fn run_step(&mut self, step: WizardStep) -> Result<(), ConfigError> {
        self.current_step = step.clone();
        
        match step {
            WizardStep::DatabaseType => self.step_database_type().await?,
            WizardStep::PerformanceLevel => self.step_performance_level().await?,
            WizardStep::Customize => self.step_customize().await?,
            WizardStep::Review => self.step_review().await?,
            WizardStep::Save => self.step_save().await?,
            WizardStep::Complete => self.step_complete().await?,
            _ => {}
        }
        
        Ok(())
    }
    
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
                self.config.database.redb_path = Some("./data/memory.redb".to_string());
                println!("✅ Local database selected");
            }
            2 => {
                // We'll configure the URL later
                self.config.database.turso_url = Some("https://your-db.turso.io".to_string());
                self.config.database.redb_path = Some("./data/memory.cache.redb".to_string());
                println!("✅ Cloud database selected");
                println!("💡 You'll need to configure your Turso database URL and token");
            }
            3 => {
                self.config.database.turso_url = None;
                self.config.database.redb_path = None;
                println!("✅ Memory-only mode selected");
            }
            _ => unreachable!(),
        }
        
        Ok(())
    }
    
    async fn step_performance_level(&mut self) -> Result<(), ConfigError> {
        println!("\n⚡ Step 2 of 3: Performance Level");
        println!("---------------------------------");
        println!("Choose the performance level that matches your needs:");
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
    
    async fn step_customize(&mut self) -> Result<(), ConfigError> {
        println!("\n🔧 Step 3 of 4: Advanced Customization");
        println!("------------------------------------");
        println!("You can fine-tune these settings (press Enter for defaults):");
        println!();
        
        // Cache size
        let cache_size = self.ui.prompt_optional_number(
            "Max episodes in cache",
            self.config.storage.max_episodes_cache
        );
        if let Some(size) = cache_size {
            self.config.storage.max_episodes_cache = size;
        }
        
        // Progress bars
        if self.ui.prompt_yes_no("Enable progress bars")? {
            self.config.cli.progress_bars = true;
        } else {
            self.config.cli.progress_bars = false;
        }
        
        // Output format
        println!("Default output format:");
        println!("[1] Human-readable (default)");
        println!("[2] JSON");
        println!("[3] YAML");
        
        let format_choice = self.ui.prompt_choice(1, 3)?;
        self.config.cli.default_format = match format_choice {
            1 => "human".to_string(),
            2 => "json".to_string(),
            3 => "yaml".to_string(),
            _ => unreachable!(),
        };
        
        println!("✅ Advanced settings configured");
        Ok(())
    }
    
    async fn step_review(&mut self) -> Result<(), ConfigError> {
        println!("\n📋 Step 4 of 4: Review Configuration");
        println!("-----------------------------------");
        
        // Validate configuration
        let validation = ConfigValidator::new().validate(&self.config)?;
        
        if !validation.is_valid {
            println!("❌ Configuration has issues that need to be fixed:");
            for error in &validation.errors {
                println!("  • [{}] {}", error.category, error.message);
                if let Some(suggestion) = &error.suggestion {
                    println!("    💡 {}", suggestion);
                }
            }
            println!();
            
            if self.ui.prompt_yes_no("Would you like to go back and fix these issues?")? {
                // Go back to appropriate step based on error category
                return Ok(());
            }
        }
        
        if !validation.warnings.is_empty() {
            println!("⚠️  Configuration warnings:");
            for warning in &validation.warnings {
                println!("  • [{}] {}", warning.category, warning.message);
                if let Some(suggestion) = &warning.suggestion {
                    println!("    💡 {}", suggestion);
                }
            }
            println!();
        }
        
        // Display configuration summary
        println!("Configuration Summary:");
        println!("=====================");
        
        // Database
        if let Some(url) = &self.config.database.turso_url {
            println!("🌐 Database: Cloud (Turso)");
            println!("   URL: {}", url);
            if self.config.database.turso_token.is_some() {
                println!("   Token: Configured");
            } else {
                println!("   Token: Not configured (you'll need to add this)");
            }
        } else if let Some(path) = &self.config.database.redb_path {
            println!("📁 Database: Local (SQLite)");
            println!("   Path: {}", path);
        } else {
            println!("🧠 Database: Memory only (temporary)");
        }
        
        // Performance
        let performance_level = if self.config.storage.max_episodes_cache <= 100 {
            "Minimal"
        } else if self.config.storage.max_episodes_cache <= 1000 {
            "Standard"
        } else {
            "High"
        };
        
        println!("⚡ Performance: {}", performance_level);
        println!("   Max Episodes: {}", self.config.storage.max_episodes_cache);
        println!("   Pool Size: {}", self.config.storage.pool_size);
        
        // CLI
        println!("🖥️  CLI Settings:");
        println!("   Output Format: {}", self.config.cli.default_format);
        println!("   Progress Bars: {}", if self.config.cli.progress_bars { "Enabled" } else { "Disabled" });
        
        Ok(())
    }
    
    async fn step_save(&mut self) -> Result<(), ConfigError> {
        println!("\n💾 Save Configuration");
        println!("--------------------");
        
        if self.ui.prompt_yes_no("Save this configuration to a file?")? {
            let filename = self.ui.prompt_filename("memory-cli.toml")?;
            
            let content = toml::to_string_pretty(&self.config)
                .map_err(|e| ConfigError::InvalidConfig { message: e.to_string() })?;
            
            std::fs::write(&filename, content)
                .map_err(|e| ConfigError::StorageError { message: format!("Failed to save: {}", e) })?;
                
            println!("✅ Configuration saved to {}", filename);
        } else {
            println!("💡 Configuration will be kept in memory (not saved to file)");
        }
        
        Ok(())
    }
    
    async fn step_complete(&mut self) -> Result<(), ConfigError> {
        println!("\n🎉 Configuration Complete!");
        println!("=========================");
        println!("Your Memory CLI is now ready to use.");
        println!();
        println!("Next steps:");
        println!("• Run 'memory-cli --help' to see available commands");
        println!("• Try 'memory-cli episode start' to create your first episode");
        println!("• Use 'memory-cli config validate' to check your setup");
        println!();
        
        if self.config.database.turso_url.is_some() && self.config.database.turso_token.is_none() {
            println!("⚠️  Don't forget to configure your Turso token:");
            println!("   1. Get a token from your Turso dashboard");
            println!("   2. Set environment variable: export TURSO_TOKEN=your_token");
            println!("   3. Or add it to your config file");
            println!();
        }
        
        println!("🚀 Happy memory building!");
        
        Ok(())
    }
}

struct WizardUI;

impl WizardUI {
    pub fn new() -> Self {
        Self
    }
    
    pub fn prompt_choice(&self, min: u32, max: u32) -> Result<u32, ConfigError> {
        use std::io::{self, Write};
        
        loop {
            print!("Enter your choice ({} - {}): ", min, max);
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim().parse::<u32>() {
                Ok(choice) if choice >= min && choice <= max => return Ok(choice),
                _ => println!("Please enter a number between {} and {}.", min, max),
            }
        }
    }
    
    pub fn prompt_yes_no(&self, question: &str) -> Result<bool, ConfigError> {
        use std::io::{self, Write};
        
        println!("{}", question);
        loop {
            print!("(y/n): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => return Ok(true),
                "n" | "no" => return Ok(false),
                _ => println!("Please enter 'y' or 'n'."),
            }
        }
    }
    
    pub fn prompt_optional_number<T: std::str::FromStr>(&self, prompt: &str, default: T) -> Option<T> {
        use std::io::{self, Write};
        
        print!("{} (default: {}): ", prompt, default);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "" => Some(default),
            text => text.parse::<T>().ok(),
        }
    }
    
    pub fn prompt_filename(&self, default: &str) -> Result<String, ConfigError> {
        use std::io::{self, Write};
        
        print!("Filename (default: {}): ", default);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(if input.trim().is_empty() {
            default.to_string()
        } else {
            input.trim().to_string()
        })
    }
}
```

### 4. Migration Assistant Design

#### 4.1 Automatic Migration Detection

```rust
pub struct ConfigMigrationAssistant {
    old_config_path: Option<PathBuf>,
    new_config_path: PathBuf,
}

impl ConfigMigrationAssistant {
    pub async fn check_and_migrate() -> Result<bool, ConfigError> {
        // Check for old configuration files
        let old_paths = vec![
            "memory-cli.toml",
            ".memory-cli.toml", 
            "memory-cli.json",
            "memory-cli.yaml",
        ];
        
        let mut found_old = None;
        for path in &old_paths {
            if Path::new(path).exists() {
                found_old = Some(PathBuf::from(path));
                break;
            }
        }
        
        if let Some(old_path) = found_old {
            println!("🔄 Found existing configuration file: {}", old_path.display());
            
            if Self::prompt_migrate()? {
                Self::migrate_configuration(&old_path).await?;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    async fn prompt_migrate() -> Result<bool, ConfigError> {
        println!("\n🔄 Configuration Migration");
        println!("==========================");
        println!("Would you like to migrate to the new simplified configuration system?");
        println!("Benefits of migration:");
        println!("• Simpler configuration with guided setup");
        println!("• Better validation with helpful error messages");
        println!("• Simple Mode for quick setup");
        println!("• Interactive configuration wizard");
        println!();
        
        let ui = WizardUI::new();
        ui.prompt_yes_no("Migrate to new configuration system?")
    }
    
    async fn migrate_configuration(old_path: &Path) -> Result<(), ConfigError> {
        println!("🔄 Migrating configuration...");
        
        // Load old configuration
        let old_config = Config::load(Some(old_path))?;
        
        // Create new configuration with same settings
        let new_config = Config {
            database: old_config.database,
            storage: old_config.storage,
            cli: old_config.cli,
        };
        
        // Validate new configuration
        let validation = ConfigValidator::new().validate(&new_config)?;
        
        if !validation.is_valid {
            println!("⚠️  Old configuration has issues that need attention:");
            for error in &validation.errors {
                println!("  • [{}] {}", error.category, error.message);
                if let Some(suggestion) = &error.suggestion {
                    println!("    💡 {}", suggestion);
                }
            }
        }
        
        // Save new configuration
        let new_path = "memory-cli.toml";
        let content = toml::to_string_pretty(&new_config)?;
        std::fs::write(new_path, content)?;
        
        // Backup old configuration
        let backup_path = format!("{}.backup", old_path.display());
        std::fs::copy(old_path, &backup_path)?;
        
        println!("✅ Configuration migrated successfully!");
        println!("📁 New configuration: {}", new_path);
        println!("📁 Old configuration backed up: {}", backup_path);
        
        Ok(())
    }
}
```

### 5. CLI Integration

#### 5.1 New Command Structure

```rust
// memory-cli/src/main.rs - Updated CLI structure

#[derive(Clap)]
#[clap(name = "memory-cli")]
#[clap(about = "CLI for Memory - Self-learning episodic memory system")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap)]
enum Command {
    /// Episode management
    Episode(EpisodeCommands),
    
    /// Configuration management  
    Config(ConfigCommands),
    
    /// Quick setup commands
    Quick(QuickCommands),
    
    /// Utilities
    Utils(UtilsCommands),
}

#[derive(Clap)]
enum QuickCommands {
    /// Quick setup for development
    Dev {
        #[clap(short, long, default_value = "standard")]
        performance: PerformanceLevel,
    },
    
    /// Quick setup for production  
    Prod {
        #[clap(short, long)]
        turso_url: Option<String>,
        #[clap(short, long)]
        turso_token: Option<String>,
    },
    
    /// Run configuration wizard
    Wizard,
}

#[derive(Clap)]
enum ConfigCommands {
    /// Validate configuration
    Validate,
    
    /// Configuration wizard
    Wizard,
    
    /// Simple setup commands
    Simple(SimpleCommands),
    
    /// Migration assistant
    Migrate,
    
    /// Show current configuration
    Show,
}

impl Cli {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Command::Episode(episode) => episode.run().await?,
            Command::Config(config) => config.run().await?,
            Command::Quick(quick) => quick.run().await?,
            Command::Utils(utils) => utils.run().await?,
        }
        
        Ok(())
    }
}

impl QuickCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Dev { performance } => {
                let config = Config::simple(DatabaseType::Local, performance)?;
                Self::create_and_validate_config("memory-cli.toml", config).await?;
            }
            
            Self::Prod { turso_url, turso_token } => {
                let mut config = Config::simple(DatabaseType::Cloud, PerformanceLevel::High)?;
                if let Some(url) = turso_url {
                    config.database.turso_url = Some(url);
                }
                if let Some(token) = turso_token {
                    config.database.turso_token = Some(token);
                }
                Self::create_and_validate_config("memory-cli.toml", config).await?;
            }
            
            Self::Wizard => {
                let config = Config::wizard().await?;
                println!("🎉 Configuration wizard completed!");
            }
        }
        
        Ok(())
    }
    
    async fn create_and_validate_config(filename: &str, config: Config) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Creating configuration...");
        
        let validation = ConfigValidator::new().validate(&config)?;
        
        if !validation.is_valid {
            println!("❌ Configuration validation failed:");
            for error in &validation.errors {
                println!("  • [{}] {}", error.category, error.message);
                if let Some(suggestion) = &error.suggestion {
                    println!("    💡 {}", suggestion);
                }
            }
            return Err("Configuration validation failed".into());
        }
        
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(filename, content)?;
        
        println!("✅ Configuration saved to {}", filename);
        println!("🚀 Run 'memory-cli --help' to get started!");
        
        Ok(())
    }
}
```

---

## Success Metrics

### 1. User Experience Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| **Time to First Use** | 15-30 min | < 2 min | New user setup time |
| **Setup Success Rate** | 60% | 95% | Users who complete setup |
| **Configuration Errors** | High | Low | Error rate during setup |
| **Support Tickets** | High | Low | Configuration-related support |

### 2. Adoption Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Simple Mode Usage** | 80% of new users | CLI command analytics |
| **Wizard Completion** | 90% of wizard users | Wizard completion rate |
| **Migration Rate** | 70% of existing users | Migration assistant usage |

### 3. Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Error Message Clarity** | 9/10 user rating | User feedback surveys |
| **Setup Guidance** | 9/10 user rating | User feedback surveys |
| **Configuration Reliability** | 99% | Error-free configuration rate |

---

**UX Design Status**: Complete and ready for implementation
**User Impact**: Transform complex setup into intuitive experience
**Business Value**: Reduce support burden, increase user adoption