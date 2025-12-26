# Configuration UX - Design

**Goal**: Progressive disclosure architecture for intuitive configuration

---

## Three-Tier Configuration Model

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

---

## Simple Mode Design

### Decision Matrix

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

### One-Call Setup Experience

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
```

---

## Configuration Wizard Design

### Interactive Wizard Flow

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
    pub async fn run() -> Result<Config> {
        println!("🧠 Welcome to Memory CLI Configuration!");
        println!("=====================================");
        println!("This wizard will help you set up Memory CLI in just a few steps.");
        println!("You can always re-run this wizard or edit config file later.");
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
    
    // Step implementations...
}
```

---

## Migration Assistant Design

### Automatic Migration Detection

```rust
pub struct ConfigMigrationAssistant {
    old_config_path: Option<PathBuf>,
    new_config_path: PathBuf,
}

impl ConfigMigrationAssistant {
    pub async fn check_and_migrate() -> Result<bool> {
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
    
    async fn migrate_configuration(old_path: &Path) -> Result<()> {
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

---

## Cross-References

- **Problems**: See [CONFIG_UX_PROBLEMS.md](CONFIG_UX_PROBLEMS.md)
- **Wizard Flow**: See [CONFIG_UX_WIZARD_FLOW.md](CONFIG_UX_WIZARD_FLOW.md)
- **CLI Integration**: See [CONFIG_UX_CLI_INTEGRATION.md](CONFIG_UX_CLI_INTEGRATION.md)
- **Metrics**: See [CONFIG_UX_METRICS.md](CONFIG_UX_METRICS.md)
- **Migration**: See [CONFIG_UX_MIGRATION.md](CONFIG_UX_MIGRATION.md)
- **Recommendations**: See [CONFIG_UX_RECOMMENDATIONS.md](CONFIG_UX_RECOMMENDATIONS.md)

---

*Status: Design Complete*
*Next: Implementation Phase*
