mod pipeline;
mod tui;

use anyhow::Context;
use clap::Parser;
use ratatui::{crossterm::event, layout::Size, Viewport};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use std::{
    env,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use comically::{ComicConfig, ComicFile};

use crate::tui::config::ConfigEvent;
use crate::tui::progress::ProgressEvent;

#[derive(Parser, Debug)]
#[command(
    name = "comically",
    about = "comically fast manga & comic optimizer for e-readers",
    version
)]
struct Args {
    /// Optional directory to scan for manga files (defaults to current directory)
    directory: Option<PathBuf>,

    /// Optional output directory (defaults to input {directory}/comically)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable debug logging to file
    #[arg(long)]
    debug: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Only initialize file logging if --debug flag is set
    if args.debug {
        let log_path = "comically.log";
        let log_file =
            std::fs::File::create(log_path).context("Failed to create debug log file")?;

        // Set log level to debug when --debug is used
        std::env::set_var(
            "RUST_LOG",
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME"))),
        );

        let file_subscriber = tracing_subscriber::fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_writer(log_file)
            .with_target(false)
            .with_ansi(false)
            .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());

        tracing_subscriber::registry()
            .with(file_subscriber)
            .with(tracing_error::ErrorLayer::default())
            .init();

        log::info!("Debug logging enabled - writing to {}", log_path);
    } else {
        // Initialize a no-op subscriber when debug is not enabled
        tracing_subscriber::registry()
            .with(tracing_error::ErrorLayer::default())
            .init();
    }

    if cfg!(target_os = "macos") {
        let additional_paths = [
            "/Applications/Kindle Comic Creator/Kindle Comic Creator.app/Contents/MacOS",
            "/Applications/Kindle Previewer 3.app/Contents/lib/fc/bin/",
        ];

        let current_path = env::var("PATH").unwrap_or_default();
        let new_path = additional_paths
            .iter()
            .fold(current_path, |acc, &path| format!("{}:{}", acc, path));

        env::set_var("PATH", new_path);
    }

    let theme = tui::Theme::detect();

    let mut terminal = ratatui::init_with_options(ratatui::TerminalOptions {
        viewport: Viewport::Fullscreen,
    });

    ratatui::crossterm::execute!(
        std::io::stderr(),
        event::EnableMouseCapture,
        ratatui::crossterm::terminal::EnterAlternateScreen
    )?;

    let dimensions = terminal.size()?;

    // need to call this after entering alternate screen, but before reading events
    let picker =
        ratatui_image::picker::Picker::from_query_stdio().context("failed to create picker")?;

    let (event_tx, event_rx) = mpsc::channel();

    thread::spawn({
        let event_tx = event_tx.clone();
        move || input_handling(event_tx, dimensions)
    });

    tui::run(
        args.directory,
        args.output,
        &mut terminal,
        picker,
        theme,
        event_tx,
        event_rx,
    );

    ratatui::crossterm::execute!(
        std::io::stderr(),
        ratatui::crossterm::event::DisableMouseCapture
    )?;
    ratatui::restore();

    Ok(())
}

fn input_handling(tx: mpsc::Sender<Event>, dimensions: Size) {
    const TICK_RATE: Duration = Duration::from_millis(200);

    let mut last_tick = Instant::now();
    let mut last_dimensions: Size = dimensions;

    loop {
        // poll for tick rate duration, if no events, send tick event.
        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout).unwrap() {
            match event::read().unwrap() {
                event::Event::Key(key) => {
                    if tx.send(Event::Key(key)).is_err() {
                        break;
                    }
                }
                event::Event::Resize(width, height) => {
                    // both dimensions must change to be considered a zoom
                    let is_zoom =
                        last_dimensions.width != width && last_dimensions.height != height;
                    let picker = is_zoom
                        .then(|| {
                            ratatui_image::picker::Picker::from_query_stdio()
                                .inspect_err(|e| log::error!("failed to create picker: {e}"))
                                .ok()
                        })
                        .flatten();

                    last_dimensions = Size::new(width, height);

                    if tx.send(Event::Resize(picker)).is_err() {
                        break;
                    }
                }
                event::Event::Mouse(mouse) => {
                    if tx.send(Event::Mouse(mouse)).is_err() {
                        break;
                    }
                }
                _ => {}
            };
        }
        if last_tick.elapsed() >= TICK_RATE {
            if tx.send(Event::Tick).is_err() {
                break;
            }
            last_tick = Instant::now();
        }
    }
}

pub enum Event {
    Mouse(event::MouseEvent),
    Key(event::KeyEvent),
    Tick,
    Resize(Option<ratatui_image::picker::Picker>),
    Progress(ProgressEvent),
    Config(ConfigEvent),
    StartProcessing {
        files: Vec<ComicFile>,
        config: ComicConfig,
        output_dir: PathBuf,
    },
}
