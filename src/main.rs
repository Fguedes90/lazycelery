mod app;
mod broker;
mod config;
mod error;
mod models;
mod ui;
mod utils;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::time;

use crate::app::App;
use crate::broker::{redis::RedisBroker, Broker};
use crate::config::Config;
use crate::ui::events::{handle_key_event, next_event, AppEvent};

use clap::Subcommand;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Broker URL (e.g., redis://localhost:6379/0)
    #[arg(short, long, global = true)]
    broker: Option<String>,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<std::path::PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize configuration with interactive setup
    Init,
    
    /// Show current configuration
    Config,
    
    /// Set broker URL in configuration
    SetBroker {
        /// Broker URL (e.g., redis://localhost:6379/0)
        url: String,
    },
    
    /// Set UI refresh interval in milliseconds
    SetRefresh {
        /// Refresh interval in milliseconds
        interval: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle subcommands
    match cli.command {
        Some(Commands::Init) => {
            run_init_command().await?;
            return Ok(());
        }
        Some(Commands::Config) => {
            show_config()?;
            return Ok(());
        }
        Some(Commands::SetBroker { url }) => {
            set_broker_url(&url)?;
            return Ok(());
        }
        Some(Commands::SetRefresh { interval }) => {
            set_refresh_interval(interval)?;
            return Ok(());
        }
        None => {
            // Run the main TUI application
            run_tui_app(cli.broker, cli.config).await?;
        }
    }
    
    Ok(())
}

async fn run_tui_app(broker_arg: Option<String>, config_arg: Option<std::path::PathBuf>) -> Result<()> {
    // Load configuration
    let config = if let Some(config_path) = config_arg {
        Config::from_file(config_path)?
    } else {
        Config::load_or_create_default()?
    };

    // Determine broker URL
    let broker_url = broker_arg.unwrap_or_else(|| config.broker.url.clone());

    // Connect to broker
    let broker: Box<dyn Broker> = if broker_url.starts_with("redis://") {
        match RedisBroker::connect(&broker_url).await {
            Ok(broker) => Box::new(broker),
            Err(e) => {
                eprintln!("\n❌ Failed to connect to Celery broker at {}", broker_url);
                eprintln!("\n{}", e);
                eprintln!("\n📋 Quick Setup Guide:");
                eprintln!("1. Make sure Redis is running:");
                eprintln!("   - Docker: docker run -d -p 6379:6379 redis");
                eprintln!("   - macOS: brew services start redis");
                eprintln!("   - Linux: sudo systemctl start redis");
                eprintln!("\n2. Verify Redis is accessible:");
                eprintln!("   redis-cli ping");
                eprintln!("\n3. Run lazycelery with your broker URL:");
                eprintln!("   lazycelery --broker redis://localhost:6379/0");
                eprintln!("\n4. Or create a config file at ~/.config/lazycelery/config.toml:");
                eprintln!("   [broker]");
                eprintln!("   url = \"redis://localhost:6379/0\"");
                eprintln!("\n💡 For more help, visit: https://github.com/Fguedes90/lazycelery");
                std::process::exit(1);
            }
        }
    } else if broker_url.starts_with("amqp://") {
        eprintln!("\n⚠️  AMQP/RabbitMQ support is coming soon!");
        eprintln!("\n📋 Currently supported brokers:");
        eprintln!("   - Redis (redis://localhost:6379/0)");
        eprintln!("\nFor updates, visit: https://github.com/Fguedes90/lazycelery");
        std::process::exit(1);
    } else {
        eprintln!("\n❌ Unknown broker type: {}", broker_url);
        eprintln!("\n📋 Supported broker URLs:");
        eprintln!("   - Redis: redis://localhost:6379/0");
        eprintln!("   - RabbitMQ: amqp://guest:guest@localhost:5672// (coming soon)");
        eprintln!("\nExample: lazycelery --broker redis://localhost:6379/0");
        std::process::exit(1);
    };

    // Create app state
    let mut app = App::new(broker);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal, &mut app, &config).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err}");
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    config: &Config,
) -> Result<()> {
    // Initial data fetch
    app.refresh_data().await?;

    // Set up refresh interval
    let mut refresh_interval = time::interval(Duration::from_millis(config.ui.refresh_interval));
    let tick_rate = Duration::from_millis(50); // 20 FPS max

    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        tokio::select! {
            // Handle user input
            event = next_event(tick_rate) => {
                match event? {
                    AppEvent::Key(key) => {
                        // Check if confirmation dialog needs execution
                        let should_execute = app.show_confirmation && matches!(
                            key.code,
                            crossterm::event::KeyCode::Char('y') |
                            crossterm::event::KeyCode::Char('Y') |
                            crossterm::event::KeyCode::Enter
                        );

                        handle_key_event(key, app);

                        // Execute pending action if confirmed
                        if should_execute {
                            app.execute_pending_action().await?;
                        }

                        if app.should_quit {
                            return Ok(());
                        }
                    }
                    AppEvent::Tick => {}
                    AppEvent::Refresh => {
                        app.refresh_data().await?;
                    }
                }
            }
            // Auto-refresh data
            _ = refresh_interval.tick() => {
                app.refresh_data().await?;
            }
        }
    }
}

