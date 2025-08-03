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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Broker URL (e.g., redis://localhost:6379/0)
    #[arg(short, long)]
    broker: Option<String>,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = if let Some(config_path) = args.config {
        Config::from_file(config_path)?
    } else {
        Config::load_or_create_default()?
    };

    // Determine broker URL
    let broker_url = args.broker.unwrap_or_else(|| config.broker.url.clone());

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