async fn run_init_command() -> Result<()> {
    use std::io::{self, Write};
    
    println!("🚀 Welcome to LazyCelery Setup!\n");
    
    // Get config directory
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("lazycelery");
    let config_path = config_dir.join("config.toml");
    
    // Check if config already exists
    if config_path.exists() {
        print!("⚠️  Configuration already exists at {}. Overwrite? (y/N): ", config_path.display());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("❌ Setup cancelled.");
            return Ok(());
        }
    }
    
    // Ask for broker URL
    print!("📡 Enter your Celery broker URL (default: redis://localhost:6379/0): ");
    io::stdout().flush()?;
    
    let mut broker_url = String::new();
    io::stdin().read_line(&mut broker_url)?;
    let broker_url = broker_url.trim();
    let broker_url = if broker_url.is_empty() {
        "redis://localhost:6379/0"
    } else {
        broker_url
    };
    
    // Validate broker URL
    if !broker_url.starts_with("redis://") && !broker_url.starts_with("amqp://") {
        eprintln!("❌ Invalid broker URL. Must start with redis:// or amqp://");
        return Ok(());
    }
    
    // Ask for refresh interval
    print!("🔄 Enter UI refresh interval in milliseconds (default: 1000): ");
    io::stdout().flush()?;
    
    let mut refresh_input = String::new();
    io::stdin().read_line(&mut refresh_input)?;
    let refresh_interval: u64 = refresh_input.trim().parse().unwrap_or(1000);
    
    // Create config
    let config = Config {
        broker: crate::config::BrokerConfig {
            url: broker_url.to_string(),
            timeout: 30,
            retry_attempts: 3,
        },
        ui: crate::config::UiConfig {
            refresh_interval,
            theme: "dark".to_string(),
        },
    };
    
    // Save config
    std::fs::create_dir_all(&config_dir)?;
    let toml_string = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, toml_string)?;
    
    println!("\n✅ Configuration saved to: {}", config_path.display());
    println!("\n📋 You can now run 'lazycelery' to start monitoring your Celery workers!");
    
    // Test connection
    print!("\n🔌 Test connection to broker? (Y/n): ");
    io::stdout().flush()?;
    
    let mut test_input = String::new();
    io::stdin().read_line(&mut test_input)?;
    if !test_input.trim().eq_ignore_ascii_case("n") {
        print!("🔄 Testing connection to {}... ", config.broker.url);
        io::stdout().flush()?;
        
        match test_broker_connection(&config.broker.url).await {
            Ok(_) => println!("✅ Success!"),
            Err(e) => println!("❌ Failed: {}", e),
        }
    }
    
    Ok(())
}

fn show_config() -> Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("lazycelery");
    let config_path = config_dir.join("config.toml");
    
    if !config_path.exists() {
        eprintln!("❌ No configuration found. Run 'lazycelery init' to create one.");
        return Ok(());
    }
    
    let config = Config::from_file(config_path.clone())?;
    
    println!("📋 Current Configuration");
    println!("📍 Location: {}", config_path.display());
    println!("\n[broker]");
    println!("  url = \"{}\"", config.broker.url);
    println!("  timeout = {}", config.broker.timeout);
    println!("  retry_attempts = {}", config.broker.retry_attempts);
    println!("\n[ui]");
    println!("  refresh_interval = {}", config.ui.refresh_interval);
    println!("  theme = \"{}\"", config.ui.theme);
    
    Ok(())
}

fn set_broker_url(url: &str) -> Result<()> {
    // Validate URL
    if !url.starts_with("redis://") && !url.starts_with("amqp://") {
        eprintln!("❌ Invalid broker URL. Must start with redis:// or amqp://");
        return Ok(());
    }
    
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("lazycelery");
    let config_path = config_dir.join("config.toml");
    
    // Load existing config or create default
    let mut config = if config_path.exists() {
        Config::from_file(config_path.clone())?
    } else {
        std::fs::create_dir_all(&config_dir)?;
        Config::default()
    };
    
    // Update broker URL
    config.broker.url = url.to_string();
    
    // Save config
    let toml_string = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, toml_string)?;
    
    println!("✅ Broker URL updated to: {}", url);
    println!("📍 Configuration saved to: {}", config_path.display());
    
    Ok(())
}

fn set_refresh_interval(interval: u64) -> Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("lazycelery");
    let config_path = config_dir.join("config.toml");
    
    // Load existing config or create default
    let mut config = if config_path.exists() {
        Config::from_file(config_path.clone())?
    } else {
        std::fs::create_dir_all(&config_dir)?;
        Config::default()
    };
    
    // Update refresh interval
    config.ui.refresh_interval = interval;
    
    // Save config
    let toml_string = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, toml_string)?;
    
    println!("✅ Refresh interval updated to: {}ms", interval);
    println!("📍 Configuration saved to: {}", config_path.display());
    
    Ok(())
}

async fn test_broker_connection(url: &str) -> Result<()> {
    if url.starts_with("redis://") {
        let _broker = RedisBroker::connect(url).await?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Only Redis brokers are currently supported"))
    }
}
